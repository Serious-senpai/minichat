from __future__ import annotations

import pydantic


__all__ = ("Status",)


class Status(pydantic.BaseModel):
    success: bool
    message: str
