# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Connected execution controls over the original native client handle."""

# === Imports === #

# Standard Library Imports
import asyncio
from collections.abc import AsyncGenerator, Awaitable, Callable, Sequence
from contextlib import aclosing
from dataclasses import dataclass
from typing import Any, Literal

# Third Party Imports
from google.protobuf.json_format import MessageToDict

# Local Imports
from dokime._native.client import (
    ClientError,
    TestExecutionHandle as NativeExecutionHandle,
)
from .. import _wire
from .._generated.dokime.v1 import execution_pb2 as execution
from .._generated.dokime.v1 import evidence_pb2 as evidence
from .._generated.dokime.v1 import history_pb2 as history
from .._generated.dokime.v1 import telemetry_pb2 as telemetry
from .._transport import Transport, _error
from ..errors import ExecutionSubmissionUncertain, RpcError
from ..history import Checkpoint
from .._generated.dokime.v1 import checkpoint_pb2 as checkpoint
from ..models import (
    Backend,
    CommandAction,
    ExecutionCommandReceipt,
    Record,
    ResultSummary,
    Sample,
)
from .request import RunRequest

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class ExecutionOutcome:
    """The terminal status and result frozen in one persisted execution seal.

    Interrupted and cancelled outcomes remain distinct from successful execution.
    Listener callback completion is separate from this durable outcome.
    """

    checkpoint: Checkpoint
    status: str
    result: ResultSummary


class TestExecutionHandle:
    """An existing execution on one native connection; construction starts no work."""

    __slots__ = ("_native", "_transport", "_backend")

    def __init__(
        self, native: NativeExecutionHandle, transport: Transport, backend: Backend
    ) -> None:
        self._native = native
        self._transport = transport
        self._backend = backend

    @property
    def id(self) -> str:
        return self._native.id

    async def snapshot(self) -> dict[str, Any]:
        """An owned current-runtime snapshot, verified against the original epoch."""
        try:
            detail = execution.ExecutionDetail.FromString(await self._native.snapshot())
        except ClientError as error:
            raise _error(error) from error
        return MessageToDict(detail, preserving_proto_field_name=True)

    async def checkpoint(self) -> Checkpoint | None:
        """Inspect this execution's saved seal; absence never certifies completion."""
        try:
            point = checkpoint.ExecutionCheckpoint.FromString(
                await self._native.checkpoint()
            )
        except ClientError as error:
            mapped = _error(error)
            if isinstance(mapped, RpcError) and mapped.code == "NOT_FOUND":
                return None
            raise mapped from error
        return Checkpoint._from_wire(point)

    async def wait(
        self, *, timeout: float | None = None, poll_interval: float | None = None
    ) -> ExecutionOutcome:
        """Wait natively for a saved outcome; cancellation leaves remote work running."""
        try:
            reply = history.CheckpointDocument.FromString(
                await self._native.wait(timeout=timeout, poll_interval=poll_interval)
            )
        except ClientError as error:
            raise _error(error) from error
        saved = reply.document.execution
        return ExecutionOutcome(
            checkpoint=Checkpoint._from_wire(reply.checkpoint),
            status=execution.ExecutionStatus.Name(saved.status)
            .removeprefix("EXECUTION_STATUS_")
            .lower(),
            result=_wire.result(saved.result),
        )

    async def start(
        self,
        *,
        request_key: str,
        mode: Literal["automatic", "interactive"] = "automatic",
    ) -> RunRequest:
        """Submit once; retain this identity, key and mode on an uncertain outcome."""
        try:
            admission = execution.Admission.FromString(
                await self._native.start(request_key, mode=mode)
            )
        except ClientError as error:
            mapped = _error(error)
            if isinstance(mapped, RpcError) and mapped.submission_uncertain:
                raise ExecutionSubmissionUncertain(
                    self.id, request_key, mode
                ) from error
            raise mapped from error
        return RunRequest(self._transport, self._backend, admission.command_id)

    async def records(
        self,
        *,
        record_types: Sequence[str] = (),
        record_ids: Sequence[str] = (),
        follow: bool = False,
        poll_interval: float | None = None,
    ) -> AsyncGenerator[Record, None]:
        """Replay this execution's live journal, optionally following through sealing.

        Replay starts at zero, so concurrent start and subscription cannot lose
        early publications. Saved storage positions are separate from live cursors.
        """
        ids = frozenset(record_ids)
        try:
            reader = await self._native.records(
                record_types=list(record_types),
                follow=follow,
                poll_interval=poll_interval,
            )
            try:
                while (payload := await reader.message()) is not None:
                    page = evidence.RecordPage.FromString(payload)
                    for item in page.records:
                        if not ids or item.record_id in ids:
                            yield _wire.record(item)
            finally:
                await reader.close()
        except ClientError as error:
            raise _error(error) from error

    async def run(
        self,
        *,
        request_key: str,
        mode: Literal["automatic", "interactive"] = "automatic",
        on_record: Callable[[Record], Awaitable[None]] | None = None,
        on_sample: Callable[[Sample], Awaitable[None]] | None = None,
        timeout: float | None = None,
        poll_interval: float | None = None,
    ) -> ExecutionOutcome:
        """Start, wait for the saved outcome, and drain supplied async callbacks.

        A callback failure or cancellation closes readers and leaves remote work
        running. Resolve uncertain admission with the same key and mode.
        """

        async def records() -> None:
            if on_record is not None:
                async with aclosing(
                    self.records(follow=True, poll_interval=poll_interval)
                ) as reader:
                    async for item in reader:
                        await on_record(item)

        async def samples() -> None:
            if on_sample is not None:
                async with aclosing(
                    self.samples(follow=True, poll_interval=poll_interval)
                ) as reader:
                    async for item in reader:
                        await on_sample(item)

        await self.start(request_key=request_key, mode=mode)
        async with asyncio.TaskGroup() as tasks:
            if on_record is not None:
                tasks.create_task(records())
            if on_sample is not None:
                tasks.create_task(samples())
            outcome = await self.wait(timeout=timeout, poll_interval=poll_interval)
        return outcome

    async def samples(
        self,
        *,
        streams: Sequence[str] = (),
        follow: bool = False,
        page_size: int = 500,
        poll_interval: float | None = None,
    ) -> AsyncGenerator[Sample, None]:
        """Replay raw samples; native polling follows through the saved seal.

        Without follow, read until caught up. Original-epoch validation rejects
        backend replacement instead of resuming raw cursors in another process.
        """
        try:
            reader = await self._native.samples(
                streams=list(streams),
                follow=follow,
                page_size=page_size,
                poll_interval=poll_interval,
            )
            try:
                while (payload := await reader.message()) is not None:
                    page = telemetry.TelemetrySamplesReply.FromString(payload)
                    for item in page.samples:
                        yield _wire.sample(item)
            finally:
                await reader.close()
        except ClientError as error:
            raise _error(error) from error

    async def next(
        self, *, request_id: str, command_id: str
    ) -> ExecutionCommandReceipt:
        return await self._command(request_id, command_id, CommandAction.NEXT)

    async def run_step(
        self, step_id: str, *, request_id: str, command_id: str
    ) -> ExecutionCommandReceipt:
        return await self._command(
            request_id, command_id, CommandAction.RUN_STEP, step_id
        )

    async def finish(
        self, *, request_id: str, command_id: str
    ) -> ExecutionCommandReceipt:
        """Submit normal finalization; an accepted receipt is not completion."""
        return await self._command(request_id, command_id, CommandAction.FINISH)

    async def _command(
        self,
        request_id: str,
        command_id: str,
        action: CommandAction,
        step_id: str | None = None,
    ) -> ExecutionCommandReceipt:
        try:
            payload = await self._native.command(
                request_id, command_id, action, step_id=step_id
            )
        except ClientError as error:
            raise _error(error) from error
        return _wire.command_receipt(
            execution.ExecutionCommandReceipt.FromString(payload)
        )

    async def command_receipt(
        self, request_id: str, command_id: str
    ) -> ExecutionCommandReceipt:
        """Read native published progress without repeating a device action."""
        try:
            payload = await self._native.command_receipt(request_id, command_id)
        except ClientError as error:
            raise _error(error) from error
        return _wire.command_receipt(
            execution.ExecutionCommandReceipt.FromString(payload)
        )

    async def pause(self) -> None:
        try:
            await self._native.pause()
        except ClientError as error:
            raise _error(error) from error

    async def resume(self) -> None:
        try:
            await self._native.resume()
        except ClientError as error:
            raise _error(error) from error

    async def stop(self) -> None:
        """Request cooperative stop; receipt progress establishes completion."""
        try:
            await self._native.stop()
        except ClientError as error:
            raise _error(error) from error


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
