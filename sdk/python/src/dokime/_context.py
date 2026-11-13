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
            "dokime.context.v1.evidence.Records/Append",
            "dokime.context.v1.evidence.RecordsAppendRequest",
            "dokime.context.v1.evidence.RecordsAppendResponse",
        ),
        (
            "dokime.context.v1.evidence.Records/List",
            "dokime.context.v1.evidence.RecordsListRequest",
            "dokime.context.v1.evidence.RecordsListResponse",
        ),
        (
            "dokime.context.v1.Evidence/Statistics",
            "dokime.context.v1.EvidenceStatisticsRequest",
            "dokime.context.v1.EvidenceStatisticsResponse",
        ),
        (
            "dokime.context.v1.executions.Progress/Get",
            "dokime.context.v1.executions.ProgressGetRequest",
            "dokime.context.v1.executions.ProgressGetResponse",
        ),
        (
            "dokime.context.v1.executions.Progress/Set",
            "dokime.context.v1.executions.ProgressSetRequest",
            "dokime.context.v1.executions.ProgressSetResponse",
        ),
        (
            "dokime.context.v1.evidence.Logs/Append",
            "dokime.context.v1.evidence.LogsAppendRequest",
            "dokime.context.v1.evidence.LogsAppendResponse",
        ),
        ("dokime.settings.read", "ReadSettings", "SavedSettings"),
        ("dokime.secrets.read", "ReadSecret", "Secret"),
        ("dokime.context.v1.Prompts/Create", "dokime.context.v1.PromptsCreateRequest", "dokime.context.v1.PromptsCreateResponse"),
        ("dokime.execution.traceability", "SetTraceability", "SetTraceabilityResponse"),
        (
            "dokime.context.v1.Documents/AppendParts",
            "dokime.context.v1.DocumentsAppendPartsRequest",
            "dokime.context.v1.DocumentsAppendPartsResponse",
        ),
        ("dokime.execution.artifact", "PublishArtifact", "ArtifactReceipt"),
        ("dokime.context.v1.Files/Capture", "dokime.context.v1.FilesCaptureRequest", "dokime.context.v1.FilesCaptureResponse"),
        (
            "dokime.context.v1.files.Captures/Start",
            "dokime.context.v1.files.CapturesStartRequest",
            "dokime.context.v1.files.CapturesStartResponse",
        ),
        (
            "dokime.context.v1.files.Captures/Stop",
            "dokime.context.v1.files.CapturesStopRequest",
            "dokime.context.v1.files.CapturesStopResponse",
        ),
        (
            "dokime.context.v1.Backend/Environment",
            "dokime.context.v1.BackendEnvironmentRequest",
            "dokime.context.v1.BackendEnvironmentResponse",
        ),
        ("dokime.execution.effect.prepare", "PrepareEffect", "Ticket"),
        ("dokime.execution.effect.applied", "ConfirmEffect", "ConfirmEffectResponse"),
    )
) + (GET_TIME, SLEEP)

# === Types === #

# === Classes === #


class Context:
    def __init__(self, transport, parameters: dict[str, Any], operations: Sequence[Operation], records: Sequence[Record] = ()):
        self._transport = transport
        self.parameters = parameters
        self.execution = transport.scope()
        self.attempt = transport.attempt
        self._operations = tuple(operations)
        self._records = {item.identifier: item for item in records}

    async def operation(self, request: Message, *, operation: Operation | None = None, policy=None) -> Any:
        candidates = [item for item in self._operations if item.request.DESCRIPTOR.full_name == request.DESCRIPTOR.full_name]
        if operation is None and len(candidates) != 1:
            raise ValueError(f"Name the declared operation for {request.DESCRIPTOR.full_name}")
        selected = operation if operation is not None else candidates[0]
        return await self._transport.call(selected, request, policy=policy)

    async def sleep(self, duration: float | Duration) -> None:
        if not isinstance(duration, Duration):
            milliseconds = int(duration * 1000)
            duration = Duration(seconds=milliseconds // 1000, nanos=(milliseconds % 1000) * 1_000_000)
        await self._transport.sleep(duration)

    async def now(self):
        return await self._transport.now()

    async def cancelled(self) -> str:
        return await self._transport.cancelled()

    async def prompt(self, text: str, *, choices: Sequence[str] = ()) -> str:
        definition = message("Prompt", metadata={"identifier": "question", "label": text}, message=text)
        for index, choice in enumerate(choices or ("OK",)):
            definition.actions.add(identifier=str(index), label=choice)
        response = await self.operation(message("dokime.context.v1.PromptsCreateRequest", prompt="question", definition=definition))
        if not response.accepted:
            raise ValueError("Operator answer was not accepted")
        actions = {action.identifier: action.label for action in definition.actions}
        return actions.get(response.action, response.action)

    async def record(self, kind: str, payload: Message | None = None, **fields: Any):
        definition = self._records.get(kind)
        if definition is None:
            raise ValueError(f"Undeclared record kind: {kind}")
        row = payload if payload is not None else definition.row(**fields)
        if not isinstance(row, definition.row):
            raise TypeError(f"Invalid record payload: {kind}")
        return await self.operation(
            message("dokime.context.v1.evidence.RecordsAppendRequest", execution=self.execution, kind=kind, payload=row.SerializeToString())
        )

    def records(self, kind: str, *, step: str = "") -> "RecordsQuery":
        if kind not in self._records:
            raise ValueError(f"Undeclared record kind: {kind}")
        return RecordsQuery(self, self._records[kind], step)

    async def statistics(self, stream: str, field: str | None = None, *, step: str = ""):
        return await self.operation(
            message("dokime.context.v1.EvidenceStatisticsRequest", execution=self.execution, stream=stream, field=field or "", step=step)
        )

    async def secret(self, identifier: str) -> bytes:
        return (await self.operation(message("ReadSecret", identifier=identifier))).value

    async def settings(self) -> dict[str, Any]:
        response = await self.operation(message("ReadSettings"))
        return {key: decode(value) for key, value in response.values.items()}

    async def progress(self, completed: int, total: int, *, label: str = "") -> None:
        current = await self.operation(message("dokime.context.v1.executions.ProgressGetRequest", name=self.execution))
        await self.operation(
            message(
                "dokime.context.v1.executions.ProgressSetRequest",
                name=self.execution,
                value={"completed": completed, "total": total, "label": label},
                expected_revision=current.revision,
                request_id=str(uuid4()),
            )
        )

    async def log(self, text: str, *, severity: int = 2) -> None:
        await self.operation(
            message("dokime.context.v1.evidence.LogsAppendRequest", execution=self.execution, message=text, severity=severity)
        )

    async def traceability(self, identifier: str, value: Any, *, label: str = "", source: str = "", unit: str = "") -> None:
        entry = message(
            "TraceabilityEntry",
            metadata={"identifier": identifier, "label": label or identifier},
            value=encode(value),
            source=source,
            unit=unit,
        )
        await self.operation(message("SetTraceability", execution=self.execution, entries=[entry]))

    async def document(self, document: str, part: Message, *, section: str = "") -> None:
        field = {"dokime.v1.Section": "subsection", "dokime.v1.Card": "card"}.get(part.DESCRIPTOR.full_name)
        if field is None:
            raise TypeError("Document parts must be Section or Card messages")
        await self.operation(
            message(
                "dokime.context.v1.DocumentsAppendPartsRequest",
                execution=self.execution,
                document=document,
                section=section,
                **{field: part},
            )
        )

    async def artifact(self, name: str, data: bytes, *, mime: str = "application/octet-stream", replace: bool = False):
        from uuid import uuid4

        return await self.operation(
            message("PublishArtifact", transfer_id=str(uuid4()), name=name, mime=mime, data=data, last=True, replace=replace)
        )

    async def copy_files(self, source: Message, output: str):
        return await self.operation(message("dokime.context.v1.FilesCaptureRequest", source=source, output=output))

    async def start_capture(self, source: Message, output: str, mode: Message):
        return await self.operation(message("dokime.context.v1.files.CapturesStartRequest", source=source, output=output, mode=mode))

    async def stop_capture(self, capture_id: int):
        return await self.operation(message("dokime.context.v1.files.CapturesStopRequest", capture_id=capture_id))

    async def environment(self):
        return await self.operation(message("dokime.context.v1.BackendEnvironmentRequest"))

    async def prepare_effect(self, resource: str, inverse_operation: str, inverse: Message, *, description: str = ""):
        return await self.operation(
            message(
                "PrepareEffect",
                resource=resource,
                inverse_operation=inverse_operation,
                inverse=inverse.SerializeToString(),
                description=description,
            )
        )

    async def confirm_effect(self, ticket: str, *, at: str = "", restored: bool = False):
        return await self.operation(message("ConfirmEffect", ticket=ticket, at=at, restored=restored))


class RecordsQuery:
    def __init__(self, context: Context, definition: Record, step: str):
        self._context = context
        self._definition = definition
        self._step = step

    async def read(self, *, limit: int = 0):
        return await self._context.operation(
            message(
                "dokime.context.v1.evidence.RecordsListRequest",
                execution=self._context.execution,
                kind=self._definition.identifier,
                step=self._step,
                limit=limit,
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
