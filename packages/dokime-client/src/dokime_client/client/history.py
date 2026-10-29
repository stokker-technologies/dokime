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
import base64
import hashlib
import re
from collections.abc import AsyncIterator
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from uuid import UUID

# Third Party Imports
from google.protobuf.json_format import MessageToDict
from google.protobuf.message import Message

# Local Imports
from . import _wire
from ._transport import Transport
from .errors import ProtocolError
from .models import Backend, SavedRecord, SavedSample, Value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PAGE_ITEMS = 100
VALUE_DEPTH = 64

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
    def _from_wire(cls, value: Message) -> "StorageCursor":
        if not value.HasField("history") or not _identifier(value.history.store) or not _identifier(value.history.epoch):
            raise ProtocolError("checkpoint storage history is missing")
        return cls(StoreHistory(value.history.store, value.history.epoch), value.sequence)

    def _wire(self) -> Message:
        return _wire.nested(
            "History",
            "GetCheckpointDocument",
            "checkpoint.position",
            history=_wire.nested(
                "History",
                "GetCheckpointDocument",
                "checkpoint.position.history",
                store=self.history.store,
                epoch=self.history.epoch,
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
    def _from_wire(cls, value: Message, maximum: int) -> "Checkpoint":
        if len(value.version) > maximum:
            raise ProtocolError("native execution checkpoint exceeds the configured byte bound")
        return cls._validated(value)

    @classmethod
    def _validated(cls, value: Message) -> "Checkpoint":
        if not value.HasField("position") or not value.HasField("evidence_through") or not value.version:
            raise ProtocolError("native execution checkpoint is incomplete")
        try:
            identity = UUID(value.execution_identifier)
        except ValueError as error:
            raise ProtocolError("checkpoint execution identity is invalid") from error
        if str(identity) != value.execution_identifier:
            raise ProtocolError("checkpoint execution identity is not canonical")
        position = StorageCursor._from_wire(value.position)
        evidence = StorageCursor._from_wire(value.evidence_through)
        history, revision = _version_token(bytes(value.version))
        if history != position.history or revision != position.sequence:
            raise ProtocolError("checkpoint version differs from inspected history")
        if position.history != evidence.history or evidence.sequence > position.sequence:
            raise ProtocolError("checkpoint evidence differs from inspected history")
        return cls(value.execution_identifier, position, bytes(value.version), evidence)

    def _wire(self) -> Message:
        value = _wire.nested(
            "History",
            "GetCheckpointDocument",
            "checkpoint",
            execution_identifier=self.execution_id,
            position=self.position._wire(),
            version=self.version,
            evidence_through=self.evidence_through._wire(),
        )
        self._validated(value)
        return value


@dataclass(frozen=True)
class TraceabilityEntry:
    metadata: dict[str, Any]
    value: Value
    source: str | None
    unit: str | None


@dataclass(frozen=True)
class Traceability:
    id: str
    scope: dict[str, Any]
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
    scope: dict[str, Any]


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
        from .execution.handle import Execution

        reply = await Execution._bind(self.checkpoint.execution_id, self._transport, self._backend)._document_at(self.checkpoint)
        return _json(reply.document.execution)

    async def traceability(self) -> tuple[Traceability, ...]:
        await self._transport.verify(self._backend)
        reply = await self._transport.unary(
            "History",
            "GetCheckpointContext",
            _wire.request("History", "GetCheckpointContext", checkpoint=self.checkpoint._wire()),
        )
        _same_checkpoint(self.checkpoint, reply)
        return tuple(
            Traceability(
                group.traceability_id,
                _wire.scope(group.scope),
                _wire.datetime_value(group.created_at)._wire_text if group.HasField("created_at") else None,
                _json(group.metadata) if group.HasField("metadata") else None,
                tuple(
                    TraceabilityEntry(
                        _json(entry.metadata),
                        _wire.decode(entry.value, VALUE_DEPTH),
                        entry.source if entry.HasField("source") else None,
                        entry.unit if entry.HasField("unit") else None,
                    )
                    for entry in group.entries
                ),
            )
            for group in reply.traceability
        )

    async def _inventory(self) -> Message:
        await self._transport.verify(self._backend)
        reply = await self._transport.unary(
            "History",
            "GetCheckpointEvidence",
            _wire.request(
                "History",
                "GetCheckpointEvidence",
                checkpoint=self.checkpoint._wire(),
                sections=[
                    {"Artifacts": {}},
                    {"RecordSchemas": {}},
                    {"TelemetryTables": {}},
                ],
            ),
        )
        _same_checkpoint(self.checkpoint, reply)
        return reply

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
                _wire.enum_name(item.state),
                _wire.scope(item.scope),
            )
            for item in reply.artifacts
        )

    async def records(self) -> AsyncIterator[SavedRecord]:
        await self._transport.verify(self._backend)
        after = ""
        position = 0
        while True:
            await self._transport.verify(self._backend)
            page = await self._transport.unary(
                "History",
                "ListCheckpointRecords",
                _wire.request(
                    "History",
                    "ListCheckpointRecords",
                    checkpoint=self.checkpoint._wire(),
                    limit=PAGE_ITEMS,
                    after=after,
                ),
            )
            _same_checkpoint(self.checkpoint, page)
            if (
                page.issues
                or page.issues_truncated
                or len(page.records) > PAGE_ITEMS
                or any(not row.epoch or not row.HasField("record") for row in page.records)
            ):
                raise ProtocolError("saved record page is incomplete or invalid")
            next_after = page.next_after if page.HasField("next_after") else None
            if next_after is not None:
                position = _page_cursor(self.checkpoint, "records", "", next_after, position)
            for item in page.records:
                yield SavedRecord(item.epoch, _wire.record(item.record, VALUE_DEPTH))
            if next_after is None:
                return
            after = next_after

    async def samples(self) -> AsyncIterator[SavedSample]:
        inventory = await self._inventory()
        tables = sorted(item.table for item in inventory.telemetry_tables)
        if any(not table for table in tables) or len(set(tables)) != len(tables):
            raise ProtocolError("saved telemetry inventory has empty or duplicate streams")
        for table in tables:
            after = ""
            position = 0
            while True:
                await self._transport.verify(self._backend)
                page = await self._transport.unary(
                    "History",
                    "ListCheckpointTelemetry",
                    _wire.request(
                        "History",
                        "ListCheckpointTelemetry",
                        checkpoint=self.checkpoint._wire(),
                        table=table,
                        limit=PAGE_ITEMS,
                        after=after,
                    ),
                )
                _same_checkpoint(self.checkpoint, page)
                if (
                    page.table != table
                    or page.issues
                    or page.issues_truncated
                    or len(page.rows) > PAGE_ITEMS
                    or any(not row.epoch or not row.HasField("sample") or row.sample.stream != table for row in page.rows)
                ):
                    raise ProtocolError("saved telemetry page is incomplete or invalid")
                next_after = page.next_after if page.HasField("next_after") else None
                if next_after is not None:
                    position = _page_cursor(self.checkpoint, "telemetry", table, next_after, position)
                for item in page.rows:
                    yield SavedSample(item.epoch, item.ordinal, _wire.sample(item.sample, VALUE_DEPTH))
                if next_after is None:
                    break
                after = next_after

    async def download_artifact(self, artifact: Artifact, destination: str | Path) -> Download:
        """Download captured bytes; incomplete artifact states remain explicit."""
        await self._transport.verify(self._backend)
        path = Path(destination)
        size, sha256, metadata = await self._transport.download_file(
            _wire.request(
                "History",
                "DownloadCheckpointArtifact",
                checkpoint=self.checkpoint._wire(),
                artifact_id=artifact.id,
            ),
            path,
            (artifact.size_bytes, artifact.sha256),
        )
        return Download(path, size, sha256, _json(metadata))


# === Public Functions === #


async def download_report(
    transport: Transport,
    cycle_id: str,
    destination: Path,
    identifier: str,
) -> Download:
    size, sha256, metadata = await transport.download_file(
        _wire.request("Reports", "DownloadCycleReport", cycle_id=cycle_id, identifier=identifier),
        destination,
    )
    return Download(destination, size, sha256, _json(metadata))


# === Private Functions === #


def _identifier(value: str) -> bool:
    return re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]*", value) is not None


def _version_token(value: bytes) -> tuple[StoreHistory, int]:
    """Read the canonical Postcard history and nonzero revision product."""
    offset = 0
    identifiers = []
    for _ in range(2):
        length, offset = _varint(value, offset)
        if length > len(value) - offset:
            raise ProtocolError("checkpoint version identifier is truncated")
        try:
            identifier = value[offset : offset + length].decode("ascii")
        except UnicodeDecodeError as error:
            raise ProtocolError("checkpoint version identifier is invalid") from error
        if not _identifier(identifier):
            raise ProtocolError("checkpoint version identifier is invalid")
        identifiers.append(identifier)
        offset += length
    revision, offset = _varint(value, offset)
    if revision == 0 or offset != len(value):
        raise ProtocolError("checkpoint version revision or encoding is invalid")
    return StoreHistory(*identifiers), revision


def _varint(value: bytes, offset: int) -> tuple[int, int]:
    result = 0
    for shift in range(0, 70, 7):
        if offset == len(value):
            raise ProtocolError("checkpoint version integer is truncated")
        byte = value[offset]
        offset += 1
        if shift == 63 and byte > 1:
            raise ProtocolError("checkpoint version integer exceeds u64")
        result |= (byte & 0x7F) << shift
        if byte < 0x80:
            if shift and byte == 0:
                raise ProtocolError("checkpoint version integer is not canonical")
            return result, offset
    raise ProtocolError("checkpoint version integer exceeds u64")


def _json(message: Message) -> dict[str, Any]:
    return MessageToDict(message, preserving_proto_field_name=True)


def _same_checkpoint(point: Checkpoint, message: Message) -> None:
    if not message.HasField("checkpoint") or not _wire.equal(message.checkpoint, point._wire()):
        raise ProtocolError("history returned a different checkpoint")


def _page_cursor(point: Checkpoint, kind: str, source: str, token: str, previous: int) -> int:
    if len(token) != 55:
        raise ProtocolError("saved page cursor length differs")
    try:
        raw = base64.b64decode(token + "=", altchars=b"-_", validate=True)
    except ValueError as error:
        raise ProtocolError("saved page cursor is invalid") from error
    if len(raw) != 41 or raw[0] != 1 or base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii") != token:
        raise ProtocolError("saved page cursor version or encoding differs")
    digest = hashlib.sha256()
    for text in (
        "dokime.native-evidence-page.v1",
        kind,
        source,
        point.execution_id,
        point.position.history.store,
        point.position.history.epoch,
        point.evidence_through.history.store,
        point.evidence_through.history.epoch,
    ):
        encoded = text.encode("utf-8")
        digest.update(len(encoded).to_bytes(8, "big"))
        digest.update(encoded)
    digest.update(point.position.sequence.to_bytes(8, "big"))
    digest.update(point.evidence_through.sequence.to_bytes(8, "big"))
    position = int.from_bytes(raw[33:], "big")
    if raw[1:33] != digest.digest() or not previous < position <= point.evidence_through.sequence:
        raise ProtocolError("saved evidence continuation differs or does not advance")
    return position


# === Entry Point === #
