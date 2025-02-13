from __future__ import annotations

from typing import Annotated

import fastapi
import grpc  # type: ignore
import pydantic
from fastapi.security import OAuth2PasswordRequestFormStrict

from ..core import ConfigClient, format_error, rpc
from ..models.adapters import get_converter
from ..models.authorization import AccountToken
from ..models.status import Status
from ..models.users import User
from ..proto import authorization_pb2, authorization_pb2_grpc, status_pb2, users_pb2


__all__ = ("router",)
router = fastapi.APIRouter(
    prefix="/auth",
)


class _Authorization(pydantic.BaseModel):
    """Data model for authorization headers when registering a new account.

    This should only be used to handle registration requests.
    """
    username: Annotated[str, pydantic.Field(description="The username for authorization")]
    password: Annotated[str, pydantic.Field(description="The password for authorization")]


@router.post(
    "/create",
    name="Account creation",
    description="Register a new account",
    responses={
        200: {
            "description": "Created a new account",
            "model": Status,
        },
        400: {
            "description": "Operation failed",
        },
    },
)
async def create(
    headers: Annotated[_Authorization, fastapi.Header(description="Authorization headers")],
    response: fastapi.Response,
) -> Status:
    channel = await rpc()
    stub = authorization_pb2_grpc.AccountServiceStub(channel)
    try:
        result: status_pb2.PStatus = await stub.Create(
            authorization_pb2.PAuthInfo(
                username=headers.username,
                password=headers.password,
            )
        )

    except grpc.aio.AioRpcError as e:
        response.status_code = fastapi.status.HTTP_400_BAD_REQUEST
        return Status(success=False, message=format_error(e))

    return get_converter(status_pb2.PStatus, Status)(result)


@router.post(
    "/token",
    name="Login",
    description="Verifies login data, returns a JWT on success",
    responses={
        200: {
            "description": "JWT for authorization",
            "model": AccountToken,
        },
        400: {
            "description": "Operation failed",
        },
    },
)
async def token(form: Annotated[OAuth2PasswordRequestFormStrict, fastapi.Depends()]) -> AccountToken:
    channel = await rpc()
    stub = authorization_pb2_grpc.AccountServiceStub(channel)
    try:
        result: users_pb2.PUser = await stub.Login(
            authorization_pb2.PAuthInfo(
                username=form.username,
                password=form.password,
            )
        )

    except grpc.aio.AioRpcError as e:
        raise fastapi.HTTPException(
            status_code=400,
            detail=format_error(e),
        )

    return AccountToken.create(
        data=get_converter(users_pb2.PUser, User)(result),
        secret_key=await ConfigClient().secret_key(),
    )
