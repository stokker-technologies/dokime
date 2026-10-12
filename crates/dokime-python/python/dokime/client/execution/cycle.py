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
from collections.abc import AsyncGenerator, AsyncIterator, Mapping, Sequence
import math
from pathlib import Path
from typing import Any

# Third Party Imports
from google.protobuf.json_format import MessageToDict

# Local Imports
from .. import _wire
from .._generated.dokime.v1 import evidence_pb2 as evidence
from .._generated.dokime.v1 import execution_pb2 as execution
from .._generated.dokime.v1 import telemetry_pb2 as telemetry
from .._generated.dokime.v1 import prompts_pb2 as prompts
from .._generated.dokime.v1 import value_pb2 as value
from .._transport import Transport, _error
from dokime._native.client import ClientError, TestCycleHandle as NativeCycleHandle
from ..errors import (
    ProtocolError,
    RpcError,
    SubmissionUncertain,
)
from ..models import (
    Backend,
    Prompt,
    Record,
    ResultSummary,
    Sample,
    Value,
)
from ..history import Download, download_report
from .handle import TestExecutionHandle
from .request import RunRequest

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Cycle:
    """A live cycle; evidence includes all attempts unless execution_id narrows it."""

    def __init__(
        self,
        native: NativeCycleHandle,
        transport: Transport,
        backend: Backend,
        executions: tuple[TestExecutionHandle, ...],
    ) -> None:
        self._native = native
        self._transport = transport
        self._backend = backend
        self._executions = executions

    @property
    def id(self) -> str:
        return self._native.id

    @property
    def executions(self) -> tuple[TestExecutionHandle, ...]:
        """Ordered membership observed at binding; handles share the native client.

        Bind the cycle again to observe later membership changes. Reading or
        binding this collection never starts execution or creates device resources.
        """
        return self._executions

    @classmethod
    async def _bind(
        cls, transport: Transport, cycle_id: str, backend: Backend
    ) -> "Cycle":
        native = await transport.cycle(cycle_id, backend)
        try:
            members = await native.executions()
        except ClientError as error:
            raise _error(error) from error
        executions = tuple(
            TestExecutionHandle(member, transport, backend) for member in members
        )
        return cls(native, transport, backend, executions)

    def _check_request(self, request: "RunRequest | None") -> None:
        if request is not None and (
            request.cycle_id != self.id
            or request._transport is not self._transport
            or request._backend != self._backend
        ):
            raise ValueError("request belongs to another cycle or connection")

    async def run(self, *, request_key: str) -> "RunRequest":
        """Submit or resolve the same cycle/key without creating another cycle."""
        try:
            admission = execution.Admission.FromString(
                await self._native.run(request_key)
            )
        except ClientError as error:
            mapped = _error(error)
            if isinstance(mapped, RpcError):
                if mapped.submission_uncertain:
                    raise SubmissionUncertain(self.id, request_key) from error
                if mapped.code == "DATA_LOSS":
                    raise ProtocolError(str(mapped)) from error
            raise mapped from error
        return RunRequest(self._transport, self._backend, admission.command_id, self.id)

    async def result(self) -> ResultSummary:
        """Read current cycle result counters, not a particular request's result."""
        cycle = await self._snapshot()
        return _wire.result(cycle.result)

    async def document(self) -> dict[str, Any]:
        """Current cycle metadata in protobuf JSON with original field names."""
        cycle = await self._snapshot()
        return MessageToDict(cycle, preserving_proto_field_name=True)

    async def _snapshot(self) -> execution.CycleSummary:
        try:
            return execution.CycleSummary.FromString(await self._native.snapshot())
        except ClientError as error:
            raise _error(error) from error

    async def download_report(
        self,
        destination: str | Path,
        *,
        identifier: str = "cycle.summary",
    ) -> Download:
        """Stream cycle HTML and verify the transfer before publishing the file."""
        await self._transport.verify(self._backend, live=True)
        return await download_report(
            self._transport, self.id, Path(destination), identifier
        )

    async def pending_prompts(self) -> AsyncIterator[Prompt]:
        """List this cycle's pending interactions; never auto-accept a prompt."""
        try:
            reader = await self._native.pending_prompts()
            try:
                while (payload := await reader.message()) is not None:
                    item = prompts.PromptInstance.FromString(payload)
                    yield Prompt(
                        item.prompt_id,
                        item.definition.prompt_type,
                        item.definition.title,
                        item.definition.message
                        if item.definition.HasField("message")
                        else None,
                        tuple(action.id for action in item.definition.actions),
                        tuple(
                            field.metadata.identifier
                            for field in item.definition.fields
                        ),
                        _wire.fields(item.values),
                    )
            finally:
                await reader.close()
        except ClientError as error:
            mapped = _error(error)
            if isinstance(mapped, RpcError) and mapped.code == "DATA_LOSS":
                raise ProtocolError(str(mapped)) from error
            raise mapped from error

    async def respond(
        self,
        prompt_id: str,
        *,
        action: str,
        values: Mapping[str, Value] | None = None,
        accepted: bool = True,
    ) -> None:
        """Respond once. After an ambiguous error, inspect the prompt; do not retry blindly."""
        encoded = value.ValueObject(
            fields={k: _wire.encode(v) for k, v in (values or {}).items()}
        )
        try:
            await self._native.respond_prompt(
                prompt_id, action, encoded.SerializeToString(), accepted
            )
        except ClientError as error:
            mapped = _error(error)
            if isinstance(mapped, RpcError):
                if mapped.code == "INVALID_ARGUMENT":
                    raise ValueError(str(mapped)) from error
                if mapped.code == "DATA_LOSS":
                    raise ProtocolError(str(mapped)) from error
            raise mapped from error

    async def records(
        self,
        *,
        record_types: Sequence[str] = (),
        record_ids: Sequence[str] = (),
        execution_id: str | None = None,
        until: "RunRequest | None" = None,
    ) -> AsyncGenerator[Record, None]:
        """Replay captured records, follow live, and drain after terminal receipt.

        Event types filter on the server; occurrence IDs filter locally. Empty
        filters collect every captured record in scope, including records from
        before subscription. Unscoped backend records are outside this scope.
        Without until, read a fixed current frontier. No cross-restart replay.
        """
        self._check_request(until)
        request = await until._handle() if until is not None else None
        ids = frozenset(record_ids)
        try:
            reader = await self._native.records(
                record_types=list(record_types),
                execution_id=execution_id,
                until=request,
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
            mapped = _error(error)
            if isinstance(mapped, RpcError) and mapped.code == "DATA_LOSS":
                raise ProtocolError(str(mapped)) from error
            raise mapped from error

    async def samples(
        self,
        *,
        streams: Sequence[str] = (),
        execution_id: str | None = None,
        until: "RunRequest | None" = None,
        poll_interval: float = 0.2,
        page_size: int = 500,
    ) -> AsyncGenerator[Sample, None]:
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
        request = await until._handle() if until is not None else None
        try:
            reader = await self._native.samples(
                streams=list(streams),
                execution_id=execution_id,
                until=request,
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
            mapped = _error(error)
            if isinstance(mapped, RpcError) and mapped.code == "DATA_LOSS":
                raise ProtocolError(str(mapped)) from error
            raise mapped from error


# === Public Functions === #

# === Private Functions === #


def _interval(value: float) -> None:
    if not math.isfinite(value) or value <= 0:
        raise ValueError("interval/timeout must be positive and finite")


# === Entry Point === #
