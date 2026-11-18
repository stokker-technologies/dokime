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
from collections.abc import Callable, Mapping, Sequence
from datetime import datetime
from types import MappingProxyType
from typing import TYPE_CHECKING, ClassVar, TypeVar, cast
from uuid import uuid4

# Third Party Imports
from google.protobuf.message import Message
from google.protobuf.timestamp_pb2 import Timestamp
from harmos import FileMetadata, PageRequest, plugin

from .._generated import shared

# Local Imports
from .._generated.plugin_types import (
    EffectWritable,
)
from .._generated.shared import (
    CapturedRecordSelection,
    Card,
    EvidenceStatisticsResponse,
    ExecutionProgressValue,
    RecordsListResponse,
    TraceabilityEntry,
)
from .._wire.host.v1.content.artifacts_pb2 import ArtifactsCreateResponse
from .._wire.model.v1.evidence_pb2 import ArtifactScope, Window
from .._wire.model.v1.metadata_pb2 import Metadata

if TYPE_CHECKING:
    from .._plugin import Plugin

from .. import operations
from .._authoring import PromptDeclaration, Record, _Template, prompt
from .._payloads import Payload, message_type, pack, unpack
from .._transfers import upload_bytes
from .._values import Scalar, encode
from ._captures import CaptureMode, CaptureResult, Environment, FileSource, Severity, Termination
from ._documents import Section

# === Public Exports === #


# === Constants === #

# === Types === #

# === Classes === #


class Context(operations.Context):
    def __init__(
        self,
        transport: plugin.Context,
        owner: "Plugin",
        parameters: Mapping[str, Scalar],
        template: _Template,
        *,
        termination: int | None = None,
    ) -> None:
        super().__init__(transport, owner)
        self.parameters: Mapping[str, Scalar] = MappingProxyType(dict(parameters))
        self._execution = transport.scope()
        self.termination = Termination(termination) if termination is not None else None
        self._template = template

    @property
    def execution(self) -> str:
        return self._execution

    async def prompt(self, text: str | PromptDeclaration | shared.Prompt, *, choices: Sequence[str] = ()) -> str:
        if self._template is None:
            raise ValueError("Create Prompt: Call From a Template Operation")
        if len(choices) != len(set(choices)):
            raise ValueError("Create Prompt: Supply Unique Action Labels")
        identity = f"{self._template.identifier}.question"
        if isinstance(text, PromptDeclaration):
            identity = self._resolve(text)
            assert self._template is not None
            definition = self._template._prompts[id(text)]
        else:
            definition = prompt(text, choices=choices).value
            definition.metadata.identifier = identity
        response = await self._host.prompts.create(
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
    ) -> int:
        kind = self._resolve(declaration)
        row = pack(declaration.row, payload)
        response = await self._host.evidence.records.append(
            self.execution, kind, row.SerializeToString(), _timestamp(await self.now()), request_id=request_id
        )
        return response.ordinal

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
        return await self._host.evidence.statistics(
            self._resolve(stream), [], Window(), [], field, self._resolve(step) if step is not None else ""
        )

    def _resolve(self, declaration: object) -> str:
        if self._template is None:
            raise ValueError("Resolve Declaration: Call From a Template Operation")
        return self._template.resolve(declaration)

    async def progress(self, completed: int, total: int, *, label: str = "", request_id: str | None = None) -> None:
        current = await self._host.executions.progress.get()
        await self._host.executions.progress.set(
            ExecutionProgressValue(completed=completed, total=total, label=label), current.revision, request_id=request_id
        )

    async def log(self, text: str, *, severity: Severity = Severity.INFO, request_id: str | None = None) -> None:
        await self._host.evidence.logs.append(
            self.execution, cast(shared.Severity, severity.value), text, _timestamp(await self.now()), request_id=request_id
        )

    async def traceability(
        self, identifier: str, value: Scalar, *, label: str = "", source: str = "", unit: str = "", request_id: str | None = None
    ) -> None:
        entry = TraceabilityEntry(
            metadata={"identifier": identifier, "label": label or identifier}, value=encode(value), source=source, unit=unit
        )
        current = await self._host.evidence.traceability.get(entry=identifier)
        await self._host.evidence.traceability.set(entry, current.revision, identifier, request_id=request_id)

    async def document(self, document: str, part: Section | Card, *, section: str = "", request_id: str | None = None) -> None:
        await self._host.documents.append_parts(
            self.execution,
            document,
            section,
            subsection=part.message if isinstance(part, Section) else None,
            card=part if isinstance(part, Card) else None,
            request_id=request_id,
        )

    async def artifact(
        self, name: str, data: bytes, *, mime: str = "application/octet-stream", request_id: str | None = None
    ) -> ArtifactsCreateResponse:
        identity = request_id if request_id is not None else str(uuid4())
        file = await upload_bytes(
            self._host.files.uploads,
            data,
            metadata=FileMetadata(scope=f"execution/{self.execution}", media_type=mime, expected_length=len(data)),
            request_id=identity,
        )
        return await self._host.content.artifacts.create(
            file,
            name,
            mime,
            scope=ArtifactScope(execution_id=self.execution),
            metadata=Metadata(identifier=name, label=name),
            request_id=f"{identity}/attach",
        )

    async def start_capture(self, source: FileSource, output: str, mode: CaptureMode) -> int:
        return await self._host.files.captures.start(source._wire(), output, mode._wire())

    async def stop_capture(self, capture_id: int) -> CaptureResult:
        return CaptureResult._from_wire(await self._host.files.captures.stop(capture_id))

    async def environment(self) -> Environment:
        value = await self._host.backend.environment()
        return Environment(value.backend_pid, value.in_step)

    async def prepare_effect(
        self, resource: str, inverse_operation: str, inverse: Message, *, description: str = "", request_id: str | None = None
    ) -> str:
        response = await self._host.effects.create(
            resource, inverse_operation, inverse.SerializeToString(), description, request_id=request_id
        )
        return response.name

    async def confirm_effect(
        self, ticket: str, *, at: datetime | None = None, restored: bool = False, request_id: str | None = None
    ) -> None:
        current = await self._host.effects.get(ticket)
        value = EffectWritable(restored=restored, applied_at=_timestamp(at) if at is not None else None)
        await self._host.effects.set(ticket, value, current.revision, request_id=request_id)

    async def capture(self, source: FileSource, output: str) -> CaptureResult:
        return CaptureResult._from_wire(await self._host.files.capture(source._wire(), output))


class RecordsQuery[T: Payload]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]

    def __init__(self, context: Context, definition: Record[T], step: str) -> None:
        self._context = context
        self._definition = definition
        self._step = step

    async def read(self, *, limit: int = 0) -> RecordsListResponse:
        return await self._context._host.evidence.records.list(
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


def _timestamp(value: datetime) -> Timestamp:
    timestamp = Timestamp()
    timestamp.FromDatetime(value)
    return timestamp


# === Entry Point === #
