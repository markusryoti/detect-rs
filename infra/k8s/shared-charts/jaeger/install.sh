#/bin/bash

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

helm upgrade --install jaeger jaegertracing/jaeger \
  --namespace monitoring --create-namespace \
  -f values-jaeger.yaml

