from __future__ import annotations

import pydantic


__all__ = ("User",)


class User(pydantic.BaseModel):
    id: int
    username: str
    permissions: int
