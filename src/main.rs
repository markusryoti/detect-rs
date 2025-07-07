use axum::{
    Router,
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
};
use detect_rs::{detector::Detector, image::ModelImage};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};
use tracing::{Level, info, span};

use opentelemetry::{
    KeyValue, global,
    trace::{Tracer, get_active_span},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn test_tracing() {
    let tracer = global::tracer("my_tracer");

    tracer.in_span("testing_tracing", |_cx| sum(1, 3));
}

fn sum(a: i32, b: i32) -> i32 {
    let s = a + b;

    info!(msg = "calculated sum", sum = s);

    get_active_span(|span| {
        span.add_event(
            "An event!".to_string(),
            vec![KeyValue::new("sum", format!("{s}"))],
        );
    });

    s
}

async fn classify(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    // let span = span!(Level::INFO, "form_detection");
    // let _enter = span.enter();

    let tracer = global::tracer("my_tracer");

    tracer
        .in_span("classifying", async |_cx| {
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

            let n_bytes = b.len();

            get_active_span(|span| {
                span.add_event(
                    "form parsed",
                    vec![KeyValue::new("form bytes", format!("{n_bytes}"))],
                )
            });

            let model_img = ModelImage::from_bytes("some name", &b);
            let img = match model_img {
                Ok(img) => img,
                Err(_e) => return Err(AppError::Message(String::from("Error classifying image"))),
            };

            info!("Have model image");

            let res = state.detector.detect(img);

            get_active_span(|span| span.add_event("inference done", vec![]));

            Ok(Json(json!(res)))
        })
        .await
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

    let tracer = global::tracer("my_tracer");

    tracer.in_span("detecting_from_files", |_cx| {
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
    })
}

struct AppState {
    detector: Detector,
}

#[tokio::main]
async fn main() {
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpJson)
        // .with_endpoint("http://jaeger-all-in-one:4318/v1/traces")
        .with_endpoint("http://localhost:4318/v1/traces")
        .build()
        .expect("OTL exporter to work");

    let resource = Resource::builder().with_service_name("api").build();

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(otlp_exporter)
        .build();

    global::set_tracer_provider(provider);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let tracer = global::tracer("my_tracer");
    tracer.in_span("doing_work", |_cx| {
        info!("tracing something");
    });

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
        .route("/trace", get(test_tracing))
        .with_state(shared_state)
        .layer(cors);

    info!("Starting server on port 8080");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
