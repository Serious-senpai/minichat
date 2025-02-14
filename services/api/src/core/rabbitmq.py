from __future__ import annotations

import aio_pika
from aio_pika.abc import AbstractChannel

from .cli import namespace
from .utils import Singleton


__all__ = ("amqp",)


async def __initialize_channel() -> AbstractChannel:
    connection = await aio_pika.connect_robust(namespace.amqp_host)
    return await connection.channel()


amqp = Singleton(__initialize_channel)
