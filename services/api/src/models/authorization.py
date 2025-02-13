from __future__ import annotations

from datetime import datetime, timedelta, timezone
from typing import Annotated, Literal

import jwt
import pydantic
from fastapi import Depends, HTTPException
from fastapi.security import OAuth2PasswordBearer

from .users import User
from ..core import ConfigClient, JWT_ALGORITHM, TOKEN_EXPIRATION_MINUTES


__all__ = ("AccountToken",)


class AccountToken(pydantic.BaseModel):
    access_token: str
    token_type: Literal["bearer"]

    @classmethod
    def create(cls, data: User, *, secret_key: str) -> AccountToken:
        encoded = data.model_dump()
        encoded["exp"] = datetime.now(timezone.utc) + timedelta(minutes=TOKEN_EXPIRATION_MINUTES)
        return cls(
            access_token=jwt.encode(encoded, secret_key, algorithm=JWT_ALGORITHM),
            token_type="bearer",
        )

    @staticmethod
    async def verify(
        token: Annotated[
            str,
            Depends(
                OAuth2PasswordBearer(
                    "/auth/token",
                    scheme_name="oauth2",
                ),
            ),
        ],
    ) -> User:
        try:
            payload = jwt.decode(
                token,
                key=await ConfigClient().secret_key(),
                algorithms=[JWT_ALGORITHM],
                options={"require": ["exp"]},
            )
            return User.model_validate(payload, strict=True)

        except Exception:
            raise HTTPException(401, detail="Invalid token")
