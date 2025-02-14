# Reference: https://docs.docker.com/reference/dockerfile/
FROM python:3.12.9-alpine AS builder

RUN apk add bash

# Delay COPY operation as much as possible to avoid cache invalidation: https://stackoverflow.com/a/48553814
COPY . /app

WORKDIR /app/services
RUN find -maxdepth 1 ! -name api ! -name . -type d -exec rm -rf {} +

WORKDIR /app/services/api
RUN bash scripts/install.sh && bash scripts/compile.sh

FROM python:3.12.9-alpine AS runner

COPY --from=builder /venv /venv
COPY --from=builder /app /app

WORKDIR /app/services/api

ENTRYPOINT ["/venv/bin/python", "main.py"]
