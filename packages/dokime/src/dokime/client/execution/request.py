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
from uuid import UUID

# Third Party Imports
from .. import _wire

# Local Imports
from .._transport import Transport
from ..errors import ProtocolError
from ..models import Backend, RunStatus

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class RunRequest:
    """An admitted operation whose durable receipt is independent of live state."""

    def __init__(
        self,
        transport: Transport,
        backend: Backend,
        request_id: str,
        cycle_id: str | None = None,
    ) -> None:
        _identity(request_id)
        if cycle_id is not None:
            _identity(cycle_id)
        self._transport = transport
        self._backend = backend
        self.id = request_id
        self._cycle_id = cycle_id

    @property
    def cycle_id(self) -> str:
        if self._cycle_id is None:
            raise ProtocolError("read request.status() before accessing its cycle")
        return self._cycle_id

    @classmethod
    async def resolve(
        cls, transport: Transport, backend: Backend, request_id: str
    ) -> "RunRequest":
        result = cls(transport, backend, request_id)
        await result.status()
        return result

    async def status(self) -> RunStatus:
        await self._transport.verify(self._backend)
        receipt = await self._transport.unary(
            "Execution",
            "GetRunRequest",
            _wire.request("Execution", "GetRunRequest", request_id=self.id),
        )
        if receipt.request_id != self.id:
            raise ProtocolError("request receipt identity differs")
        try:
            _identity(receipt.cycle_id)
        except ValueError as error:
            raise ProtocolError("request receipt cycle identity is invalid") from error
        if self._cycle_id is not None and receipt.cycle_id != self._cycle_id:
            raise ProtocolError("request receipt belongs to another cycle")
        status = _wire.status(receipt)
        self._cycle_id = receipt.cycle_id
        return status

    async def wait(
        self, *, timeout: float | None = None, poll_interval: float = 0.2
    ) -> RunStatus:
        """Wait for a terminal receipt; local cancellation never stops remote work."""
        _interval(poll_interval)
        if timeout is not None:
            _interval(timeout)
        async with asyncio.timeout(timeout):
            async for status in self.updates(poll_interval=poll_interval):
                if status.terminal:
                    return status
        raise ProtocolError("request updates ended before terminal status")

    async def updates(self, *, poll_interval: float = 0.2) -> AsyncIterator[RunStatus]:
        _interval(poll_interval)
        previous = None
        while True:
            status = await self.status()
            if status != previous:
                yield status
                previous = status
            if status.terminal:
                return
            await asyncio.sleep(poll_interval)


# === Public Functions === #

# === Private Functions === #


def _interval(value: float) -> None:
    if not math.isfinite(value) or value <= 0:
        raise ValueError("interval/timeout must be positive and finite")


def _identity(value: str) -> None:
    try:
        identity = UUID(value)
    except (ValueError, AttributeError) as error:
        raise ValueError("a registered UUID is required") from error
    if str(identity) != value:
        raise ValueError("a canonical registered UUID is required")


# === Entry Point === #
