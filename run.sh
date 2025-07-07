#/bin/bash



RUST_LOG=info OTEL_ADDR=http://jaeger-all-in-one:4318/v1/traces cargo run
