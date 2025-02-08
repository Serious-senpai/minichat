# Reference: https://docs.docker.com/reference/dockerfile/
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    iputils-ping \
    protobuf-compiler \
    python-is-python3 \
    python3 \
    python3-pip \
    python3-venv
RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=1.84 -y

COPY services/api /tmp/api-service
RUN bash /tmp/api-service/scripts/install.sh
RUN rm -rf /tmp/api-service
