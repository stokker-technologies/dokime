# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncIterator
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from ._transport import Transport
from .models import Backend, SavedRecord, SavedSample, Value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PAGE_ITEMS: int
VALUE_DEPTH: int

# === Types === #

# === Classes === #

@dataclass(frozen=True)
class StoreHistory:
    store: str
    epoch: str

@dataclass(frozen=True)
class StorageCursor:
    history: StoreHistory
    sequence: int

    @classmethod
    def _from_wire(cls, value: Message) -> StorageCursor: ...
    def _wire(self) -> Message: ...

@dataclass(frozen=True)
class Checkpoint:
    execution_id: str
    position: StorageCursor
    version: bytes
    evidence_through: StorageCursor

    @classmethod
    def _from_wire(cls, value: Message, maximum: int) -> Checkpoint: ...
    @classmethod
    def _validated(cls, value: Message) -> Checkpoint: ...
    def _wire(self) -> Message: ...

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
    checkpoint: Checkpoint
    def __init__(self, transport: Transport, point: Checkpoint, backend: Backend): ...
    async def document(self) -> dict[str, Any]: ...
    async def traceability(self) -> tuple[Traceability, ...]: ...
    async def _inventory(self) -> Message: ...
    async def schemas(self) -> dict[str, Any]: ...
    async def artifacts(self) -> tuple[Artifact, ...]: ...
    def records(self) -> AsyncIterator[SavedRecord]: ...
    def samples(self) -> AsyncIterator[SavedSample]: ...
    async def download_artifact(self, artifact: Artifact, destination: str | Path) -> Download: ...

async def download_report(
    transport: Transport,
    cycle_id: str,
    destination: Path,
    identifier: str,
) -> Download: ...
def _identifier(value: str) -> bool: ...
def _version_token(value: bytes) -> tuple[StoreHistory, int]: ...
def _varint(value: bytes, offset: int) -> tuple[int, int]: ...
def _json(message: Message) -> dict[str, Any]: ...
def _same_checkpoint(point: Checkpoint, message: Message) -> None: ...
def _page_cursor(point: Checkpoint, kind: str, source: str, token: str, previous: int) -> int: ...
