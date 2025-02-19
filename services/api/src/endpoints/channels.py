from __future__ import annotations

from typing import Annotated, List

import aio_pika
import pydantic
from fastapi import APIRouter, Depends, Query, WebSocket

from ..core import amqp, rpc
from ..proto import channels_pb2, channels_pb2_grpc
from ..models.adapters import get_converter
from ..models.authorization import AccountToken
from ..models.channels import Channel, Message
from ..models.users import User


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


@router.websocket(
    "/{channel_id}/ws",
    name="Listen to messages in real-time",
)
async def receive_messages(ws: WebSocket, channel_id: int) -> None:
    await ws.accept()
    channel = await amqp()

    exchange = await channel.declare_exchange(
        "channel-messages",
        aio_pika.ExchangeType.DIRECT,
    )
    queue = await channel.declare_queue()
    await queue.bind(exchange, f"channel-{channel_id}")

    async with queue.iterator() as q:
        data = channels_pb2.PMessage()
        converter = get_converter(channels_pb2.PMessage, Message)

        async for message in q:
            async with message.process():
                data.ParseFromString(message.body)
                await ws.send_json(converter(data).model_dump())


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
    m: channels_pb2.PMessage = await stub.CreateMessage(
        channels_pb2.PCreateMessageRequest(
            content=body.content,
            author_id=author.id,
            channel_id=channel_id,
        )
    )
    return Message(
        id=m.id,
        content=m.content,
        author=User(
            id=m.author.id,
            username=m.author.username,
            permissions=m.author.permissions,
        ),
        channel=Channel(
            id=m.channel.id,
            name=m.channel.name,
            description=m.channel.description,
            owner=User(
                id=m.channel.owner.id,
                username=m.channel.owner.username,
                permissions=m.channel.owner.permissions,
            ),
        ),
    )


@router.get(
    "/{channel_id}/messages",
    name="List messages",
    description="Query messages in a channel history",
)
async def list_messages(
    channel_id: int,
    *,
    newest: Annotated[bool, Query(description="Sort messages from newest to oldest")] = True,
    before_id: Annotated[int, Query(description="The upper limit of snowflake ID")] = (1 << 53) - 1,  # https://github.com/fastapi/fastapi/discussions/6237
    after_id: Annotated[int, Query(description="The lower limit of snowflake ID")] = 0,
    limit: Annotated[int, Query(description="The maximum number of messages to return (maximum 50)")] = 50,
) -> List[Message]:
    stub = channels_pb2_grpc.ChannelServiceStub(await rpc())
    m: channels_pb2.PHistoryQueryResult = await stub.History(
        channels_pb2.PHistoryQuery(
            id=channel_id,
            newest=newest,
            before_id=before_id,
            after_id=after_id,
            limit=limit,
        )
    )

    converter = get_converter(channels_pb2.PMessage, Message)
    return [converter(message) for message in m.messages]
