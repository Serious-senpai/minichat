from __future__ import annotations

import pydantic

from .users import User


__all__ = ("Message",)


class Channel(pydantic.BaseModel):
    id: int
    name: str
    description: str
    owner: User


class Message(pydantic.BaseModel):
    id: int
    content: str
    author: User
    channel: Channel
