from __future__ import annotations

import asyncio
import os
import sys
from pathlib import Path

import uvicorn
import uvloop
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from src.endpoints import auth, channels
from src.core import namespace, parse_args


asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())
app = FastAPI(
    title="Minichat API",
    summary="HTTP API for minichat",
)
app.include_router(auth.router)
app.include_router(channels.router)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)
parse_args()


if __name__ == "__main__":
    print(f"[{os.getpid()}]", namespace, file=sys.stderr)
    uvicorn.run(
        Path(__file__).stem + ":app",
        host=namespace.host,
        port=namespace.port,
        workers=namespace.workers,
        log_level=namespace.log_level,
    )
