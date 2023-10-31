#!/bin/bash

export TEST_USER=testuser
export TEST_PASS=testpassword

kubectl create secret docker-registry secret-tekton \
  --docker-username=$TEST_USER \
  --docker-password=$TEST_PASS \
  --docker-server=localhost:5000 \
   --namespace=tekton-pipelines
