#!/bin/bash

docker run \
	-p 16686:16686 \
	-p 4317:4317 \
	-p 4318:4318 \
	-e COLLECTOR_OTLP_ENABLED=true \
	jaegertracing/all-in-one:latest
