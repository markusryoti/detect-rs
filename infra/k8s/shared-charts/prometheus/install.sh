#/bin/bash

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

helm upgrade --install kube-prometheus-stack prometheus-community/kube-prometheus-stack \
  --namespace monitoring --create-namespace

