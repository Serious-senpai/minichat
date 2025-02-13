from __future__ import annotations

from typing import Annotated, List

import pydantic
from fastapi import APIRouter, Depends

from ..core import rpc
from ..models.adapters import get_converter
from ..models.authorization import AccountToken
from ..models.channels import Channel, Message
from ..models.users import User
from ..proto import channels_pb2, channels_pb2_grpc


__all__ = ("router",)
router = APIRouter(
    prefix="/channels",
)


class __CreateChannelBody(pydantic.BaseModel):
    name: Annotated[str, pydantic.Field(description="The name of the channel")]
    description: Annotated[str, pydantic.Field(description="The description of the channel")]


@router.post(
    "/",
    name="Create a channel",
    description="Create a new channel",
)
async def create_channel(
    body: __CreateChannelBody,
    owner: Annotated[User, Depends(AccountToken.verify)],
) -> Channel:
    stub = channels_pb2_grpc.ChannelServiceStub(await rpc())
    channel: channels_pb2.PChannel = await stub.CreateChannel(
        channels_pb2.PCreateChannelRequest(
            name=body.name,
            description=body.description,
            owner_id=owner.id,
        )
    )

    return Channel(
        id=channel.id,
        name=channel.name,
        description=channel.description,
        owner=User(
            id=channel.owner.id,
            username=channel.owner.username,
            permissions=channel.owner.permissions,
        ),
    )


@router.get(
    "/",
    name="List channels",
    description="Query all channels",
)
async def list_channels() -> List[Channel]:
    stub = channels_pb2_grpc.ChannelServiceStub(await rpc())
    c: channels_pb2.PChannelQueryResult = await stub.Query(None)
    converter = get_converter(channels_pb2.PChannel, Channel)
    return [converter(channel) for channel in c.channels]


class __CreateMessageBody(pydantic.BaseModel):
    content: Annotated[str, pydantic.Field(description="The content of the message")]


@router.post(
    "/{channel_id}/messages",
    name="Create a message",
    description="Create a new message in a channel",
)
async def create_message(
    channel_id: int,
    body: __CreateMessageBody,
    author: Annotated[User, Depends(AccountToken.verify)],
) -> Message:
    stub = channels_pb2_grpc.ChannelServiceStub(await rpc())
    message: channels_pb2.PMessage = await stub.CreateMessage(
        channels_pb2.PCreateMessageRequest(
            content=body.content,
            author_id=author.id,
            channel_id=channel_id,
        )
    )
    return Message(
        id=message.id,
        content=message.content,
        author=User(
            id=message.author.id,
            username=message.author.username,
            permissions=message.author.permissions,
        ),
        channel=Channel(
            id=message.channel.id,
            name=message.channel.name,
            description=message.channel.description,
            owner=User(
                id=message.channel.owner.id,
                username=message.channel.owner.username,
                permissions=message.channel.owner.permissions,
            ),
        ),
    )
