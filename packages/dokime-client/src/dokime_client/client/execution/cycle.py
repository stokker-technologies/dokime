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
import asyncio
import hashlib
import json
import os
from collections.abc import AsyncGenerator, Mapping, Sequence
from contextlib import aclosing
from pathlib import Path
from types import TracebackType
from typing import TYPE_CHECKING, Any, Self

# Third Party Imports
from google.protobuf.json_format import MessageToDict
from google.protobuf.message import Message

# Local Imports
from .. import _wire, prompts
from .._transport import Transport, _cursor
from ..errors import (
    ProtocolError,
)
from ..exports import export_evidence
from ..history import Download, download_report
from ..models import (
    Backend,
    Prompt,
    Record,
    ResultSummary,
    Sample,
    SaveManifest,
    Value,
)
from .handle import Execution
from .listeners import read_records, read_samples
from .request import _identity

# Type Checking Imports
if TYPE_CHECKING:
    from ..client import DokimeClient

# === Public Exports === #

# === Constants === #

MAXIMUM_SAMPLE_PAGE_ITEMS = 5000
PAGE_ITEMS = 100
SAMPLE_PAGE_ITEMS = 1000
VALUE_DEPTH = 64

# === Types === #

# === Classes === #


class Cycle:
    """A live cycle; evidence includes all attempts unless execution_id narrows it."""

    def __init__(
        self,
        cycle_id: str,
        transport: Transport,
        backend: Backend,
        executions: tuple[Execution, ...],
        *,
        client: "DokimeClient | None" = None,
    ) -> None:
        _identity(cycle_id)
        self._id = cycle_id
        self._transport = transport
        self._backend = backend
        self._executions = executions
        self._client = client

    async def __aenter__(self) -> Self:
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        # Shield Cleanup Even From Repeated Cancellation (Including Ctrl+C).
        cleanup = asyncio.create_task(self._close())
        cancellation: asyncio.CancelledError | None = None
        try:
            while not cleanup.done():
                try:
                    await asyncio.shield(cleanup)
                except asyncio.CancelledError as error:
                    cancellation = cancellation or error
            cleanup.result()
        except BaseException as error:
            original = exc if exc is not None else cancellation
            if original is not None:
                raise BaseExceptionGroup("cycle body and cleanup failed", [original, error]) from None
            raise
        if exc is None and cancellation is not None:
            raise cancellation

    async def _close(self) -> None:
        # SealCycle Stops Owned Work And Closes Admission Before Draining Seals.
        await self.seal()
        results = await asyncio.gather(*(execution.wait() for execution in self._executions), return_exceptions=True)
        failures = [result for result in results if isinstance(result, BaseException)]
        if failures:
            raise BaseExceptionGroup("cycle outcome drain failed", failures)

    async def start(self, template: str, **parameters: Value) -> Execution | tuple[Execution, ...]:
        """Materialize, Append And Start; Expansion Returns All Handles In Case Order.

        Starts are admitted in order to the server's execution queue. A single
        case returns one handle; multiple cases return a tuple, including every
        attempt. Partial/uncertain admission remains owned by this cycle.
        """
        if self._client is None:
            raise ValueError("start requires a cycle obtained from DokimeClient")
        selected = await self._client.template(template)
        cases = await selected.materialize(parameters=parameters)
        executions = tuple(Execution(case) for case in cases)
        await self.append(executions)
        for execution in executions:
            await execution.start()
        return executions[0] if len(executions) == 1 else executions

    async def save(self, directory: str | Path, report: str | None = None) -> SaveManifest:
        """Save Sealed Evidence And A Digest Manifest Into A Fresh Directory.

        Existing output files are refused. File paths in the manifest are
        relative to directory; manifest.json does not hash itself.
        """
        destination = Path(directory)
        destination.mkdir(parents=True, exist_ok=True)
        names = [f"execution-{execution.id}.zip" for execution in self._executions]
        names.append("cycle.zip")
        if report is not None:
            names.append("report.html")
        for name in [*names, "manifest.json", "manifest.json.part"]:
            path = destination / name
            if path.exists() or path.is_symlink():
                raise FileExistsError(path)

        for execution in self._executions:
            await execution.export(destination / f"execution-{execution.id}.zip")
        await self.export(destination / "cycle.zip")
        if report is not None:
            await self.download_report(destination / "report.html", identifier=report)

        manifest: SaveManifest = {"cycle_id": self.id, "files": []}
        for name in names:
            path = destination / name
            with path.open("rb") as content:
                digest = hashlib.file_digest(content, "sha256").hexdigest()
            manifest["files"].append({"path": name, "size": path.stat().st_size, "sha256": digest})
        partial = destination / "manifest.json.part"
        content = partial.open("x", encoding="utf-8")
        try:
            with content:
                content.write(json.dumps(manifest, indent=2) + "\n")
                content.flush()
                os.fsync(content.fileno())
            os.link(partial, destination / "manifest.json")
        finally:
            partial.unlink()
        return manifest

    @property
    def id(self) -> str:
        return self._id

    async def append(self, executions: Sequence[Execution]) -> None:
        """Atomically Register These Exact Attempts In Order; Retrying Cannot Duplicate Them."""
        executions = tuple(executions)
        if not executions:
            raise ValueError("append requires at least one execution")
        identifiers = [execution.id for execution in executions]
        if len(set(identifiers)) != len(identifiers):
            raise ValueError("append repeats an execution identity")
        bindings = []
        for execution in executions:
            case = execution._case
            same_connection = execution._transport is self._transport and execution._backend == self._backend
            if case is None or not same_connection:
                raise ValueError("append requires materialized executions from this connection")
            if execution._cycle_id not in (None, self.id):
                raise ValueError("execution already belongs to another cycle")
            bindings.append({"execution_id": execution.id, "case_id": case.id})

        request = _wire.request(
            "Execution",
            "AppendExecutions",
            cycle_id=self.id,
            executions=bindings,
        )
        await self._transport.verify(self._backend, live=True)
        reply = await self._transport.unary("Execution", "AppendExecutions", request)
        _cursor(self._backend, reply)
        if list(reply.execution_ids) != identifiers:
            raise ProtocolError("append reply changed execution identities or order")

        members = {execution.id: execution for execution in self._executions}
        for execution in executions:
            execution._cycle_id = self.id
            members.setdefault(execution.id, execution)
        self._executions = tuple(members.values())

    async def export(self, destination: str | Path, *, overwrite: bool = False) -> Download:
        """Download Every Member Of This Durably Sealed Cycle."""
        return await export_evidence(self._transport, self._backend, "Cycle", self.id, destination, overwrite=overwrite)

    @property
    def executions(self) -> tuple[Execution, ...]:
        """Ordered membership observed at binding; handles share the connection.

        Bind the cycle again to observe later membership changes. Reading or
        binding this collection never starts execution or creates device resources.
        """
        return self._executions

    @classmethod
    async def _bind(cls, transport: Transport, cycle_id: str, backend: Backend, *, client: "DokimeClient | None" = None) -> "Cycle":
        result = cls(cycle_id, transport, backend, (), client=client)
        await result._snapshot()
        request = _wire.request("Execution", "ListExecutions", cycle_id=cycle_id, limit=PAGE_ITEMS)
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
                members.append(Execution._bind(row.execution_id, transport, backend))
            if not page.HasField("next_after_execution_id"):
                break
            if not page.executions or page.next_after_execution_id != page.executions[-1].execution_id:
                raise ProtocolError("execution membership continuation is invalid")
            request.after_execution_id = page.next_after_execution_id
            _wire.copy(request.cursor, page.cursor)
        result._executions = tuple(members)
        return result

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
    ) -> AsyncGenerator[Record]:
        """Replay captured records through the current frontier.

        Event types filter on the server; occurrence IDs filter locally. Empty
        filters collect every captured record in scope, including records from
        before subscription. Unscoped backend records are outside this scope.
        Read a fixed current frontier. No cross-restart replay.
        """
        await self._snapshot()
        scope = _wire.nested(
            "Evidence",
            "ListRecords",
            "selection.scope",
            cycle_id=self.id,
            execution_id=execution_id,
        )
        ids = frozenset(record_ids)

        async with aclosing(
            read_records(
                self._transport,
                self._backend,
                scope,
                record_types,
                None,
            )
        ) as reader:
            async for page in reader:
                for item in page.records:
                    if not ids or item.record_id in ids:
                        yield _wire.record(item, VALUE_DEPTH)

    async def samples(
        self,
        *,
        streams: Sequence[str] = (),
        execution_id: str | None = None,
        page_size: int | None = None,
    ) -> AsyncGenerator[Sample]:
        """Replay Saved Telemetry Through One Fixed Durable Frontier."""
        page_size = SAMPLE_PAGE_ITEMS if page_size is None else page_size
        if type(page_size) is not int or not 1 <= page_size <= MAXIMUM_SAMPLE_PAGE_ITEMS:
            raise ValueError("page_size exceeds the configured maximum")
        await self._snapshot()
        scope = _wire.nested(
            "Evidence",
            "ListRecords",
            "selection.scope",
            cycle_id=self.id,
            execution_id=execution_id,
        )

        async for page in read_samples(
            self._transport,
            self._backend,
            scope,
            streams,
            page_size,
            False,
        ):
            for item in page.samples:
                yield _wire.sample(item, VALUE_DEPTH)


# === Public Functions === #

# === Private Functions === #


# === Entry Point === #
