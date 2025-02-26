[![Lint](https://github.com/Serious-senpai/minichat/actions/workflows/lint.yml/badge.svg)](https://github.com/Serious-senpai/minichat/actions/workflows/lint.yml)
[![Build](https://github.com/Serious-senpai/minichat/actions/workflows/build.yml/badge.svg)](https://github.com/Serious-senpai/minichat/actions/workflows/build.yml)

# minichat

Horizontally scalable system for a minimal chat application.

### Architecture

The client's requests are initially handled by an [nginx reverse proxy](https://hub.docker.com/_/nginx), which serves a static frontend (written in [Vue.js](http://vuejs.org/)) as well as forwards traffic to 2 FastAPI applications. These FastAPI applications then communicate with a Data service (written in Rust), which connects to a ScyllaDB cluster for persistent data storage. Additionally, the Data service publishes messages, and both FastAPI applications subscribe to message queues of a RabbitMQ cluster.

![system](resources/system.drawio.png)

### Deployment

Deployment using [Docker Compose](https://docs.docker.com/compose) is as easy as:
```bash
$ docker compose up -d
```
