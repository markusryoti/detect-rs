#/bin/bash

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

helm upgrade --install otel-collector open-telemetry/opentelemetry-collector \
	--namespace monitoring --create-namespace \
	-f values-otel.yaml --set image.repository="otel/opentelemetry-collector-k8s"

