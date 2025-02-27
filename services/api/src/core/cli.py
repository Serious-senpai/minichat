from __future__ import annotations

import argparse
from typing import Optional, TYPE_CHECKING


__all__ = ("namespace", "parse_args")


class __Namespace(argparse.Namespace):
    if TYPE_CHECKING:
        host: str
        port: int
        workers: Optional[int]
        log_level: str
        data_service: str
        amqp_host: str
        cors: bool


namespace = __Namespace()
__parser = argparse.ArgumentParser(
    description="Run HTTP API server for minichat application",
    formatter_class=argparse.ArgumentDefaultsHelpFormatter,
)
__parser.add_argument("--host", type=str, default="0.0.0.0", help="The host to bind the HTTP server to")
__parser.add_argument("--port", type=int, default=8000, help="The port to bind the HTTP server to")
__parser.add_argument("--workers", type=int, required=False, help="The number of worker processes to run")
__parser.add_argument("--log-level", type=str, default="debug", help="The log level for the application")
__parser.add_argument("--data-service", type=str, default="localhost:16000", help="The URL to the data service to pass to `grpc.aio.insecure_channel`")
__parser.add_argument("--amqp-host", type=str, default="amqp://guest:guest@rabbitmq-proxy:5672", help="The URL to RabbitMQ to pass to `aio_pika.connect_robust`")
__parser.add_argument("--cors", action="store_true", help="Enable CORS for the HTTP server")


def parse_args() -> None:
    __parser.parse_args(namespace=namespace)
