# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Typed Unary Host Calls Scoped To One Harmos Invocation."""

# === Imports === #

# Standard Library Imports
from collections.abc import Sequence
from typing import Any
from uuid import uuid4

# Third Party Imports
from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from google.protobuf.timestamp_pb2 import Timestamp
from harmos import GET_TIME, SLEEP, Operation
from harmos.v1 import types_pb2 as wire

# Local Imports
from .authoring import Record
from .v1 import message, message_type
from .values import decode, encode

# Type Checking Imports

# === Public Exports === #

__all__ = ["HOST_OPERATIONS", "Context"]

# === Constants === #

HOST_OPERATIONS = tuple(
    Operation(identifier, message_type(request), message_type(response), wire.VISIBILITY_PUBLIC)
    for identifier, request, response in (
        (
            "dokime.plugin.host.v1.evidence.Records/Append",
            "dokime.plugin.host.v1.evidence.RecordsAppendRequest",
            "dokime.plugin.host.v1.evidence.RecordsAppendResponse",
        ),
        (
            "dokime.plugin.host.v1.evidence.Records/List",
            "dokime.plugin.host.v1.evidence.RecordsListRequest",
            "dokime.plugin.host.v1.evidence.RecordsListResponse",
        ),
        (
            "dokime.plugin.host.v1.Evidence/Statistics",
            "dokime.plugin.host.v1.EvidenceStatisticsRequest",
            "dokime.plugin.host.v1.EvidenceStatisticsResponse",
        ),
        (
            "dokime.plugin.host.v1.executions.Progress/Get",
            "dokime.plugin.host.v1.executions.ProgressGetRequest",
            "dokime.plugin.host.v1.executions.ProgressGetResponse",
        ),
        (
            "dokime.plugin.host.v1.executions.Progress/Set",
            "dokime.plugin.host.v1.executions.ProgressSetRequest",
            "dokime.plugin.host.v1.executions.ProgressSetResponse",
        ),
        (
            "dokime.plugin.host.v1.evidence.Logs/Append",
            "dokime.plugin.host.v1.evidence.LogsAppendRequest",
            "dokime.plugin.host.v1.evidence.LogsAppendResponse",
        ),
        ("dokime.settings.read", "dokime.plugin.host.v1.ReadSettings", "dokime.plugin.host.v1.SavedSettings"),
        ("dokime.secrets.read", "dokime.plugin.host.v1.ReadSecret", "dokime.plugin.host.v1.Secret"),
        (
            "dokime.plugin.host.v1.Prompts/Create",
            "dokime.plugin.host.v1.PromptsCreateRequest",
            "dokime.plugin.host.v1.PromptsCreateResponse",
        ),
        (
            "dokime.plugin.host.v1.evidence.Traceability/Get",
            "dokime.plugin.host.v1.evidence.TraceabilityGetRequest",
            "dokime.plugin.host.v1.evidence.TraceabilityGetResponse",
        ),
        (
            "dokime.plugin.host.v1.evidence.Traceability/Set",
            "dokime.plugin.host.v1.evidence.TraceabilitySetRequest",
            "dokime.plugin.host.v1.evidence.TraceabilitySetResponse",
        ),
        (
            "dokime.plugin.host.v1.Documents/AppendParts",
            "dokime.plugin.host.v1.DocumentsAppendPartsRequest",
            "dokime.plugin.host.v1.DocumentsAppendPartsResponse",
        ),
        ("dokime.execution.artifact", "dokime.plugin.host.v1.content.PublishArtifact", "dokime.plugin.host.v1.content.ArtifactReceipt"),
        ("dokime.plugin.host.v1.Files/Capture", "dokime.plugin.host.v1.FilesCaptureRequest", "dokime.plugin.host.v1.FilesCaptureResponse"),
        (
            "dokime.plugin.host.v1.files.Captures/Start",
            "dokime.plugin.host.v1.files.CapturesStartRequest",
            "dokime.plugin.host.v1.files.CapturesStartResponse",
        ),
        (
            "dokime.plugin.host.v1.files.Captures/Stop",
            "dokime.plugin.host.v1.files.CapturesStopRequest",
            "dokime.plugin.host.v1.files.CapturesStopResponse",
        ),
        (
            "dokime.plugin.host.v1.Backend/Environment",
            "dokime.plugin.host.v1.BackendEnvironmentRequest",
            "dokime.plugin.host.v1.BackendEnvironmentResponse",
        ),
        (
            "dokime.plugin.host.v1.Effects/Create",
            "dokime.plugin.host.v1.EffectsCreateRequest",
            "dokime.plugin.host.v1.EffectsCreateResponse",
        ),
        ("dokime.plugin.host.v1.Effects/Get", "dokime.plugin.host.v1.EffectsGetRequest", "dokime.plugin.host.v1.EffectsGetResponse"),
        ("dokime.plugin.host.v1.Effects/Set", "dokime.plugin.host.v1.EffectsSetRequest", "dokime.plugin.host.v1.EffectsSetResponse"),
        (
            "dokime.plugin.host.v1.content.Artifacts/Create",
            "dokime.plugin.host.v1.content.ArtifactsCreateRequest",
            "dokime.plugin.host.v1.content.ArtifactsCreateResponse",
        ),
    )
) + (GET_TIME, SLEEP)

# === Types === #


# === Classes === #


class Context:
    def __init__(
        self,
        transport: Any,
        parameters: dict[str, Any],
        operations: Sequence[Operation],
        records: Sequence[Record] = (),
        *,
        termination: int | None = None,
    ) -> None:
        self._transport = transport
        self.parameters = parameters
        self.execution = transport.scope()
        self.attempt = transport.attempt
        self.termination = termination
        self._operations = tuple(operations)
        self._records = {item.identifier: item for item in records}

    def operation[Q: Message, R: Message](self, operation: Operation[Q, R]) -> "BoundOperation[Q, R]":
        return BoundOperation(self._transport, operation)

    async def _call(self, request: Message) -> Any:
        candidates = [item for item in self._operations if item.request.DESCRIPTOR.full_name == request.DESCRIPTOR.full_name]
        if len(candidates) != 1:
            raise ValueError(f"Call Host Operation: Name the declared operation for {request.DESCRIPTOR.full_name}")
        return await self.operation(candidates[0])(request)

    async def sleep(self, duration: float | Duration) -> None:
        if not isinstance(duration, Duration):
            milliseconds = int(duration * 1000)
            duration = Duration(seconds=milliseconds // 1000, nanos=(milliseconds % 1000) * 1_000_000)
        await self._transport.sleep(duration)

    async def now(self) -> Timestamp:
        return await self._transport.now()

    async def cancelled(self) -> str:
        return await self._transport.cancelled()

    async def prompt(self, text: str, *, choices: Sequence[str] = ()) -> str:
        definition = message("Prompt", metadata={"identifier": "question", "label": text}, message=text)
        for index, choice in enumerate(choices or ("OK",)):
            definition.actions.add(identifier=str(index), label=choice)
        response = await self._call(message("dokime.plugin.host.v1.PromptsCreateRequest", prompt="question", definition=definition))
        if not response.accepted:
            raise ValueError("Create Prompt: Operator Answer Was Not Accepted")
        actions = {action.identifier: action.label for action in definition.actions}
        return actions.get(response.action, response.action)

    async def record(self, kind: str, payload: Message | None = None, **fields: Any) -> Any:
        definition = self._records.get(kind)
        if definition is None:
            raise ValueError(f"Append Record: Undeclared Record Kind {kind}")
        row = payload if payload is not None else definition.row(**fields)
        if not isinstance(row, definition.row):
            raise TypeError(f"Append Record: Invalid Payload for {kind}")
        return await self._call(
            message(
                "dokime.plugin.host.v1.evidence.RecordsAppendRequest",
                execution=self.execution,
                kind=kind,
                payload=row.SerializeToString(),
                request_id=str(uuid4()),
            )
        )

    def records(self, kind: str, *, step: str = "") -> "RecordsQuery":
        if kind not in self._records:
            raise ValueError(f"Read Records: Undeclared Record Kind {kind}")
        return RecordsQuery(self, self._records[kind], step)

    async def statistics(self, stream: str, field: str | None = None, *, step: str = "") -> Any:
        return await self._call(
            message(
                "dokime.plugin.host.v1.EvidenceStatisticsRequest", execution=self.execution, stream=stream, field=field or "", step=step
            )
        )

    async def secret(self, identifier: str) -> bytes:
        return (await self._call(message("dokime.plugin.host.v1.ReadSecret", identifier=identifier))).value

    async def settings(self) -> dict[str, Any]:
        response = await self._call(message("dokime.plugin.host.v1.ReadSettings"))
        return {key: decode(value) for key, value in response.values.items()}

    async def progress(self, completed: int, total: int, *, label: str = "") -> None:
        current = await self._call(message("dokime.plugin.host.v1.executions.ProgressGetRequest", name=self.execution))
        await self._call(
            message(
                "dokime.plugin.host.v1.executions.ProgressSetRequest",
                name=self.execution,
                value={"completed": completed, "total": total, "label": label},
                expected_revision=current.revision,
                request_id=str(uuid4()),
            )
        )

    async def log(self, text: str, *, severity: int = 2) -> None:
        await self._call(
            message(
                "dokime.plugin.host.v1.evidence.LogsAppendRequest",
                execution=self.execution,
                message=text,
                severity=severity,
                request_id=str(uuid4()),
            )
        )

    async def traceability(self, identifier: str, value: Any, *, label: str = "", source: str = "", unit: str = "") -> None:
        entry = message(
            "TraceabilityEntry",
            metadata={"identifier": identifier, "label": label or identifier},
            value=encode(value),
            source=source,
            unit=unit,
        )
        current = await self._call(message("dokime.plugin.host.v1.evidence.TraceabilityGetRequest", name=self.execution, entry=identifier))
        await self._call(
            message(
                "dokime.plugin.host.v1.evidence.TraceabilitySetRequest",
                name=self.execution,
                entry=identifier,
                value=entry,
                expected_revision=current.revision,
                request_id=str(uuid4()),
            )
        )

    async def document(self, document: str, part: Message, *, section: str = "") -> None:
        field = {"dokime.v1.Section": "subsection", "dokime.v1.Card": "card"}.get(part.DESCRIPTOR.full_name)
        if field is None:
            raise TypeError("Append Document Parts: Supply a Section or Card Message")
        await self._call(
            message(
                "dokime.plugin.host.v1.DocumentsAppendPartsRequest",
                execution=self.execution,
                document=document,
                section=section,
                request_id=str(uuid4()),
                **{field: part},
            )
        )

    async def artifact(self, name: str, data: bytes, *, mime: str = "application/octet-stream", replace: bool = False) -> Any:
        from uuid import uuid4

        return await self._call(
            message(
                "dokime.plugin.host.v1.content.PublishArtifact",
                transfer_id=str(uuid4()),
                name=name,
                mime=mime,
                data=data,
                last=True,
                replace=replace,
            )
        )

    async def copy_files(self, source: Message, output: str) -> Any:
        return await self._call(message("dokime.plugin.host.v1.FilesCaptureRequest", source=source, output=output))

    async def start_capture(self, source: Message, output: str, mode: Message) -> Any:
        return await self._call(message("dokime.plugin.host.v1.files.CapturesStartRequest", source=source, output=output, mode=mode))

    async def stop_capture(self, capture_id: int) -> Any:
        return await self._call(message("dokime.plugin.host.v1.files.CapturesStopRequest", capture_id=capture_id))

    async def environment(self) -> Any:
        return await self._call(message("dokime.plugin.host.v1.BackendEnvironmentRequest"))

    async def prepare_effect(self, resource: str, inverse_operation: str, inverse: Message, *, description: str = "") -> Any:
        return await self._call(
            message(
                "dokime.plugin.host.v1.EffectsCreateRequest",
                resource=resource,
                inverse_operation=inverse_operation,
                inverse=inverse.SerializeToString(),
                description=description,
                request_id=str(uuid4()),
            )
        )

    async def confirm_effect(self, ticket: str, *, at: Timestamp | None = None, restored: bool = False) -> Any:
        current = await self._call(message("dokime.plugin.host.v1.EffectsGetRequest", name=ticket))
        value = message("dokime.plugin.host.v1.EffectWritable", restored=restored)
        if at is not None:
            value.applied_at.CopyFrom(at)
        return await self._call(
            message(
                "dokime.plugin.host.v1.EffectsSetRequest",
                name=ticket,
                value=value,
                expected_revision=current.revision,
                request_id=str(uuid4()),
            )
        )


class BoundOperation[RequestT: Message, ResponseT: Message]:
    __type_params__: tuple[Any, ...]

    def __init__(self, transport: Any, operation: Operation[RequestT, ResponseT]) -> None:
        self._transport = transport
        self._operation = operation

    async def __call__(self, request: RequestT, *, policy: wire.OperationPolicy | None = None) -> ResponseT:
        return await self._transport.call(self._operation, request, policy=policy)


class RecordsQuery:
    def __init__(self, context: Context, definition: Record, step: str) -> None:
        self._context = context
        self._definition = definition
        self._step = step

    async def read(self, *, limit: int = 0) -> Any:
        return await self._context._call(
            message(
                "dokime.plugin.host.v1.evidence.RecordsListRequest",
                execution=self._context.execution,
                kind=self._definition.identifier,
                step=self._step,
                page={"page_size": limit},
            )
        )

    async def count(self) -> int:
        return (await self.read()).count

    async def last(self) -> Message | None:
        response = await self.read()
        return self._definition.row.FromString(response.last.payload) if response.HasField("last") else None


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
