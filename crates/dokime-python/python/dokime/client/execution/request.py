# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Durable run admission and request progress."""

# === Imports === #

# Standard Library Imports
import asyncio
import math
from collections.abc import AsyncIterator

# Third Party Imports

# Local Imports
from dokime._native.client import ClientError, ExecutionRequestHandle
from .._transport import Transport, _error
from ..errors import ProtocolError, RpcError
from .. import _wire
from .._generated.dokime.v1 import execution_pb2 as execution
from ..models import Backend, RunStatus

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class RunRequest:
    """An admitted operation. Its durable receipt is independent of live cycle state."""

    def __init__(
        self,
        transport: Transport,
        backend: Backend,
        request_id: str,
        cycle_id: str | None = None,
    ) -> None:
        self._transport = transport
        self._backend = backend
        self.id = request_id
        self._cycle_id = cycle_id
        self._native: ExecutionRequestHandle | None = None

    @property
    def cycle_id(self) -> str:
        """Published cycle identity; read status first when bound by admission ID."""
        if self._cycle_id is None:
            raise ProtocolError("read request.status() before accessing its cycle")
        return self._cycle_id

    @classmethod
    async def resolve(
        cls, transport: Transport, backend: Backend, request_id: str
    ) -> "RunRequest":
        native = await transport.execution_request(request_id)
        try:
            receipt = execution.RunRequestReceipt.FromString(await native.status())
        except ClientError as error:
            raise _receipt_error(error) from error
        result = cls(transport, backend, receipt.request_id, receipt.cycle_id)
        result._native = native
        return result

    async def _handle(self) -> ExecutionRequestHandle:
        if self._native is None:
            self._native = await self._transport.execution_request(
                self.id, cycle_id=self._cycle_id
            )
        return self._native

    def _status(self, payload: bytes) -> RunStatus:
        receipt = execution.RunRequestReceipt.FromString(payload)
        status = _wire.status(receipt)
        self._cycle_id = receipt.cycle_id
        return status

    async def status(self) -> RunStatus:
        """Read published progress through the original native client owner."""
        handle = await self._handle()
        try:
            return self._status(await handle.status())
        except ClientError as error:
            raise _receipt_error(error) from error

    async def wait(
        self, *, timeout: float | None = None, poll_interval: float = 0.2
    ) -> RunStatus:
        """Wait natively; timeout/cancellation never stops remote work."""
        _interval(poll_interval)
        if timeout is not None:
            _interval(timeout)
        async with asyncio.timeout(timeout):
            handle = await self._handle()
            try:
                return self._status(await handle.wait(poll_interval=poll_interval))
            except ClientError as error:
                raise _receipt_error(error) from error

    async def updates(self, *, poll_interval: float = 0.2) -> AsyncIterator[RunStatus]:
        """Read native receipt changes, including the terminal receipt, then close."""
        _interval(poll_interval)
        handle = await self._handle()
        try:
            updates = await handle.updates(poll_interval=poll_interval)
            try:
                while (payload := await updates.message()) is not None:
                    yield self._status(payload)
            finally:
                await updates.close()
        except ClientError as error:
            raise _receipt_error(error) from error


# === Public Functions === #


# === Private Functions === #


def _receipt_error(error: ClientError) -> Exception:
    mapped = _error(error)
    if isinstance(mapped, RpcError) and mapped.code == "DATA_LOSS":
        return ProtocolError(str(mapped))
    return mapped


def _interval(value: float) -> None:
    if not math.isfinite(value) or value <= 0:
        raise ValueError("interval/timeout must be positive and finite")


# === Entry Point === #
