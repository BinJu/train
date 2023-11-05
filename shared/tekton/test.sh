#!/bin/bash

kubectl apply -n tekton-test -f train.yaml

cmd=$(tkn pipeline start env-deployment -n tekton-test -p opsman_version="3.0" | tail -n 1)
echo "executing commmand: $cmd"
eval $cmd
