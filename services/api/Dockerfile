# Reference: https://docs.docker.com/reference/dockerfile/
FROM python:3.12.9-alpine AS runner

RUN apk add bash

# Delay COPY operation as much as possible to avoid cache invalidation: https://stackoverflow.com/a/48553814
COPY . /app

WORKDIR /app/services/api
RUN bash scripts/install.sh && bash scripts/compile.sh

ENV PATH="/venv/bin:${PATH}"
CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0"]
