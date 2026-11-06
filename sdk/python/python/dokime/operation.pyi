# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncIterator
from dataclasses import dataclass
from typing import ClassVar, Literal, Protocol, TypeVar

# Third Party Imports
from google.protobuf.message import Message

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Supplied From The Domain Identity Alphabet During Export.
OWNER_INITIAL: str
OWNER_EXTRA: str

# === Types === #

type Mode = Literal["unary", "client_stream", "server_stream", "bidi"]

# === Classes === #

@dataclass(frozen=True, slots=True)
class Operation[Request: Message, Response: Message, Input: Message, Output: Message]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]
    id: str
    generation: int
    mode: Mode
    codec: Literal["protobuf"]
    request: type[Request]
    response: type[Response]
    input: type[Input] | None
    output: type[Output] | None
    grpc_path: str
    rpc_request: type[Message]
    rpc_response: type[Message]
    run_request: type[Message]
    run_response: type[Message]
    contract: str
    fingerprint: str

    @property
    def run_grpc_path(self) -> str: ...
    def __post_init__(self) -> None: ...

class OperationError(RuntimeError):
    code: int
    message: str
    def __init__(self, code: int, message: str) -> None: ...

class ResponseStream[Response: Message, Output: Message](Protocol):
    __type_params__: ClassVar[tuple[TypeVar, ...]]
    def __aiter__(self) -> AsyncIterator[Output]: ...
    async def response(self) -> Response: ...
    async def close(self) -> None: ...

class OperationHandle[Response: Message, Output: Message](ResponseStream[Response, Output], Protocol):
    __type_params__: ClassVar[tuple[TypeVar, ...]]
    async def cancel(self) -> Response: ...

class Invoker(Protocol):
    async def start[
        Request: Message,
        Response: Message,
        Input: Message,
        Output: Message,
    ](
        self,
        operation: Operation[Request, Response, Input, Output],
        request: Request,
        items: AsyncIterator[Input] | None = None,
    ) -> OperationHandle[Response, Output]: ...
    async def call[Request: Message, Response: Message](
        self,
        operation: Operation[Request, Response, Message, Message],
        request: Request,
    ) -> Response: ...
    async def upload[Request: Message, Response: Message, Input: Message](
        self,
        operation: Operation[Request, Response, Input, Message],
        request: Request,
        items: AsyncIterator[Input],
    ) -> Response: ...
    def download[Request: Message, Response: Message, Output: Message](
        self,
        operation: Operation[Request, Response, Message, Output],
        request: Request,
    ) -> ResponseStream[Response, Output]: ...
    def exchange[Request: Message, Response: Message, Input: Message, Output: Message](
        self,
        operation: Operation[Request, Response, Input, Output],
        request: Request,
        items: AsyncIterator[Input],
    ) -> ResponseStream[Response, Output]: ...

# === Public Functions === #

def owner_segment(value: str) -> bool: ...

# === Private Functions === #

# === Entry Point === #
