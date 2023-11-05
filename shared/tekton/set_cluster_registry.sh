#!/bin/sh

# Document the local registry
# https://github.com/kubernetes/enhancements/tree/master/keps/sig-cluster-lifecycle/generic/1755-communicating-a-local-registry
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
 name: local-registry-hosting
 namespace: kube-public
data:
 localRegistryHosting.v1: |
   host: "localhost:${reg_port}"
   help: "https://kind.sigs.k8s.io/docs/user/local-registry/"
EOF
