#!/bin/bash

#! https://stackoverflow.com/a/246128
SCRIPT_DIR=$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)
API_SERVICE_DIR=$(realpath $SCRIPT_DIR/..)

echo "Got root of API service: $API_SERVICE_DIR"

execute() {
    if [ "$EUID" -eq 0 ]
    then
        command=$1
    else
        command="sudo $1"
    fi

    echo "Running \"$command\""
    $command

    status=$?
    if [ $status -ne 0 ]; then
        echo "::error::\"$command\" exit with status $status"
        exit $status
    fi
}

execute "python -m venv /venv"
execute "/venv/bin/pip install -r $API_SERVICE_DIR/requirements.txt"
