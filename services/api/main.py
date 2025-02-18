from __future__ import annotations

import asyncio
import sys

import uvicorn
import uvloop
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from src.endpoints import authorization, channels
from src.core import namespace, parse_args


if __name__ == "__main__":
    parse_args()
    print(namespace, file=sys.stderr)

    asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())
    app = FastAPI(
        title="Minichat API",
        summary="HTTP API for minichat",
    )
    app.include_router(authorization.router)
    app.include_router(channels.router)
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_methods=["*"],
        allow_headers=["*"],
    )

    uvicorn.run(
        app,
        host=namespace.host,
        port=namespace.port,
        workers=namespace.workers,
        log_level=namespace.log_level,
    )
