# Reference: https://docs.docker.com/reference/dockerfile/
FROM rust:1.84 AS builder

RUN apt-get update && apt-get install -y musl-tools protobuf-compiler
RUN rustup target add x86_64-unknown-linux-musl

# Delay COPY operation as much as possible to avoid cache invalidation: https://stackoverflow.com/a/48553814
COPY . /app

WORKDIR /app/services/data
RUN rm -rf target && cargo build --release --target x86_64-unknown-linux-musl

FROM busybox:1.37-musl AS runner

WORKDIR /app
COPY --from=builder /app/services/data/target/x86_64-unknown-linux-musl/release/data-service .

ENTRYPOINT ["./data-service"]
