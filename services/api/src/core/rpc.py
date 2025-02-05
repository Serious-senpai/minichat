from __future__ import annotations

from typing import ClassVar, Optional, TYPE_CHECKING

import grpc  # type: ignore

from .utils import Singleton
from ..proto import config_pb2, config_pb2_grpc


__all__ = ("rpc", "format_error", "ConfigClient")


async def _initialize_channel() -> grpc.aio.Channel:
    channel = grpc.aio.insecure_channel("data-service:16000")
    await channel.channel_ready()
    return channel


rpc = Singleton(_initialize_channel)


def format_error(e: grpc.aio.AioRpcError) -> str:
    return f"[{e.code().name}] {e.details()}"


class ConfigClient:

    __instance__: ClassVar[Optional[ConfigClient]] = None
    __slots__ = ("_stub",)
    if TYPE_CHECKING:
        _stub: Optional[config_pb2_grpc.ConfigServiceStub]

    def __new__(cls) -> ConfigClient:
        if cls.__instance__ is None:
            cls.__instance__ = self = super().__new__(cls)
            self._stub = None

        return cls.__instance__

    async def stub(self) -> config_pb2_grpc.ConfigServiceStub:
        if self._stub is None:
            self._stub = config_pb2_grpc.ConfigServiceStub(await rpc())

        return self._stub

    async def secret_key(self) -> str:
        s = await self.stub()
        m = await s.StringConfig(config_pb2.ConfigRequestMessage(config_type=config_pb2.ConfigType.SECRET_KEY))
        return m.value
