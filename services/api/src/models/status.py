from __future__ import annotations

import pydantic

from ..proto import status_pb2


__all__ = ("Status",)


class Status(pydantic.BaseModel):
    success: bool
    message: str

    @classmethod
    def from_proto(cls, message: status_pb2.StatusMessage) -> Status:
        return cls(
            success=message.success,
            message=message.message,
        )
