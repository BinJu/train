#!/usr/bin/env bash

echo "### Start local registry"
./start_registry.sh
echo "### Create kind cluster"
./create_kind_cluster.sh
echo "### Document local registry"
./set_cluster_registry.sh

echo "### Create secret"
./create_secret.sh

export KO_DOCKER_REPO='localhost:5000'

echo "### Create Service Account"
kubectl apply -f 200-serviceaccount.yaml

echo "### Install tekton pipeline"
kubectl apply --filename https://storage.googleapis.com/tekton-releases/pipeline/latest/release.yaml
