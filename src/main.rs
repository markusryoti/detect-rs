use std::sync::Arc;

use axum::{Router, extract::State, response::Json, routing::get};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use detect_rs::YOLOV8_CLASS_LABELS;
use image::{GenericImageView, ImageReader, imageops::FilterType};
use ndarray::{Array, Axis, s};
use ort::{
    inputs,
    session::{Session, SessionOutputs, builder::GraphOptimizationLevel},
    util::Mutex,
    value::TensorRef,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct BoundingBox {
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

async fn detect(State(state): State<Arc<AppState>>) -> Json<Value> {
    let original_img = ImageReader::open("model/data/golden-retriever-tongue-out.jpg")
        .unwrap()
        .decode()
        .unwrap();

    let (img_width, img_height) = (original_img.width(), original_img.height());
    let img = original_img.resize_exact(640, 640, FilterType::CatmullRom);

    let mut input = Array::zeros((1, 3, 640, 640));

    for pixel in img.pixels() {
        let x = pixel.0 as _;
        let y = pixel.1 as _;
        let [r, g, b, _] = pixel.2.0;
        input[[0, 0, y, x]] = (r as f32) / 255.;
        input[[0, 1, y, x]] = (g as f32) / 255.;
        input[[0, 2, y, x]] = (b as f32) / 255.;
    }

    let inputs = inputs!["images" => TensorRef::from_array_view(&input).unwrap()];

    let mut model_guard = state.model.lock();
    let result = model_guard.run(inputs);

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
            .filter(|box1| intersection(&boxes[0].0, &box1.0) / union(&boxes[0].0, &box1.0) < 0.7)
            .copied()
            .collect();
    }

    println!("Results: {result:?}");

    Json(json!(result))
}

struct AppState {
    model: Mutex<Session>,
}

#[tokio::main]
async fn main() {
    let model = Session::builder()
        .unwrap()
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .unwrap()
        .with_intra_threads(4)
        .unwrap()
        .commit_from_file("model/yolov8n.onnx")
        .unwrap();

    let shared_state = Arc::new(AppState {
        model: Mutex::new(model),
    });

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/detect", get(detect))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Starting server");

    axum::serve(listener, app).await.unwrap();
}
