# Reference: https://docs.docker.com/reference/dockerfile/
FROM ubuntu:22.04

RUN apt-get update && apt-get install python-is-python3 python3 python3-pip python3-venv -y

COPY . /app

WORKDIR /app/services/api
RUN bash scripts/install.sh && bash scripts/compile.sh

ENV PATH="/venv/bin:${PATH}"
CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0"]
