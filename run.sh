#/bin/bash

export RUST_LOG=info
export OTEL_ADDR=http://localhost:4318/v1/traces

cargo run
