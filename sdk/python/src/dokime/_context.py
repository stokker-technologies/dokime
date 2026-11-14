# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Typed Host Resources Scoped to One Library Invocation."""

# === Imports === #

from collections.abc import Sequence
from typing import Any, Generic, TypeVar, cast
from uuid import uuid4

from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from google.protobuf.timestamp_pb2 import Timestamp
from harmos import FileMetadata, OperationPolicy, PageRequest, Receipt, SettingValue
from harmos.plugin import Context as InvocationContext
from harmos.plugin import Operation

from ._generated.host import HostRoot
from ._generated.plugin_types import (
    BackendEnvironmentResponse,
    EffectsCreateResponse,
    EffectsSetResponse,
    EffectWritable,
    EvidenceRecordsAppendResponse,
    FilesCaptureMode,
    FilesCapturesStopResponse,
)
from ._generated.shared import (
    EvidenceCapturedRecordSelection,
    EvidenceRecordsListResponse,
    EvidenceStatisticsResponse,
    ExecutionProgressValue,
    Prompt,
    TraceabilityEntry,
)
from ._wire.host.v1.content.artifacts_pb2 import ArtifactsCreateResponse
from ._wire.model.v1.documents_pb2 import Card, Section
from ._wire.model.v1.evidence_pb2 import ArtifactScope, Severity, Window
from ._wire.model.v1.files_pb2 import FileSource
from ._wire.model.v1.metadata_pb2 import Metadata
from .authoring import Record
from .values import Scalar, encode

# === Public Exports === #

__all__ = ["Context"]

# === Constants === #

# === Types === #

_RequestT = TypeVar("_RequestT", bound=Message)
_ResponseT = TypeVar("_ResponseT", bound=Message)

# === Classes === #


class Context:
    def __init__(
        self,
        transport: InvocationContext,
        parameters: dict[str, Scalar],
        operations: Sequence[Operation[Any, Any]],
        records: Sequence[Record] = (),
        *,
        termination: int | None = None,
        secrets: Sequence[str] = (),
    ) -> None:
        self._transport = transport
        self.host = HostRoot(transport)
        self.parameters = parameters
        self.execution = transport.scope()
        self.attempt = transport.attempt
        self.termination = termination
        self._records = {item.identifier: item for item in records}
        self._secrets = frozenset(secrets)

    def operation[Q: Message, R: Message](self, operation: Operation[Q, R]) -> "BoundOperation[Q, R]":
        return BoundOperation(self._transport, operation)

    async def sleep(self, duration: float | Duration) -> None:
        if not isinstance(duration, Duration):
            nanoseconds = int(duration * 1_000_000_000)
            duration = Duration(seconds=nanoseconds // 1_000_000_000, nanos=nanoseconds % 1_000_000_000)
        await self.host.time.sleep(duration)

    async def now(self) -> Timestamp:
        return await self.host.time.get()

    async def cancelled(self) -> str:
        return await self._transport.cancelled()

    async def prompt(self, text: str, *, choices: Sequence[str] = ()) -> str:
        definition = Prompt(metadata={"identifier": "question", "label": text}, message=text)
        for index, choice in enumerate(choices or ("OK",)):
            definition.actions.add(identifier=str(index), label=choice)
        response = await self.host.prompts.create("question", "", {}, definition=definition)
        if not response.accepted:
            raise ValueError("Create Prompt: Operator Answer Was Not Accepted")
        actions = {action.identifier: action.label for action in definition.actions}
        return actions.get(response.action, response.action)

    async def record(
        self, kind: str, payload: Message | None = None, *, request_id: str | None = None, **fields: object
    ) -> EvidenceRecordsAppendResponse:
        definition = self._records.get(kind)
        if definition is None:
            raise ValueError(f"Append Record: Undeclared Record Kind {kind}")
        row = payload if payload is not None else definition.row(**fields)
        if not isinstance(row, definition.row):
            raise TypeError(f"Append Record: Invalid Payload for {kind}")
        return await self.host.evidence.records.append(
            self.execution, kind, row.SerializeToString(), await self.now(), request_id=request_id
        )

    def records(self, kind: str, *, step: str = "") -> "RecordsQuery":
        if kind not in self._records:
            raise ValueError(f"Read Records: Undeclared Record Kind {kind}")
        return RecordsQuery(self, self._records[kind], step)

    async def statistics(self, stream: str, field: str = "", *, step: str = "") -> EvidenceStatisticsResponse:
        return await self.host.evidence.statistics(stream, [], Window(), [], field, step)

    async def secret(self, identifier: str) -> bytes:
        if identifier not in self._secrets:
            raise ValueError(f"Read Secret: Undeclared Secret {identifier}")
        return (await self.host.secrets.get(identifier)).secret

    async def settings(self) -> dict[str, SettingValue]:
        response = await self.host.plugins.settings.list()
        return dict(response.value.values)

    async def progress(self, completed: int, total: int, *, label: str = "", request_id: str | None = None) -> None:
        current = await self.host.executions.progress.get()
        await self.host.executions.progress.set(
            ExecutionProgressValue(completed=completed, total=total, label=label), current.revision, request_id=request_id
        )

    async def log(self, text: str, *, severity: int = Severity.SEVERITY_INFO, request_id: str | None = None) -> None:
        await self.host.evidence.logs.append(self.execution, cast(Severity, severity), text, await self.now(), request_id=request_id)

    async def traceability(
        self, identifier: str, value: Scalar, *, label: str = "", source: str = "", unit: str = "", request_id: str | None = None
    ) -> None:
        entry = TraceabilityEntry(
            metadata={"identifier": identifier, "label": label or identifier}, value=encode(value), source=source, unit=unit
        )
        current = await self.host.evidence.traceability.get(entry=identifier)
        await self.host.evidence.traceability.set(entry, current.revision, identifier, request_id=request_id)

    async def document(self, document: str, part: Section | Card, *, section: str = "", request_id: str | None = None) -> Receipt:
        return await self.host.documents.append_parts(
            self.execution,
            document,
            section,
            subsection=part if isinstance(part, Section) else None,
            card=part if isinstance(part, Card) else None,
            request_id=request_id,
        )

    async def artifact(
        self, name: str, data: bytes, *, mime: str = "application/octet-stream", request_id: str | None = None
    ) -> ArtifactsCreateResponse:
        from .transfers import upload_bytes

        identity = request_id if request_id is not None else str(uuid4())
        file = await upload_bytes(
            self.host.files.uploads,
            data,
            metadata=FileMetadata(scope=self.execution, media_type=mime, expected_length=len(data)),
            request_id=identity,
        )
        return await self.host.content.artifacts.create(
            file,
            name,
            mime,
            scope=ArtifactScope(execution_id=self.execution),
            metadata=Metadata(identifier=name, label=name),
            request_id=f"{identity}/attach",
        )

    async def start_capture(self, source: FileSource, output: str, mode: FilesCaptureMode) -> int:
        return await self.host.files.captures.start(source, output, mode)

    async def stop_capture(self, capture_id: int) -> FilesCapturesStopResponse:
        return await self.host.files.captures.stop(capture_id)

    async def environment(self) -> BackendEnvironmentResponse:
        return await self.host.backend.environment()

    async def prepare_effect(
        self, resource: str, inverse_operation: str, inverse: Message, *, description: str = "", request_id: str | None = None
    ) -> EffectsCreateResponse:
        return await self.host.effects.create(resource, inverse_operation, inverse.SerializeToString(), description, request_id=request_id)

    async def confirm_effect(
        self, ticket: str, *, at: Timestamp | None = None, restored: bool = False, request_id: str | None = None
    ) -> EffectsSetResponse:
        current = await self.host.effects.get(ticket)
        value = EffectWritable(restored=restored, applied_at=at)
        return await self.host.effects.set(ticket, value, current.revision, request_id=request_id)


class BoundOperation(Generic[_RequestT, _ResponseT]):
    def __init__(self, transport: InvocationContext, operation: Operation[_RequestT, _ResponseT]) -> None:
        self._transport = transport
        self._operation = operation

    async def __call__(self, request: _RequestT, *, policy: OperationPolicy | None = None) -> _ResponseT:
        return await self._transport.call(self._operation, request, policy=policy)


class RecordsQuery:
    def __init__(self, context: Context, definition: Record, step: str) -> None:
        self._context = context
        self._definition = definition
        self._step = step

    async def read(self, *, limit: int = 0) -> EvidenceRecordsListResponse:
        return await self._context.host.evidence.records.list(
            self._definition.identifier, [], Window(), self._step, EvidenceCapturedRecordSelection(), page=PageRequest(page_size=limit)
        )

    async def count(self) -> int:
        return (await self.read()).count

    async def last(self) -> Message | None:
        response = await self.read()
        return self._definition.row.FromString(response.last.payload) if response.HasField("last") else None


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
