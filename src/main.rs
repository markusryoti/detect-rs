use std::sync::Arc;

use axum::{Router, extract::State, response::Json, routing::get};
use serde_json::{Value, json};

use detect_rs::Detector;

async fn detect(State(state): State<Arc<AppState>>) -> Json<Value> {
    let result = state.detector.detect();

    Json(json!(result))
}

struct AppState {
    detector: Detector,
}

#[tokio::main]
async fn main() {
    let detector = Detector::new();

    let shared_state = Arc::new(AppState { detector });

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/detect", get(detect))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Starting server");

    axum::serve(listener, app).await.unwrap();
}
