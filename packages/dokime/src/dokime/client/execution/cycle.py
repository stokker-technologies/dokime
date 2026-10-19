# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Cycle-owned control, interaction, evidence and report operations."""

# === Imports === #

# Standard Library Imports
import math
from collections.abc import AsyncGenerator, Mapping, Sequence
from contextlib import aclosing
from pathlib import Path
from typing import Any

from google.protobuf.json_format import MessageToDict

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from .. import _wire, prompts
from .._transport import Transport, _cursor
from ..errors import (
    ProtocolError,
    RpcError,
    SubmissionUncertain,
)
from ..exports import export_evidence
from ..history import Download, download_report
from ..models import (
    Backend,
    Prompt,
    Record,
    ResultSummary,
    Sample,
    Value,
)
from .handle import TestExecutionHandle, _admission, _key
from .listeners import read_records, read_samples
from .request import RunRequest, _identity

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Cycle:
    """A live cycle; evidence includes all attempts unless execution_id narrows it."""

    def __init__(
        self,
        cycle_id: str,
        transport: Transport,
        backend: Backend,
        executions: tuple[TestExecutionHandle, ...],
    ) -> None:
        _identity(cycle_id)
        self._id = cycle_id
        self._transport = transport
        self._backend = backend
        self._executions = executions

    @property
    def id(self) -> str:
        return self._id

    async def export(self, destination: str | Path, *, overwrite: bool = False) -> Download:
        """Download Every Member Of This Durably Sealed Cycle."""
        return await export_evidence(self._transport, self._backend, "Cycle", self.id, destination, overwrite=overwrite)

    @property
    def executions(self) -> tuple[TestExecutionHandle, ...]:
        """Ordered membership observed at binding; handles share the connection.

        Bind the cycle again to observe later membership changes. Reading or
        binding this collection never starts execution or creates device resources.
        """
        return self._executions

    @classmethod
    async def _bind(cls, transport: Transport, cycle_id: str, backend: Backend) -> "Cycle":
        result = cls(cycle_id, transport, backend, ())
        await result._snapshot()
        request = _wire.request("Execution", "ListExecutions", cycle_id=cycle_id, limit=100)
        members = []
        seen = set()
        while True:
            page = await transport.unary("Execution", "ListExecutions", request)
            _cursor(backend, page, "cursor")
            if request.HasField("cursor") and not _wire.equal(request.cursor, page.cursor) or len(page.executions) > request.limit:
                raise ProtocolError("execution membership changed cursor or exceeded limit")
            for row in page.executions:
                try:
                    _identity(row.execution_id)
                except ValueError as error:
                    raise ProtocolError("execution membership identity is invalid") from error
                if not row.case_id or row.ordinal != len(members) or row.execution_id in seen:
                    raise ProtocolError("execution membership identity or order differs")
                seen.add(row.execution_id)
                members.append(TestExecutionHandle(row.execution_id, transport, backend))
            if not page.HasField("next_after_execution_id"):
                break
            if not page.executions or page.next_after_execution_id != page.executions[-1].execution_id:
                raise ProtocolError("execution membership continuation is invalid")
            request.after_execution_id = page.next_after_execution_id
            _wire.copy(request.cursor, page.cursor)
        result._executions = tuple(members)
        return result

    def _check_request(self, request: "RunRequest | None") -> None:
        if request is not None and (
            request.cycle_id != self.id or request._transport is not self._transport or request._backend != self._backend
        ):
            raise ValueError("request belongs to another cycle or connection")

    async def run(self, *, request_key: str) -> "RunRequest":
        """Submit or resolve the same cycle/key without creating another cycle."""
        _key(request_key)
        await self._transport.verify(self._backend, live=True)
        try:
            reply = await self._transport.unary(
                "Execution",
                "RunCycle",
                _wire.request(
                    "Execution",
                    "RunCycle",
                    cycle_id=self.id,
                    idempotency_key=request_key,
                ),
            )
        except RpcError as error:
            if error.submission_uncertain:
                raise SubmissionUncertain(self.id, request_key) from error
            raise
        admission = _admission(self._backend, reply)
        return RunRequest(self._transport, self._backend, admission.command_id, self.id)

    async def schedule_rerun(self, case_id: str) -> None:
        """Append the case's next attempt as a new execution; nothing runs yet.

        Creation is not idempotent: each call that succeeds appends another
        attempt, and earlier attempts and their evidence stay as they are.
        """
        if not case_id:
            raise ValueError("case_id must not be empty")
        await self._transport.verify(self._backend, live=True)
        reply = await self._transport.unary(
            "Execution",
            "ScheduleCaseRerun",
            _wire.request("Execution", "ScheduleCaseRerun", cycle_id=self.id, case_id=case_id),
        )
        _cursor(self._backend, reply)

    async def run_case(self, case_id: str, *, request_key: str) -> "RunRequest":
        """Admit the case's queued attempt once under the caller-retained key."""
        if not case_id:
            raise ValueError("case_id must not be empty")
        _key(request_key)
        await self._transport.verify(self._backend, live=True)
        try:
            reply = await self._transport.unary(
                "Execution",
                "RunCase",
                _wire.request(
                    "Execution",
                    "RunCase",
                    cycle_id=self.id,
                    case_id=case_id,
                    idempotency_key=request_key,
                ),
            )
        except RpcError as error:
            if error.submission_uncertain:
                raise SubmissionUncertain(self.id, request_key) from error
            raise
        admission = _admission(self._backend, reply)
        return RunRequest(self._transport, self._backend, admission.command_id, self.id)

    async def result(self) -> ResultSummary:
        """Read current cycle result counters, not a particular request's result."""
        cycle = await self._snapshot()
        return _wire.result(cycle.result)

    async def seal(self) -> None:
        """Close admission, stop this cycle's work and await its durable seal.

        Completed executions keep their original seals. Unrun executions are
        cancelled. Repeating this request is safe, including after reconnecting
        to resolve a lost response; it never reruns device operations.
        """
        await self._transport.verify(self._backend, live=True)
        reply = await self._transport.unary(
            "Execution",
            "SealCycle",
            _wire.request("Execution", "SealCycle", cycle_id=self.id),
        )
        _cursor(self._backend, reply)

    async def document(self) -> dict[str, Any]:
        """Current cycle metadata in protobuf JSON with original field names."""
        cycle = await self._snapshot()
        return MessageToDict(cycle, preserving_proto_field_name=True)

    async def _snapshot(self) -> Message:
        await self._transport.verify(self._backend, live=True)
        snapshot = await self._transport.unary(
            "Execution",
            "GetCycle",
            _wire.request("Execution", "GetCycle", cycle_id=self.id),
        )
        if snapshot.cycle_id != self.id:
            raise ProtocolError("cycle snapshot identity differs")
        _cursor(self._backend, snapshot)
        return snapshot

    async def download_report(
        self,
        destination: str | Path,
        *,
        identifier: str = "cycle.summary",
    ) -> Download:
        """Stream cycle HTML and verify the transfer before publishing the file."""
        await self._transport.verify(self._backend, live=True)
        return await download_report(self._transport, self.id, Path(destination), identifier)

    async def pending_prompts(self) -> AsyncGenerator[Prompt]:
        """List this cycle's pending interactions; never auto-accept a prompt."""
        scope = prompts.Scope(self._transport, self._backend, self.id)
        async with aclosing(scope.pending()) as pending:
            async for prompt in pending:
                yield prompt

    async def _prompt(self, prompt_id: str):
        return await prompts.Scope(self._transport, self._backend, self.id).read(prompt_id)

    async def respond(
        self,
        prompt_id: str,
        *,
        action: str,
        values: Mapping[str, Value] | None = None,
        accepted: bool = True,
    ) -> None:
        """Respond once. After an ambiguous error, inspect the prompt; do not retry blindly."""
        scope = prompts.Scope(self._transport, self._backend, self.id)
        await scope.respond(prompt_id, action=action, values=values, accepted=accepted)

    async def records(
        self,
        *,
        record_types: Sequence[str] = (),
        record_ids: Sequence[str] = (),
        execution_id: str | None = None,
        until: "RunRequest | None" = None,
    ) -> AsyncGenerator[Record]:
        """Replay captured records, follow live, and drain after terminal receipt.

        Event types filter on the server; occurrence IDs filter locally. Empty
        filters collect every captured record in scope, including records from
        before subscription. Unscoped backend records are outside this scope.
        Without until, read a fixed current frontier. No cross-restart replay.
        """
        self._check_request(until)
        await self._snapshot()
        scope = _wire.nested(
            "Evidence",
            "ListRecords",
            "selection.scope",
            cycle_id=self.id,
            execution_id=execution_id,
        )
        ids = frozenset(record_ids)

        async def complete():
            assert until is not None
            status = await until.wait()
            if status.phase.value == "interrupted":
                raise ProtocolError("interrupted acquisition does not certify complete live evidence")

        async with aclosing(
            read_records(
                self._transport,
                self._backend,
                scope,
                record_types,
                complete() if until is not None else None,
            )
        ) as reader:
            async for page in reader:
                for item in page.records:
                    if not ids or item.record_id in ids:
                        yield _wire.record(item)

    async def samples(
        self,
        *,
        streams: Sequence[str] = (),
        execution_id: str | None = None,
        until: "RunRequest | None" = None,
        poll_interval: float = 0.2,
        page_size: int = 500,
    ) -> AsyncGenerator[Sample]:
        """Read raw telemetry from the beginning with lossless cursor polling.

        This wraps ListSamples, not a server-streaming raw-telemetry RPC.
        until=request drains after durable completion. Otherwise read until
        caught up; that is not an immutable snapshot of an actively writing run.
        Oversized replies fail explicitly; lower page_size and retry collection.
        """
        _interval(poll_interval)
        if not 1 <= page_size <= 5000:
            raise ValueError("page_size must be between 1 and 5000")
        self._check_request(until)
        await self._snapshot()
        scope = _wire.nested(
            "Evidence",
            "ListRecords",
            "selection.scope",
            cycle_id=self.id,
            execution_id=execution_id,
        )

        async def complete() -> bool:
            if until is None:
                return True
            status = await until.status()
            if status.phase.value == "interrupted":
                raise ProtocolError("interrupted acquisition does not certify complete live evidence")
            return status.terminal

        async for page in read_samples(
            self._transport,
            self._backend,
            scope,
            streams,
            page_size,
            poll_interval,
            complete,
        ):
            for item in page.samples:
                yield _wire.sample(item)


# === Public Functions === #

# === Private Functions === #


def _interval(value: float) -> None:
    if not math.isfinite(value) or value <= 0:
        raise ValueError("interval/timeout must be positive and finite")


# === Entry Point === #
