from __future__ import annotations

from datetime import datetime, timedelta, timezone
from typing import Annotated, Optional, Literal

import jwt
import pydantic
from fastapi import Depends
from fastapi.security import OAuth2PasswordBearer

from ..core import ConfigClient, JWT_ALGORITHM, TOKEN_EXPIRATION_MINUTES
from ..proto import users_pb2


__all__ = ("Account", "AccountToken")


class Account(pydantic.BaseModel):
    id: int
    username: str
    permissions: int

    @classmethod
    def from_proto(cls, message: users_pb2.PUser) -> Account:
        return cls(
            id=message.id,
            username=message.username,
            permissions=message.permissions,
        )


class AccountToken(pydantic.BaseModel):
    access_token: str
    token_type: Literal["bearer"]

    @classmethod
    def create(cls, data: Account, *, secret_key: str) -> AccountToken:
        encoded = data.model_dump()
        encoded["exp"] = datetime.now(timezone.utc) + timedelta(minutes=TOKEN_EXPIRATION_MINUTES)
        return cls(
            access_token=jwt.encode(encoded, secret_key, algorithm=JWT_ALGORITHM),
            token_type="bearer",
        )

    @staticmethod
    async def verify(token: Annotated[str, Depends(OAuth2PasswordBearer("/auth/token", scheme_name="oauth2"))]) -> Optional[Account]:
        try:
            payload = jwt.decode(
                token,
                key=await ConfigClient().secret_key(),
                algorithms=[JWT_ALGORITHM],
                options={"require": ["exp"]},
            )
            return Account.model_validate(payload)

        except Exception:
            return None
