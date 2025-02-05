from __future__ import annotations

from typing import Annotated

import fastapi
import grpc  # type: ignore
import pydantic
from fastapi.security import OAuth2PasswordRequestForm

from ..core import ConfigClient, format_error, rpc
from ..models import authorization, status
from ..proto import authorization_pb2, authorization_pb2_grpc


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
)
async def create(
    headers: Annotated[_Authorization, fastapi.Header(description="Authorization headers")],
    response: fastapi.Response,
) -> status.Status:
    channel = await rpc()
    stub = authorization_pb2_grpc.AccountServiceStub(channel)
    try:
        result = await stub.Create(
            authorization_pb2.InfoMessage(
                username=headers.username,
                password=headers.password,
            )
        )

    except grpc.aio.AioRpcError as e:
        response.status_code = fastapi.status.HTTP_400_BAD_REQUEST
        return status.Status(success=False, message=format_error(e))

    return status.Status.from_proto(result)


@router.post(
    "/token",
    name="Login",
    description="Verifies login data, returns a JWT on success",
    responses={
        200: {
            "description": "JWT for authorization",
            "model": authorization.AccountToken,
        },
        400: {
            "description": "Invalid login data",
        }
    },
)
async def token(form: Annotated[OAuth2PasswordRequestForm, fastapi.Depends()]) -> authorization.AccountToken:
    channel = await rpc()
    stub = authorization_pb2_grpc.AccountServiceStub(channel)
    try:
        result = await stub.Login(
            authorization_pb2.InfoMessage(
                username=form.username,
                password=form.password,
            )
        )

    except grpc.aio.AioRpcError as e:
        raise fastapi.HTTPException(
            status_code=400,
            detail=format_error(e),
        )

    return authorization.AccountToken.create(
        data=authorization.Account.from_proto(result),
        secret_key=await ConfigClient().secret_key(),
    )
