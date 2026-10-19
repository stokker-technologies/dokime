# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Execution controls over one authenticated client connection."""

# === Imports === #

# Standard Library Imports
import asyncio
import unicodedata
from collections.abc import AsyncGenerator, Awaitable, Callable, Mapping, Sequence
from contextlib import aclosing
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

from google.protobuf.json_format import MessageToDict

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from .. import _wire, prompts
from .._transport import Transport, _cursor
from ..errors import ExecutionSubmissionUncertain, ProtocolError, RpcError, RunRejected
from ..exports import export_evidence
from ..files import FileIdentity, FileUpdate, follow_file
from ..history import Checkpoint, Download
from ..models import (
    Backend,
    CommandAction,
    ExecutionCommandReceipt,
    Prompt,
    Record,
    ResultSummary,
    Sample,
    Value,
)
from .listeners import read_records, read_samples
from .request import RunRequest, _identity, _interval

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

    __slots__ = ("_backend", "_id", "_transport")

    def __init__(self, execution_id: str, transport: Transport, backend: Backend) -> None:
        _identity(execution_id)
        self._id = execution_id
        self._transport = transport
        self._backend = backend

    @property
    def id(self) -> str:
        return self._id

    async def export(self, destination: str | Path, *, overwrite: bool = False) -> Download:
        """Download This Execution's Sealed Evidence, Including From An Active Cycle."""
        return await export_evidence(self._transport, self._backend, "Execution", self.id, destination, overwrite=overwrite)

    async def snapshot(self) -> dict[str, Any]:
        """An owned current-runtime snapshot, verified against the original epoch."""
        detail = await self._snapshot()
        return MessageToDict(detail, preserving_proto_field_name=True)

    async def _prompt_scope(self) -> prompts.Scope:
        detail = await self._snapshot()
        try:
            return prompts.Scope(self._transport, self._backend, detail.cycle_id, self.id)
        except ValueError as error:
            raise ProtocolError("execution has an invalid cycle identity") from error

    async def pending_prompts(self) -> AsyncGenerator[Prompt]:
        """List only this execution's pending prompts through the native prompt owner."""
        scope = await self._prompt_scope()
        async with aclosing(scope.pending()) as pending:
            async for prompt in pending:
                yield prompt

    async def prompt(self, prompt_id: str) -> dict[str, Any]:
        """Inspect an admitted prompt and any response, requiring this execution's identity."""
        _identity(prompt_id)
        scope = await self._prompt_scope()
        view = await scope.read(prompt_id)
        return MessageToDict(view, preserving_proto_field_name=True)

    async def respond(self, prompt_id: str, *, action: str, values: Mapping[str, Value] | None = None, accepted: bool = True) -> None:
        """Answer once. After an ambiguous error inspect prompt(); never retry blindly."""
        _identity(prompt_id)
        scope = await self._prompt_scope()
        await scope.respond(prompt_id, action=action, values=values, accepted=accepted)

    async def _snapshot(self) -> Message:
        await self._transport.verify(self._backend, live=True)
        detail = await self._transport.unary(
            "Execution",
            "GetExecution",
            _wire.request("Execution", "GetExecution", execution_id=self.id),
        )
        if not detail.HasField("summary") or detail.summary.execution_id != self.id:
            raise ProtocolError("execution snapshot identity differs")
        _cursor(self._backend, detail)
        return detail

    async def checkpoint(self) -> Checkpoint | None:
        """Inspect this execution's saved seal; absence never certifies completion."""
        await self._transport.verify(self._backend)
        try:
            point = await self._transport.unary(
                "History",
                "GetExecutionCheckpoint",
                _wire.request("History", "GetExecutionCheckpoint", execution_identifier=self.id),
            )
        except RpcError as error:
            if error.code == "NOT_FOUND":
                return None
            raise
        if point.execution_identifier != self.id:
            raise ProtocolError("saved checkpoint belongs to another execution")
        return Checkpoint._from_wire(point)

    async def _document_at(self, point: Checkpoint) -> Message:
        if point.execution_id != self.id:
            raise ValueError("checkpoint belongs to another execution")
        await self._transport.verify(self._backend)
        reply = await self._transport.unary(
            "History",
            "GetCheckpointDocument",
            _wire.request("History", "GetCheckpointDocument", checkpoint=point._wire()),
        )
        if (
            not reply.HasField("checkpoint")
            or not _wire.equal(reply.checkpoint, point._wire())
            or not reply.acquisition_finished
            or not reply.HasField("document")
            or not reply.document.HasField("execution")
        ):
            raise ProtocolError("execution outcome differs from the persisted seal")
        saved = reply.document.execution
        if (
            saved.execution_id != self.id
            or not saved.HasField("result")
            or _wire.enum_name(saved.status)
            not in (
                "invalidated",
                "passed",
                "failed",
                "blocked",
                "cancelled",
                "interrupted",
                "skipped",
            )
        ):
            raise ProtocolError("sealed execution outcome is invalid or not terminal")
        _wire.result(saved.result)
        return reply

    async def wait(self, *, timeout: float | None = None, poll_interval: float | None = None) -> ExecutionOutcome:
        """Wait natively for a saved outcome; cancellation leaves remote work running."""
        interval = 0.2 if poll_interval is None else poll_interval
        _interval(interval)
        if timeout is not None:
            _interval(timeout)
        async with asyncio.timeout(timeout):
            while True:
                point = await self.checkpoint()
                if point is not None:
                    reply = await self._document_at(point)
                    break
                await self._snapshot()
                await asyncio.sleep(interval)
        saved = reply.document.execution
        return ExecutionOutcome(
            checkpoint=Checkpoint._from_wire(reply.checkpoint),
            status=_wire.enum_name(saved.status),
            result=_wire.result(saved.result),
        )

    async def start(
        self,
        *,
        request_key: str,
        mode: Literal["automatic", "interactive"] = "automatic",
    ) -> RunRequest:
        """Submit once; retain this identity, key and mode on an uncertain outcome."""
        if mode not in ("automatic", "interactive"):
            raise ValueError("execution mode must be automatic or interactive")
        _key(request_key)
        await self._transport.verify(self._backend, live=True)
        try:
            reply = await self._transport.unary(
                "Execution",
                "RunExecution",
                _wire.request(
                    "Execution",
                    "RunExecution",
                    execution_id=self.id,
                    idempotency_key=request_key,
                    mode={mode.title(): {}},
                ),
            )
        except RpcError as error:
            if error.submission_uncertain:
                raise ExecutionSubmissionUncertain(self.id, request_key, mode) from error
            raise
        admission = _admission(self._backend, reply)
        return RunRequest(self._transport, self._backend, admission.command_id)

    async def follow_file(
        self,
        relative_path: str,
        destination: str | Path,
        *,
        identity: FileIdentity | None = None,
        max_bytes: int = 64 * 1024,
    ) -> AsyncGenerator[FileUpdate]:
        """Follow A Registered File; Supply Its Identity To Resume A Committed Prefix"""
        async with aclosing(
            follow_file(
                self._transport,
                self._backend,
                self.id,
                relative_path,
                Path(destination),
                identity=identity,
                max_bytes=max_bytes,
            )
        ) as reader:
            async for update in reader:
                yield update

    async def records(
        self,
        *,
        record_types: Sequence[str] = (),
        record_ids: Sequence[str] = (),
        follow: bool = False,
        poll_interval: float | None = None,
    ) -> AsyncGenerator[Record]:
        """Replay this execution's live journal, optionally following through sealing.

        Replay starts at zero, so concurrent start and subscription cannot lose
        early publications. Saved storage positions are separate from live cursors.
        """
        await self._snapshot()
        interval = 0.2 if poll_interval is None else poll_interval
        _interval(interval)
        completion = self.wait(poll_interval=interval) if follow else None
        ids = frozenset(record_ids)
        async with aclosing(
            read_records(
                self._transport,
                self._backend,
                _wire.nested("Evidence", "ListRecords", "selection.scope", execution_id=self.id),
                record_types,
                completion,
            )
        ) as reader:
            async for page in reader:
                for item in page.records:
                    if not ids or item.record_id in ids:
                        yield _wire.record(item)

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
                async with aclosing(self.records(follow=True, poll_interval=poll_interval)) as reader:
                    async for item in reader:
                        await on_record(item)

        async def samples() -> None:
            if on_sample is not None:
                async with aclosing(self.samples(follow=True, poll_interval=poll_interval)) as reader:
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
    ) -> AsyncGenerator[Sample]:
        """Replay raw samples; native polling follows through the saved seal.

        Without follow, read until caught up. Original-epoch validation rejects
        backend replacement instead of resuming raw cursors in another process.
        """
        await self._snapshot()
        interval = 0.2 if poll_interval is None else poll_interval
        _interval(interval)

        async def complete() -> bool:
            return not follow or await self.checkpoint() is not None

        async for page in read_samples(
            self._transport,
            self._backend,
            _wire.nested("Evidence", "ListRecords", "selection.scope", execution_id=self.id),
            streams,
            page_size,
            interval,
            complete,
        ):
            for item in page.samples:
                yield _wire.sample(item)

    async def next(self, *, request_id: str, command_id: str) -> ExecutionCommandReceipt:
        return await self._command(request_id, command_id, CommandAction.NEXT)

    async def run_step(self, step_id: str, *, request_id: str, command_id: str) -> ExecutionCommandReceipt:
        return await self._command(request_id, command_id, CommandAction.RUN_STEP, step_id)

    async def finish(self, *, request_id: str, command_id: str) -> ExecutionCommandReceipt:
        """Submit normal finalization; an accepted receipt is not completion."""
        return await self._command(request_id, command_id, CommandAction.FINISH)

    async def _command(
        self,
        request_id: str,
        command_id: str,
        action: CommandAction,
        step_id: str | None = None,
    ) -> ExecutionCommandReceipt:
        _identity(request_id)
        _identity(command_id)
        if step_id is not None:
            _identity(step_id)
        await self._transport.verify(self._backend, live=True)
        request = _wire.request(
            "Execution",
            "SubmitExecutionCommand",
            request_id=request_id,
            command_id=command_id,
            execution_id=self.id,
            action={
                CommandAction.NEXT: {"Next": {}},
                CommandAction.RUN_STEP: {"RunStep": {}},
                CommandAction.FINISH: {"Finish": {}},
            }[action],
            step_id=step_id,
        )
        receipt = await self._transport.unary("Execution", "SubmitExecutionCommand", request)
        _command_receipt(self.id, request_id, command_id, receipt)
        if not _wire.equal(receipt.action, request.action) or receipt.step_id != request.step_id:
            raise ProtocolError("command receipt differs from submitted action")
        return _wire.command_receipt(receipt)

    async def command_receipt(self, request_id: str, command_id: str) -> ExecutionCommandReceipt:
        """Read published progress without repeating a device action."""
        _identity(request_id)
        _identity(command_id)
        await self._transport.verify(self._backend)
        receipt = await self._transport.unary(
            "Execution",
            "GetExecutionCommand",
            _wire.request(
                "Execution",
                "GetExecutionCommand",
                request_id=request_id,
                command_id=command_id,
            ),
        )
        _command_receipt(self.id, request_id, command_id, receipt)
        return _wire.command_receipt(receipt)

    async def _control(self, method: str) -> None:
        await self._transport.verify(self._backend, live=True)
        reply = await self._transport.unary(
            "Execution",
            method,
            _wire.request("Execution", method, execution_id=self.id),
        )
        _cursor(self._backend, reply)

    async def pause(self) -> None:
        await self._control("PauseExecution")

    async def resume(self) -> None:
        await self._control("ResumeExecution")

    async def stop(self) -> None:
        """Request cooperative stop; receipt progress establishes completion."""
        await self._control("StopExecution")


# === Public Functions === #

# === Private Functions === #


def _admission(backend: Backend, reply: Message) -> Message:
    if not reply.HasField("outcome"):
        raise ProtocolError("execution admission has no outcome")
    kind = _wire.variant(reply.outcome)
    outcome = getattr(reply.outcome, kind).field_0
    if kind == "Rejected":
        raise RunRejected(outcome.code, outcome.message)
    if kind != "Accepted":
        raise ProtocolError("execution admission has an unknown outcome")
    try:
        _identity(outcome.command_id)
    except ValueError as error:
        raise ProtocolError("invalid admission identity") from error
    _cursor(backend, outcome, "cursor")
    return outcome


def _key(key: str) -> None:
    if not key.strip() or len(key.encode("utf-8")) > 256 or any(unicodedata.category(c) == "Cc" for c in key):
        raise ValueError("request key must contain 1–256 bytes without control characters")


def _command_receipt(
    execution_id: str,
    request_id: str,
    command_id: str,
    receipt: Message,
) -> None:
    if (receipt.execution_id, receipt.request_id, receipt.command_id) != (
        execution_id,
        request_id,
        command_id,
    ):
        raise RpcError("DATA_LOSS", "command receipt identity differs")
    if _wire.enum_name(receipt.action) == "run_step":
        if not receipt.HasField("step_id"):
            raise ProtocolError("command receipt has no step identity")
        try:
            _identity(receipt.step_id)
        except ValueError as error:
            raise ProtocolError("command receipt step identity is invalid") from error
    elif _wire.enum_name(receipt.action) not in ("next", "finish") or receipt.HasField("step_id"):
        raise ProtocolError("command receipt action is invalid")
    if receipt.HasField("attempt_id"):
        try:
            _identity(receipt.attempt_id)
        except ValueError as error:
            raise ProtocolError("command receipt attempt identity is invalid") from error
    _wire.command_receipt(receipt)


# === Entry Point === #
