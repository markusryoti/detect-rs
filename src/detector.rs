use std::fmt::Debug;

use image::{GenericImageView, imageops::FilterType};
use ndarray::{Array, Axis, s};
use ort::{
    inputs,
    session::{Session, SessionOutputs, builder::GraphOptimizationLevel},
    util::Mutex,
    value::TensorRef,
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use crate::image::ModelImage;

pub struct Detector {
    model: Mutex<Session>,
}

impl Debug for Detector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(write!(f, "")?)
    }
}

impl Detector {
    pub fn new() -> Self {
        let model = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .with_intra_threads(4)
            .unwrap()
            .commit_from_file("model/yolov8n.onnx")
            .unwrap();

        Detector {
            model: Mutex::new(model),
        }
    }

    #[instrument]
    pub fn detect(&self, image: ModelImage) -> Vec<(BoundingBox, &str, f32)> {
        let name = image.get_name();
        let image = image.get_dynamic();

        info!(msg = "Starting detection", name = name);

        let (img_width, img_height) = (image.width(), image.height());
        let img = image.resize_exact(640, 640, FilterType::Gaussian);

        info!("Image resized");

        let mut input = Array::zeros((1, 3, 640, 640));

        for pixel in img.pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b, _] = pixel.2.0;
            input[[0, 0, y, x]] = (r as f32) / 255.;
            input[[0, 1, y, x]] = (g as f32) / 255.;
            input[[0, 2, y, x]] = (b as f32) / 255.;
        }

        let inputs = inputs![TensorRef::from_array_view(&input).unwrap()];

        info!("Tensors created");

        let mut model_guard = self.model.lock();
        let result = model_guard.run(inputs);

        info!("Inference completed");

        let outputs: SessionOutputs = match result {
            Ok(outputs) => outputs,
            Err(e) => panic!("{e}"),
        };

        let output = outputs["output0"]
            .try_extract_array::<f32>()
            .unwrap()
            .t()
            .into_owned();

        let mut boxes = Vec::new();
        let output = output.slice(s![.., .., 0]);

        for row in output.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().copied().collect();

            let (class_id, prob) = row
                .iter()
                // skip bounding box coordinates
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap();

            if prob < 0.5 {
                continue;
            }

            let label = YOLOV8_CLASS_LABELS[class_id];
            let xc = row[0] / 640. * (img_width as f32);
            let yc = row[1] / 640. * (img_height as f32);
            let w = row[2] / 640. * (img_width as f32);
            let h = row[3] / 640. * (img_height as f32);

            boxes.push((
                BoundingBox {
                    x1: xc - w / 2.,
                    y1: yc - h / 2.,
                    x2: xc + w / 2.,
                    y2: yc + h / 2.,
                },
                label,
                prob,
            ));
        }

        boxes.sort_by(|box1, box2| box2.2.total_cmp(&box1.2));
        let mut result = Vec::new();

        while !boxes.is_empty() {
            result.push(boxes[0]);
            boxes = boxes
                .iter()
                .filter(|box1| {
                    intersection(&boxes[0].0, &box1.0) / union(&boxes[0].0, &box1.0) < 0.7
                })
                .copied()
                .collect();
        }

        info!("Result objects created");

        info!(msg = "Detection done", name = name);

        result
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BoundingBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

fn intersection(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    (box1.x2.min(box2.x2) - box1.x1.max(box2.x1)) * (box1.y2.min(box2.y2) - box1.y1.max(box2.y1))
}

fn union(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    ((box1.x2 - box1.x1) * (box1.y2 - box1.y1)) + ((box2.x2 - box2.x1) * (box2.y2 - box2.y1))
        - intersection(box1, box2)
}

pub const YOLOV8_CLASS_LABELS: [&str; 80] = [
    "person",
    "bicycle",
    "car",
    "motorcycle",
    "airplane",
    "bus",
    "train",
    "truck",
    "boat",
    "traffic light",
    "fire hydrant",
    "stop sign",
    "parking meter",
    "bench",
    "bird",
    "cat",
    "dog",
    "horse",
    "sheep",
    "cow",
    "elephant",
    "bear",
    "zebra",
    "giraffe",
    "backpack",
    "umbrella",
    "handbag",
    "tie",
    "suitcase",
    "frisbee",
    "skis",
    "snowboard",
    "sports ball",
    "kite",
    "baseball bat",
    "baseball glove",
    "skateboard",
    "surfboard",
    "tennis racket",
    "bottle",
    "wine glass",
    "cup",
    "fork",
    "knife",
    "spoon",
    "bowl",
    "banana",
    "apple",
    "sandwich",
    "orange",
    "broccoli",
    "carrot",
    "hot dog",
    "pizza",
    "donut",
    "cake",
    "chair",
    "couch",
    "potted plant",
    "bed",
    "dining table",
    "toilet",
    "tv",
    "laptop",
    "mouse",
    "remote",
    "keyboard",
    "cell phone",
    "microwave",
    "oven",
    "toaster",
    "sink",
    "refrigerator",
    "book",
    "clock",
    "vase",
    "scissors",
    "teddy bear",
    "hair drier",
    "toothbrush",
];
