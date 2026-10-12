# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Immutable execution evidence and verified, bounded-memory downloads."""

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncIterator
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# Third Party Imports
from google.protobuf.json_format import MessageToDict
from google.protobuf.message import Message

# Local Imports
from . import _wire
from dokime._native.client import ClientError
from ._generated.dokime.v1 import checkpoint_pb2 as checkpoint
from ._generated.dokime.v1 import content_pb2 as content
from ._generated.dokime.v1 import history_pb2 as history
from ._generated.dokime.v1 import reports_pb2 as reports
from ._transport import Transport, _error
from .errors import ProtocolError, RpcError
from .models import Backend, SavedSample, SavedRecord, Value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class StoreHistory:
    """The original logical store and its non-reusable history epoch."""

    store: str
    epoch: str


@dataclass(frozen=True)
class StorageCursor:
    """An exact position in the original native history."""

    history: StoreHistory
    sequence: int

    @classmethod
    def _from_wire(cls, value: checkpoint.StorageCursor) -> "StorageCursor":
        if (
            not value.HasField("history")
            or not value.history.store
            or not value.history.epoch
        ):
            raise ProtocolError("checkpoint storage history is missing")
        return cls(
            StoreHistory(value.history.store, value.history.epoch), value.sequence
        )

    def _wire(self) -> checkpoint.StorageCursor:
        return checkpoint.StorageCursor(
            history=checkpoint.StoreHistory(
                store=self.history.store, epoch=self.history.epoch
            ),
            sequence=self.sequence,
        )


@dataclass(frozen=True)
class Checkpoint:
    """The inspected sealed revision and its original captured evidence cut."""

    execution_id: str
    position: StorageCursor
    version: bytes
    evidence_through: StorageCursor

    @classmethod
    def _from_wire(cls, value: checkpoint.ExecutionCheckpoint) -> "Checkpoint":
        if (
            not value.execution_identifier
            or not value.HasField("position")
            or not value.HasField("evidence_through")
            or not 0 < len(value.version) <= 4096
        ):
            raise ProtocolError("native execution checkpoint is incomplete")
        position = StorageCursor._from_wire(value.position)
        evidence = StorageCursor._from_wire(value.evidence_through)
        if (
            position.history != evidence.history
            or evidence.sequence > position.sequence
        ):
            raise ProtocolError("checkpoint evidence differs from inspected history")
        return cls(value.execution_identifier, position, bytes(value.version), evidence)

    def _wire(self) -> checkpoint.ExecutionCheckpoint:
        return checkpoint.ExecutionCheckpoint(
            execution_identifier=self.execution_id,
            position=self.position._wire(),
            version=self.version,
            evidence_through=self.evidence_through._wire(),
        )


@dataclass(frozen=True)
class TraceabilityEntry:
    metadata: dict[str, Any]
    value: Value
    source: str | None
    unit: str | None


@dataclass(frozen=True)
class Traceability:
    id: str
    scope: dict[str, str]
    created_at: str | None
    metadata: dict[str, Any] | None
    entries: tuple[TraceabilityEntry, ...]


@dataclass(frozen=True)
class Artifact:
    id: str
    filename: str
    content_type: str | None
    size_bytes: int
    sha256: str
    state: str
    scope: dict[str, str]


@dataclass(frozen=True)
class Download:
    path: Path
    size_bytes: int
    sha256: str
    metadata: dict[str, Any]


class ExecutionEvidence:
    """Reads one immutable checkpoint. JSON documents retain protobuf field names.

    The document contains captured parameters and definitions, not current
    plugin metadata. Record/sample iterators fail on incomplete history pages.
    """

    def __init__(self, transport: Transport, point: Checkpoint, backend: Backend):
        self._transport = transport
        self.checkpoint = point
        self._backend = backend

    async def document(self) -> dict[str, Any]:
        await self._transport.verify(self._backend)
        handle = await self._transport.execution(self.checkpoint.execution_id)
        try:
            payload = await handle.document_at(
                self.checkpoint._wire().SerializeToString()
            )
        except ClientError as error:
            raise _history_error(error) from error
        reply = history.CheckpointDocument.FromString(payload)
        return _json(reply.document.execution)

    async def traceability(self) -> tuple[Traceability, ...]:
        await self._transport.verify(self._backend)
        handle = await self._transport.execution(self.checkpoint.execution_id)
        try:
            payload = await handle.context_at(
                self.checkpoint._wire().SerializeToString()
            )
        except ClientError as error:
            raise _history_error(error) from error
        reply = history.CheckpointContext.FromString(payload)
        return tuple(
            Traceability(
                group.traceability_id,
                _wire.scope(group.scope),
                group.created_at.ToJsonString()
                if group.HasField("created_at")
                else None,
                _json(group.metadata) if group.HasField("metadata") else None,
                tuple(
                    TraceabilityEntry(
                        _json(entry.metadata),
                        _wire.decode(entry.value),
                        entry.source if entry.HasField("source") else None,
                        entry.unit if entry.HasField("unit") else None,
                    )
                    for entry in group.entries
                ),
            )
            for group in reply.traceability
        )

    async def _inventory(self) -> history.CheckpointEvidence:
        await self._transport.verify(self._backend)
        handle = await self._transport.execution(self.checkpoint.execution_id)
        try:
            payload = await handle.evidence_at(
                self.checkpoint._wire().SerializeToString()
            )
        except ClientError as error:
            raise _history_error(error) from error
        return history.CheckpointEvidence.FromString(payload)

    async def schemas(self) -> dict[str, Any]:
        """Captured record and stream schemas for decoding the JSONL evidence."""
        reply = await self._inventory()
        return {
            "records": [_json(item) for item in reply.record_schemas],
            "streams": [_json(item) for item in reply.telemetry_tables],
        }

    async def artifacts(self) -> tuple[Artifact, ...]:
        reply = await self._inventory()
        return tuple(
            Artifact(
                item.artifact_id,
                item.filename,
                item.content_type if item.HasField("content_type") else None,
                item.size_bytes,
                item.sha256,
                content.ArtifactState.Name(item.state),
                _wire.scope(item.scope),
            )
            for item in reply.artifacts
        )

    async def records(self) -> AsyncIterator[SavedRecord]:
        await self._transport.verify(self._backend)
        handle = await self._transport.execution(self.checkpoint.execution_id)
        try:
            reader = await handle.records_at(
                self.checkpoint._wire().SerializeToString()
            )
            try:
                while (payload := await reader.message()) is not None:
                    page = history.CheckpointRecordPage.FromString(payload)
                    for item in page.records:
                        yield SavedRecord(item.epoch, _wire.record(item.record))
            finally:
                await reader.close()
        except ClientError as error:
            raise _history_error(error) from error

    async def samples(self) -> AsyncIterator[SavedSample]:
        await self._transport.verify(self._backend)
        handle = await self._transport.execution(self.checkpoint.execution_id)
        try:
            reader = await handle.samples_at(
                self.checkpoint._wire().SerializeToString()
            )
            try:
                while (payload := await reader.message()) is not None:
                    page = history.CheckpointTelemetryPage.FromString(payload)
                    for item in page.rows:
                        yield SavedSample(
                            item.epoch, item.ordinal, _wire.sample(item.sample)
                        )
            finally:
                await reader.close()
        except ClientError as error:
            raise _history_error(error) from error

    async def download_artifact(
        self, artifact: Artifact, destination: str | Path
    ) -> Download:
        """Download captured bytes; incomplete artifact states remain explicit."""
        await self._transport.verify(self._backend)
        path = Path(destination)
        size, sha256, metadata = await self._transport.download(
            history.DownloadCheckpointArtifactRequest(
                checkpoint=self.checkpoint._wire(),
                artifact_id=artifact.id,
            ),
            path,
            (artifact.size_bytes, artifact.sha256),
        )
        return Download(
            path, size, sha256, _json(content.ArtifactMetadata.FromString(metadata))
        )


# === Public Functions === #


async def download_report(
    transport: Transport,
    cycle_id: str,
    destination: Path,
    identifier: str,
) -> Download:
    size, sha256, metadata = await transport.download(
        reports.DownloadCycleReportRequest(cycle_id=cycle_id, identifier=identifier),
        destination,
    )
    return Download(
        destination, size, sha256, _json(reports.ReportMetadata.FromString(metadata))
    )


# === Private Functions === #


def _json(message: Message) -> dict[str, Any]:
    return MessageToDict(message, preserving_proto_field_name=True)


def _history_error(error: ClientError) -> Exception:
    mapped = _error(error)
    if isinstance(mapped, RpcError) and mapped.code == "DATA_LOSS":
        return ProtocolError(str(mapped))
    return mapped


# === Entry Point === #
