# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Authenticated native gRPC transport and capability negotiation."""

# === Imports === #

# Standard Library Imports
import asyncio
import math
from pathlib import Path
from typing import Any

# Third Party Imports
from google.protobuf import descriptor_pool, message_factory
from google.protobuf.message import Message

# Local Imports
from dokime._native.client import (
    Client as NativeClient,
    ClientError,
    ExecutionRequestHandle as NativeRequestHandle,
    ResponseStream,
    SuiteStream,
    Target as NativeTarget,
    TestCycleHandle as NativeCycleHandle,
    TestExecutionHandle as NativeExecutionHandle,
)
from ._generated.dokime.v1 import backend_pb2 as backend
from ._generated.dokime.v1 import execution_pb2 as execution
from ._generated.dokime.v1 import catalog_pb2 as catalog
from ._generated.dokime.v1 import value_pb2 as value
from .errors import BackendChanged, ClientClosed, ProtocolError, RpcError, RunRejected
from .models import Backend

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Transport:
    def __init__(
        self,
        endpoint: str,
        token: str | None,
        ca_certificate: str | Path | None,
        timeout: float,
    ) -> None:
        ca = (
            Path(ca_certificate).read_text(encoding="utf-8")
            if ca_certificate is not None
            else None
        )
        self._target = NativeTarget(endpoint, ca_certificate_pem=ca)
        if token is not None and (
            not 32 <= len(token) <= 4096 or not all(33 <= ord(c) <= 126 for c in token)
        ):
            raise ValueError(
                "bearer token must contain 32–4096 visible ASCII characters"
            )
        if self._target.is_remote and not token:
            raise ValueError("HTTPS endpoints require a bearer token")
        if not math.isfinite(timeout) or timeout <= 0:
            raise ValueError("RPC timeout must be positive and finite")

        self._token = token
        self.timeout = timeout
        self._client: NativeClient | None = None
        self._lock = asyncio.Lock()
        self.closed = False
        self.backend: Backend | None = None

    async def connect(self) -> Backend:
        async with self._lock:
            if self.closed:
                raise ClientClosed("DokimeClient is closed")
            if self.backend is not None:
                return self.backend
            try:
                client = await NativeClient.connect(
                    self._target,
                    token=self._token,
                    timeout=self.timeout,
                )
            except ClientError as error:
                raise _error(error, negotiating=True) from error
            self._client = client
            native = client.backend
            self.backend = Backend(
                native.backend_id,
                native.epoch,
                native.api_version,
                native.display_name,
                frozenset(native.methods),
            )
            return self.backend

    async def _invoke(self, service: str, method: str, request: Message) -> Any:
        if self._client is None:
            raise ClientClosed("DokimeClient is closed")
        response = _response_type(service, method)
        try:
            payload = await self._client.unary(
                f"/dokime.v1.{service}/{method}",
                request.SerializeToString(),
                timeout=self.timeout,
            )
            return response.FromString(payload)
        except ClientError as error:
            raise _error(error) from error

    async def unary(self, service: str, method: str, request: Message) -> Any:
        await self.connect()
        return await self._invoke(service, method, request)

    async def cycle(self, cycle_id: str, expected: Backend) -> NativeCycleHandle:
        current = await self.connect()
        if current != expected:
            raise BackendChanged("cycle belongs to another backend connection")
        if self._client is None:
            raise ClientClosed("DokimeClient is closed")
        try:
            return self._client.cycle(cycle_id, timeout=self.timeout)
        except ClientError as error:
            raise _error(error) from error

    async def plan_cycle(
        self,
        suite: str,
        template: str,
        parameters: value.ValueObject,
        *,
        label: str | None,
    ) -> NativeCycleHandle:
        await self.connect()
        if self._client is None:
            raise ClientClosed("DokimeClient is closed")
        try:
            return await self._client.plan_cycle(
                suite,
                template,
                parameters.SerializeToString(),
                label=label,
                timeout=self.timeout,
            )
        except ClientError as error:
            mapped = _error(error)
            if isinstance(mapped, RpcError) and mapped.code == "INVALID_ARGUMENT":
                raise ValueError(str(mapped)) from error
            raise mapped from error

    async def suites(self) -> SuiteStream:
        await self.connect()
        if self._client is None:
            raise ClientClosed("DokimeClient is closed")
        try:
            return await self._client.suites(timeout=self.timeout)
        except ClientError as error:
            raise _error(error) from error

    async def suite(self, identifier: str) -> catalog.SuiteDefinition:
        await self.connect()
        if self._client is None:
            raise ClientClosed("DokimeClient is closed")
        try:
            payload = await self._client.suite(identifier, timeout=self.timeout)
            return catalog.SuiteDefinition.FromString(payload)
        except ClientError as error:
            mapped = _error(error)
            if isinstance(mapped, RpcError) and mapped.code == "INVALID_ARGUMENT":
                raise ValueError(str(mapped)) from error
            raise mapped from error

    async def execution_request(
        self, request_id: str, *, cycle_id: str | None = None
    ) -> NativeRequestHandle:
        await self.connect()
        if self._client is None:
            raise ClientClosed("DokimeClient is closed")
        try:
            return self._client.execution_request(
                request_id, cycle_id=cycle_id, timeout=self.timeout
            )
        except ClientError as error:
            raise _error(error) from error

    async def execution(self, execution_id: str) -> NativeExecutionHandle:
        await self.connect()
        if self._client is None:
            raise ClientClosed("DokimeClient is closed")
        try:
            return self._client.execution(execution_id, timeout=self.timeout)
        except ClientError as error:
            raise _error(error) from error

    async def stream(self, service: str, method: str, request: Message) -> Any:
        await self.connect()
        if self._client is None:
            raise ClientClosed("DokimeClient is closed")
        response = _response_type(service, method)
        try:
            stream = await self._client.stream(
                f"/dokime.v1.{service}/{method}", request.SerializeToString()
            )
            return _Stream(stream, response)
        except ClientError as error:
            raise _error(error) from error

    async def download(
        self,
        request: Message,
        destination: Path,
        expected: tuple[int, str] | None = None,
    ) -> tuple[int, str, bytes]:
        await self.connect()
        if self._client is None:
            raise ClientClosed("DokimeClient is closed")
        try:
            if expected is not None:
                return await self._client.download_artifact(
                    request.SerializeToString(),
                    destination,
                    *expected,
                )
            return await self._client.download_report(
                request.SerializeToString(), destination
            )
        except ClientError as error:
            raise _error(error) from error
        except ValueError as error:
            raise ProtocolError(str(error)) from error

    async def verify(self, expected: Backend, *, live: bool = False) -> None:
        identity = await self.unary(
            "Backend", "GetIdentity", backend.GetIdentityRequest()
        )
        if identity.backend_id != expected.id or (
            live and identity.epoch != expected.epoch
        ):
            raise BackendChanged(
                "backend identity or live epoch changed; recover published evidence through History"
            )

    async def close(self) -> None:
        async with self._lock:
            self.closed = True
            if self._client is not None:
                await self._client.close()
                self._client = None
            self._token = None


class _Stream:
    def __init__(self, stream: ResponseStream, response: type[Message]) -> None:
        self._stream = stream
        self._response = response

    def __aiter__(self):
        return self

    async def __anext__(self) -> Message:
        try:
            payload = await self._stream.message()
        except ClientError as error:
            raise _error(error) from error
        if payload is None:
            raise StopAsyncIteration
        return self._response.FromString(payload)

    async def close(self) -> None:
        await self._stream.close()


# === Public Functions === #

# === Private Functions === #


def _response_type(service: str, method: str) -> type[Message]:
    descriptor = descriptor_pool.Default().FindServiceByName(f"dokime.v1.{service}")
    return message_factory.GetMessageClass(
        descriptor.methods_by_name[method].output_type
    )


def _error(error: ClientError, *, negotiating: bool = False) -> Exception:
    if error.kind == "execution_rejected":
        refusal = execution.Rejection.FromString(error.details)
        return RunRejected(refusal.code, refusal.message)
    if error.kind == "wait_timeout":
        return TimeoutError(str(error))
    if error.kind == "closed":
        return ClientClosed(str(error))
    if error.kind == "identity_changed":
        return ProtocolError(str(error)) if negotiating else BackendChanged(str(error))
    if error.kind == "incompatible_api":
        return ProtocolError(str(error))
    if error.kind in {"invalid_target", "authentication_required"}:
        return ValueError(str(error))
    codes = (
        "OK",
        "CANCELLED",
        "UNKNOWN",
        "INVALID_ARGUMENT",
        "DEADLINE_EXCEEDED",
        "NOT_FOUND",
        "ALREADY_EXISTS",
        "PERMISSION_DENIED",
        "RESOURCE_EXHAUSTED",
        "FAILED_PRECONDITION",
        "ABORTED",
        "OUT_OF_RANGE",
        "UNIMPLEMENTED",
        "INTERNAL",
        "UNAVAILABLE",
        "DATA_LOSS",
        "UNAUTHENTICATED",
    )
    code = (
        codes[error.code]
        if error.code is not None and 0 <= error.code < len(codes)
        else "UNAVAILABLE"
    )
    return RpcError(code, str(error))


# === Entry Point === #
