#!/bin/bash

#! https://stackoverflow.com/a/246128
SCRIPT_DIR=$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)
ROOT_DIR=$(realpath $SCRIPT_DIR/../../..)
API_SERVICE_DIR=$(realpath $SCRIPT_DIR/..)

echo "Got root of repository: $ROOT_DIR, root of API service: $API_SERVICE_DIR"

mkdir -p $API_SERVICE_DIR/src/proto
/venv/bin/python -m grpc_tools.protoc \
    -I $ROOT_DIR/protobuf \
    --pyi_out=$API_SERVICE_DIR/src/proto/ \
    --python_out=$API_SERVICE_DIR/src/proto/ \
    --grpc_python_out=$API_SERVICE_DIR/src/proto/ \
    $ROOT_DIR/protobuf/*.proto

for f in $(find $API_SERVICE_DIR/src/proto -name '*.py' -o -name '*.pyi')
do
    sed -i 's/^import \(.*_pb2\)/from . import \1/' $f
    sed -i '1i# mypy: ignore-errors' $f
done
