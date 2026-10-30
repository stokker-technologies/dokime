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
from collections.abc import AsyncGenerator, Generator, Mapping, Sequence
from contextlib import aclosing
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal
from uuid import uuid4

# Third Party Imports
from google.protobuf.json_format import MessageToDict
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
from ..planning import TestCase
from .listeners import read_records, read_samples, validate_patterns
from .request import _identity, _interval

# Type Checking Imports

# === Public Exports === #

# === Constants === #

CHECKPOINT_BYTES = 4096
REQUEST_POLL_MS = 200
SAMPLE_PAGE_ITEMS = 1000
VALUE_DEPTH = 64

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class ExecutionOutcome:
    """The terminal status and result frozen in one persisted execution seal.

    Interrupted and cancelled outcomes remain distinct from successful execution.
    Evidence readers drain independently through this durable outcome.
    """

    checkpoint: Checkpoint
    status: str
    result: ResultSummary

    @property
    def exit_code(self) -> int:
        if self.status in ("interrupted", "cancelled"):
            return 130
        return 0 if self.status == "passed" and self.result.status == "passed" else 1


class Execution:
    """A Fresh Local Attempt; Append It To A Cycle Before Using Remote Controls."""

    __slots__ = ("_backend", "_case", "_cycle_id", "_id", "_outcome", "_transport")

    def __init__(self, test_case: TestCase) -> None:
        if not isinstance(test_case, TestCase):
            raise TypeError("Execution requires a materialized TestCase")
        self._id = str(uuid4())
        self._case: TestCase | None = test_case
        self._transport = test_case._transport
        self._backend = test_case._backend
        self._cycle_id: str | None = None
        self._outcome: ExecutionOutcome | None = None

    @classmethod
    def _bind(cls, execution_id: str, transport: Transport, backend: Backend) -> "Execution":
        _identity(execution_id)
        result = cls.__new__(cls)
        result._id = execution_id
        result._case = None
        result._transport = transport
        result._backend = backend
        # Existing Identities Are Already Registered; Snapshot Resolves Their Cycle.
        result._cycle_id = ""
        result._outcome = None
        return result

    def _require_bound(self) -> None:
        if self._cycle_id is None:
            raise ValueError("append this execution to a cycle before using remote controls")

    @property
    def id(self) -> str:
        return self._id

    @property
    def outcome(self) -> ExecutionOutcome:
        """The Saved Outcome After Awaiting This Handle Or Draining An Iterator."""
        if self._outcome is None:
            raise RuntimeError("execution outcome is not available; await the execution first")
        return self._outcome

    def __await__(self) -> Generator[Any, None, ExecutionOutcome]:
        return self.wait().__await__()

    async def export(self, destination: str | Path, *, overwrite: bool = False) -> Download:
        """Download This Execution's Sealed Evidence, Including From An Active Cycle."""
        self._require_bound()
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
        self._require_bound()
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
        self._require_bound()
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
        return Checkpoint._from_wire(point, CHECKPOINT_BYTES)

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
        interval = REQUEST_POLL_MS / 1000 if poll_interval is None else poll_interval
        _interval(interval)
        if timeout is not None:
            _interval(timeout)
        if self._outcome is not None:
            return self._outcome
        async with asyncio.timeout(timeout):
            while True:
                point = await self.checkpoint()
                if point is not None:
                    reply = await self._document_at(point)
                    break
                await self._snapshot()
                await asyncio.sleep(interval)
        saved = reply.document.execution
        self._outcome = ExecutionOutcome(
            checkpoint=Checkpoint._from_wire(reply.checkpoint, CHECKPOINT_BYTES),
            status=_wire.enum_name(saved.status),
            result=_wire.result(saved.result),
        )
        return self._outcome

    async def start(
        self,
        *,
        mode: Literal["automatic", "interactive"] = "automatic",
    ) -> None:
        """Admit This Attempt Once; Retry The Same Execution And Mode After An Uncertain Reply."""
        self._require_bound()
        if mode not in ("automatic", "interactive"):
            raise ValueError("execution mode must be automatic or interactive")
        await self._transport.verify(self._backend, live=True)
        try:
            reply = await self._transport.unary(
                "Execution",
                "StartExecution",
                _wire.request(
                    "Execution",
                    "StartExecution",
                    execution_id=self.id,
                    mode={mode.title(): {}},
                ),
            )
        except RpcError as error:
            if error.submission_uncertain:
                raise ExecutionSubmissionUncertain(self.id, mode) from error
            raise
        _admission(self._backend, reply)

    async def follow_file(
        self,
        relative_path: str,
        destination: str | Path,
        *,
        identity: FileIdentity | None = None,
        max_bytes: int | None = None,
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
        *types: str,
        record_ids: Sequence[str] = (),
        poll_interval: float | None = None,
    ) -> AsyncGenerator[Record]:
        """Replay Matching Records From Zero And Follow Through The Saved Seal."""
        validate_patterns(types)
        await self._snapshot()
        interval = REQUEST_POLL_MS / 1000 if poll_interval is None else poll_interval
        _interval(interval)
        completion = self.wait(poll_interval=interval)
        ids = frozenset(record_ids)
        async with aclosing(
            read_records(
                self._transport,
                self._backend,
                _wire.nested("Evidence", "ListRecords", "selection.scope", execution_id=self.id),
                types,
                completion,
            )
        ) as reader:
            async for page in reader:
                for item in page.records:
                    if not ids or item.record_id in ids:
                        yield _wire.record(item, VALUE_DEPTH)

    async def samples(
        self,
        *streams: str,
        page_size: int | None = None,
    ) -> AsyncGenerator[Sample]:
        """Replay Matching Samples And Follow Durable Changes Through The Seal."""
        validate_patterns(streams)
        await self._snapshot()
        page_size = SAMPLE_PAGE_ITEMS if page_size is None else page_size
        async with aclosing(
            read_samples(
                self._transport,
                self._backend,
                _wire.nested("Evidence", "ListRecords", "selection.scope", execution_id=self.id),
                streams,
                page_size,
                True,
            )
        ) as reader:
            async for page in reader:
                for item in page.samples:
                    yield _wire.sample(item, VALUE_DEPTH)
        await self.wait()

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
