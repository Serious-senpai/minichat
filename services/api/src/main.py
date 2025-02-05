from __future__ import annotations

import asyncio

import uvloop
from fastapi import FastAPI

from .endpoints import authorization


__all__ = ("app",)


asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())
app = FastAPI()
app.include_router(authorization.router)
