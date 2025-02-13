from __future__ import annotations

import asyncio

import uvloop
from fastapi import FastAPI

from .endpoints import authorization, channels


__all__ = ("app",)


asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())
app = FastAPI(
    title="Minichat API",
    summary="HTTP API for minichat",
)
app.include_router(authorization.router)
app.include_router(channels.router)
