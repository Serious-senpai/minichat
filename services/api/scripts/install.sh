#!/bin/bash

#! https://stackoverflow.com/a/246128
SCRIPT_DIR=$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)
API_SERVICE_DIR=$(realpath $SCRIPT_DIR/..)

echo "Got root of API service: $API_SERVICE_DIR"

if [ "$EUID" -eq 0 ]
then
    python -m venv /venv
else
    sudo python -m venv /venv
fi

/venv/bin/pip install -r $API_SERVICE_DIR/requirements.txt
