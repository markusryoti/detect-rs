use detect_rs::{Detector, ModelImage};

use axum::{Router, extract::State, response::Json, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::{Level, info, span};

async fn detect(State(state): State<Arc<AppState>>) -> Json<Value> {
    let span = span!(Level::INFO, "detect");
    let _enter = span.enter();

    info!("Detection route");

    let img = ModelImage::new("golden-retriever-tongue-out.jpg");
    let result = state.detector.detect(img);

    info!("Detection route complete");

    Json(json!(result))
}

struct AppState {
    detector: Detector,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let span = span!(Level::INFO, "main");
    let _enter = span.enter();

    info!("Starting application");

    let detector = Detector::new();

    info!("Detector initialized");

    let shared_state = Arc::new(AppState { detector });

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/detect", get(detect))
        .with_state(shared_state);

    info!("Starting server on port 3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
