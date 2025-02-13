from __future__ import annotations

import pydantic

from ..proto import status_pb2


__all__ = ("Status",)


class Status(pydantic.BaseModel):
    success: bool
    message: str
