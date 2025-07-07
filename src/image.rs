use std::time::Instant;

use image::{DynamicImage, ImageReader, load_from_memory};
use tracing::{info, instrument};

pub struct ModelImage {
    name: String,
    image: DynamicImage,
}

pub enum ModelImageError {
    Message(String),
}

impl ModelImage {
    #[instrument(level = "info", name = "load_image")]
    pub fn new(name: &str) -> Self {
        info!("Start loading");

        let file_name = format!("model/data/{name}");

        let img = ImageReader::open(file_name).unwrap().decode().unwrap();

        info!("Image loaded");

        ModelImage {
            name: String::from(name),
            image: img,
        }
    }

    #[instrument(level = "info", name = "load_image_from_bytes", skip(b))]
    pub fn from_bytes(name: &str, b: &[u8]) -> Result<Self, ModelImageError> {
        info!("Start loading");

        let start = Instant::now();

        let img = match load_from_memory(&b) {
            Ok(i) => i,
            Err(e) => return Err(ModelImageError::Message(e.to_string())),
        };

        let duration = start.elapsed();
        info!("Image loaded in {:.2?}", duration);

        Ok(ModelImage {
            name: String::from(name),
            image: img,
        })
    }

    pub fn from_dynamic(name: &str, img: DynamicImage) -> Self {
        ModelImage {
            name: String::from(name),
            image: img,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_dynamic(self) -> DynamicImage {
        self.image
    }
}
