# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Typed Host Resources Scoped to One Library Invocation."""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable, Sequence
from typing import TYPE_CHECKING, ClassVar, TypeVar, cast
from uuid import uuid4

# Third Party Imports
from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from google.protobuf.timestamp_pb2 import Timestamp
from harmos import FileMetadata, OperationPolicy, PageRequest, Receipt, SettingValue, plugin
from harmos._invocation import FailureError

# Local Imports
from ._generated.host import HostRoot
from ._generated.plugin_types import (
    BackendEnvironmentResponse,
    CaptureMode,
    EffectsCreateResponse,
    EffectsSetResponse,
    EffectWritable,
    RecordsAppendResponse,
    host,
)
from ._generated.shared import (
    CapturedRecordSelection,
    EvidenceStatisticsResponse,
    ExecutionProgressValue,
    RecordsListResponse,
    TraceabilityEntry,
)
from ._wire.host.v1.content.artifacts_pb2 import ArtifactsCreateResponse
from ._wire.model.v1.documents_pb2 import Card, Section
from ._wire.model.v1.evidence_pb2 import ArtifactScope, Severity, Window
from ._wire.model.v1.files_pb2 import FileSource
from ._wire.model.v1.metadata_pb2 import Metadata

if TYPE_CHECKING:
    from . import operations

from .authoring import PromptDeclaration, Record, Template, prompt
from .payloads import Payload, message_type, pack, unpack
from .transfers import upload_bytes
from .values import Scalar, encode

# === Public Exports === #

__all__ = ["Context"]

# === Constants === #

# === Types === #

# === Classes === #


class Context:
    def __init__(
        self,
        transport: plugin.Context,
        parameters: dict[str, Scalar] | None = None,
        template: Template | None = None,
        *,
        termination: int | None = None,
        secrets: Sequence[str] = (),
    ) -> None:
        self._transport = transport
        self.host = HostRoot(transport)
        self.parameters = parameters or {}
        try:
            self.execution = transport.scope()
        except FailureError as error:
            if error.value.code != "scope_required":
                raise
            self.execution = ""
        self.attempt = transport.attempt
        self.termination = termination
        self._template = template
        self._secrets = frozenset(secrets)

    def operation[Q: Payload, R: Payload](self, operation: "operations.Operation[Q, R]") -> "BoundOperation[Q, R]":
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

    async def prompt(self, text: str | PromptDeclaration, *, choices: Sequence[str] = ()) -> str:
        identity = "question"
        if isinstance(text, PromptDeclaration):
            identity = self._resolve(text)
            assert self._template is not None
            definition = self._template._prompts[id(text)]
        else:
            definition = prompt(text, choices=choices).value
            definition.metadata.identifier = identity
        response = await self.host.prompts.create(
            identity, identity if isinstance(text, PromptDeclaration) else "", {}, definition=definition
        )
        if not response.accepted:
            raise ValueError("Create Prompt: Choose an Accepted Operator Answer")
        actions = {action.identifier: action.label for action in definition.actions}
        if response.action not in actions:
            raise ValueError("Read Prompt Answer: Choose One of the Declared Actions")
        return actions[response.action]

    async def record[T: Payload](
        self,
        declaration: Record[T],
        payload: T,
        *,
        request_id: str | None = None,
    ) -> RecordsAppendResponse:
        kind = self._resolve(declaration)
        row = pack(declaration.row, payload)
        return await self.host.evidence.records.append(
            self.execution, kind, row.SerializeToString(), await self.now(), request_id=request_id
        )

    def records[T: Payload](self, declaration: Record[T], *, step: Callable[..., object] | None = None) -> "RecordsQuery[T]":
        self._resolve(declaration)
        return RecordsQuery(self, declaration, self._resolve(step) if step is not None else "")

    async def statistics(
        self,
        stream: object,
        field: str = "",
        *,
        step: Callable[..., object] | None = None,
    ) -> EvidenceStatisticsResponse:
        return await self.host.evidence.statistics(
            self._resolve(stream), [], Window(), [], field, self._resolve(step) if step is not None else ""
        )

    def _resolve(self, declaration: object) -> str:
        if self._template is None:
            raise ValueError("Resolve Declaration: Call From a Template Operation")
        return self._template.resolve(declaration)

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
        identity = request_id if request_id is not None else str(uuid4())
        file = await upload_bytes(
            self.host.files.uploads,
            data,
            metadata=FileMetadata(scope=f"execution/{self.execution}", media_type=mime, expected_length=len(data)),
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

    async def start_capture(self, source: FileSource, output: str, mode: CaptureMode) -> int:
        return await self.host.files.captures.start(source, output, mode)

    async def stop_capture(self, capture_id: int) -> host.files.CapturesStopResponse:
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


class BoundOperation[RequestT: Payload, ResponseT: Payload]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]

    def __init__(self, transport: plugin.Context, operation: "operations.Operation[RequestT, ResponseT]") -> None:
        self._transport = transport
        self._operation = operation

    async def __call__(self, request: RequestT, *, policy: OperationPolicy | None = None) -> ResponseT:
        wire = self._operation.wire()
        response = await self._transport.call(wire, pack(self._operation.request, request), policy=policy)
        return unpack(self._operation.response, response)


class RecordsQuery[T: Payload]:
    def __init__(self, context: Context, definition: Record[T], step: str) -> None:
        self._context = context
        self._definition = definition
        self._step = step

    async def read(self, *, limit: int = 0) -> RecordsListResponse:
        return await self._context.host.evidence.records.list(
            self._context._resolve(self._definition), [], Window(), self._step, CapturedRecordSelection(), page=PageRequest(page_size=limit)
        )

    async def count(self) -> int:
        return (await self.read()).count

    async def last(self) -> T:
        response = await self.read()
        if not response.HasField("last"):
            raise ValueError("Read Records: Append a Record Before Reading the Last Value")
        message = message_type(self._definition.row).FromString(response.last.payload)
        return unpack(self._definition.row, message)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
