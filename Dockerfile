FROM rust:1.87-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    # libstdc++6 \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./

COPY . .

RUN cargo build --release --locked --target x86_64-unknown-linux-gnu

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/detect-rs ./detect-rs

COPY model/yolov8n.onnx model/yolov8n.onnx

EXPOSE 8080

CMD ["./detect-rs"]
