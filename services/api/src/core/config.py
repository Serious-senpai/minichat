from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import TypedDict


__all__ = (
    "API_SERVICES",
    "ROOT",
    "BCRYPT_COST",
    "EPOCH",
    "JWT_ALGORITHM",
    "TOKEN_EXPIRATION_MINUTES",
)


API_SERVICES = Path(__file__).parent.parent.parent.resolve()
ROOT = API_SERVICES.parent.parent.resolve()


_Setup = TypedDict(
    "_Setup",
    {
        "bcrypt-cost": int,
        "epoch": int,
        "jwt-algorithm": str,
        "token-expiration-minutes": int,
    },
)
with open(ROOT / "setup.json", "r", encoding="utf-8") as _f:
    _data: _Setup = json.load(_f)


BCRYPT_COST = _data["bcrypt-cost"]
EPOCH = datetime.fromtimestamp(_data["epoch"] / 1000, tz=timezone.utc)
JWT_ALGORITHM = _data["jwt-algorithm"]
TOKEN_EXPIRATION_MINUTES = _data["token-expiration-minutes"]
