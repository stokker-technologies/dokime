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
from collections import ChainMap
from collections.abc import Callable, Mapping, Sequence
from datetime import datetime
from types import MappingProxyType
from typing import TYPE_CHECKING

# Third Party Imports
from google.protobuf.timestamp_pb2 import Timestamp
from harmos import plugin

# Local Imports
from .. import operations
from .._authoring import PromptDeclaration, Record, _Template, prompt
from .._evidence import RecordsQuery
from .._generated import shared
from .._generated.shared import (
    Card,
    ExecutionProgressValue,
    TraceabilityEntry,
)
from .._parameters import Parameter
from .._payloads import Payload, pack
from .._transfers import publish_artifact
from .._transport import refused
from .._values import Artifact, Note, Scalar, Statistics, Termination, decode, encode
from .._wire.model.v1.evidence_pb2 import ArtifactScope, Window
from ._captures import CaptureMode, CaptureResult, Environment, FileSource
from ._documents import Document, Section

# Type Checking Imports
if TYPE_CHECKING:
    from .._plugin import Plugin

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
        self._parameters: Mapping[str, Scalar] = MappingProxyType(dict(parameters))
        self._execution = transport.scope()
        self.termination = Termination(termination) if termination is not None else None
        self._template = template
        self._stream_identities = ChainMap(template._identifiers, owner._streams)

    @property
    def execution(self) -> str:
        return self._execution

    def parameter[T](self, declaration: Parameter[T]) -> T:
        if not any(declaration is member for member in self._template._methods.values()):
            raise ValueError("Read Parameter: Select a Parameter Declared by This Template")
        name = declaration._message.metadata.identifier
        return declaration._read(self._parameters[name])

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
        return RecordsQuery._for_context(self, declaration, step=self._resolve(step) if step is not None else "")

    async def statistics(
        self,
        stream: object,
        field: str = "",
        *,
        step: Callable[..., object] | None = None,
    ) -> Statistics:
        response = await self._host.evidence.statistics(
            self._resolve(stream), [], Window(), [], field, self._resolve(step) if step is not None else ""
        )
        value = response.statistics
        fields = ("min", "max", "mean", "median", "p05", "p95", "p99", "first", "last", "sum")
        values = {name: decode(getattr(value, name)) if value.HasField(name) else None for name in fields}
        return Statistics(
            count=value.count,
            min=values["min"],
            max=values["max"],
            mean=values["mean"],
            stddev=value.stddev,
            median=values["median"],
            p05=values["p05"],
            p95=values["p95"],
            p99=values["p99"],
            first=values["first"],
            last=values["last"],
            sum=values["sum"],
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

    async def traceability(
        self, identifier: str, value: Scalar, *, label: str = "", source: str = "", unit: str = "", request_id: str | None = None
    ) -> None:
        entry = TraceabilityEntry(
            metadata={"identifier": identifier, "label": label or identifier}, value=encode(value), source=source, unit=unit
        )
        current = await self._host.evidence.traceability.get(entry=identifier)
        await self._host.evidence.traceability.set(entry, current.revision, identifier, request_id=request_id)

    async def document(self, document: str | Document, part: Section | Card, *, section: str = "", request_id: str | None = None) -> None:
        if isinstance(document, Document):
            identity = self._resolve(document)
        else:
            documents = {item.message.metadata.identifier for item in self._template.documents}
            identity = document if document in documents else f"{self._template.identifier}.{document}"
            if identity not in documents:
                raise ValueError("Append Document: Select a Document Declared by This Template")
        await self._host.documents.append_parts(
            self.execution,
            identity,
            section,
            subsection=part.message if isinstance(part, Section) else None,
            card=part if isinstance(part, Card) else None,
            request_id=request_id,
        )

    async def note(self, text: str, *, request_id: str | None = None) -> Note:
        raise refused(
            "evidence.notes.create",
            "Plugin Note Publication Is Not Admitted by the Host Contract",
            "Admit Notes/Create for Plugin Callers and Regenerate the Host Bindings",
        )

    async def artifact(self, name: str, data: bytes, *, mime: str | None = None, request_id: str | None = None) -> Artifact:
        return await publish_artifact(self._host, ArtifactScope(execution_id=self.execution), name, data, mime=mime, request_id=request_id)

    async def start_capture(self, source: FileSource, output: str, mode: CaptureMode) -> int:
        return await self._host.files.captures.start(source._wire(), output, mode._wire())

    async def stop_capture(self, capture_id: int) -> CaptureResult:
        return CaptureResult._from_wire(await self._host.files.captures.stop(capture_id))

    async def environment(self) -> Environment:
        value = await self._host.backend.environment()
        return Environment(value.backend_pid, value.in_step)

    async def capture(self, source: FileSource, output: str) -> CaptureResult:
        return CaptureResult._from_wire(await self._host.files.capture(source._wire(), output))


# === Public Functions === #

# === Private Functions === #


def _timestamp(value: datetime) -> Timestamp:
    timestamp = Timestamp()
    timestamp.FromDatetime(value)
    return timestamp


# === Entry Point === #
