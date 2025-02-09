# Reference: https://docs.docker.com/reference/dockerfile/
FROM rust:1.84 AS builder

RUN apt-get update && apt-get install -y protobuf-compiler

# Delay COPY operation as much as possible to avoid cache invalidation: https://stackoverflow.com/a/48553814
COPY . /app

WORKDIR /app/services/data
RUN rm -rf target && cargo build --release

FROM rust:1.84-slim AS runner

WORKDIR /app
COPY --from=builder /app/services/data/target/release/data .

CMD ["./data"]
