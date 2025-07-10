use axum::{
    Router,
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
};
use detect_rs::{detector::Detector, image::ModelImage};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

use serde_json::{Value, json};
use std::{collections::HashMap, env, sync::Arc};

use tracing::{Level, info, span};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;

async fn classify(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    let span = span!(Level::INFO, "form_detection");
    let _enter = span.enter();

    info!("Classifying route");

    let mut img_bytes: Option<bytes::Bytes> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(name) = field.name() {
            if name == "image" {
                if let Ok(data) = field.bytes().await {
                    img_bytes = Some(data);
                }
            }
        }
    }

    info!("Read multipart");

    let b = match img_bytes {
        Some(b) => b,
        None => return Err(AppError::Message(String::from("no image in request"))),
    };

    let model_img = ModelImage::from_bytes("some name", &b);
    let img = match model_img {
        Ok(img) => img,
        Err(_e) => return Err(AppError::Message(String::from("Error classifying image"))),
    };

    info!("Have model image");

    let res = state.detector.detect(img);

    Ok(Json(json!(res)))
}

enum AppError {
    Message(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message) = match self {
            AppError::Message(message) => {
                tracing::error!(message);
                (StatusCode::INTERNAL_SERVER_ERROR, String::from(message))
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

async fn detect(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let span = span!(Level::INFO, "detect");
    let _enter = span.enter();

    info!("Detection route");

    let file = if let Some(name) = params.get("s") {
        match name.as_str() {
            "golden" => "golden.jpg",
            "kitten" => "kitten.png",
            "husky" => "husky.webp",
            _ => return Err(AppError::Message(String::from("invalid file"))),
        }
    } else {
        return Err(AppError::Message(String::from("invalid file")));
    };

    let img = ModelImage::new(file);
    let result = state.detector.detect(img);

    info!("Detection route complete");

    Ok(Json(json!(result)))
}

struct AppState {
    detector: Detector,
}

#[tokio::main]
async fn main() {
    let otel_addr = env::var("OTEL_ADDR").expect("need otel address");

    info!("Trace address: {otel_addr}");

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpJson)
        .with_endpoint(otel_addr)
        .build()
        .expect("OTL exporter to work");

    let resource = Resource::builder().with_service_name("api").build();

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(otlp_exporter)
        .build();

    let tracer = provider.tracer("my_tracer");

    global::set_tracer_provider(provider);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    let span = span!(Level::INFO, "main");
    let _enter = span.enter();

    info!("Starting application");

    let detector = Detector::new();

    info!("Detector initialized");

    let shared_state = Arc::new(AppState { detector });

    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/detect", get(detect))
        .route("/classify", post(classify))
        .with_state(shared_state)
        .layer(cors);

    info!("Starting server on port 8080");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
