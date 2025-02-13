from __future__ import annotations

from typing import Any, Callable, Dict, Type, TypeVar

from .channels import Channel, Message
from .status import Status
from .users import User
from ..proto.channels_pb2 import PChannel, PMessage
from ..proto.status_pb2 import PStatus
from ..proto.users_pb2 import PUser


__all__ = ("get_converter",)
SrcT = TypeVar("SrcT")
DstT = TypeVar("DstT")
Converter = Callable[[SrcT], DstT]


_converters: Dict[Type, Dict[Type, Callable[[Any], Any]]] = {}


def get_converter(src: Type[SrcT], dst: Type[DstT]) -> Converter[SrcT, DstT]:
    return _converters[src][dst]


def __declare_converter(src: Type[SrcT], dst: Type[DstT], converter: Converter[SrcT, DstT]) -> None:
    try:
        _converters[src][dst] = converter
    except KeyError:
        _converters[src] = {dst: converter}


__declare_converter(
    PChannel,
    Channel,
    lambda channel: Channel(
        id=channel.id,
        name=channel.name,
        description=channel.description,
        owner=_converters[PUser][User](channel.owner),
    ),
)


__declare_converter(
    PMessage,
    Message,
    lambda message: Message(
        id=message.id,
        content=message.content,
        author=_converters[PUser][User](message.author),
        channel=_converters[PChannel][Channel](message.channel),
    ),
)


__declare_converter(
    PStatus,
    Status,
    lambda status: Status(
        success=status.success,
        message=status.message,
    ),
)


__declare_converter(
    PUser,
    User,
    lambda user: User(
        id=user.id,
        username=user.username,
        permissions=user.permissions,
    ),
)
