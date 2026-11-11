# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator
from dataclasses import dataclass
from pathlib import Path

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from ._transport import Transport
from .models import Backend

# Type Checking Imports

# === Public Exports === #

# === Constants === #

FOLLOW_BUFFER_BYTES: int

# === Types === #

# === Classes === #

@dataclass(frozen=True, slots=True)
class FileIdentity:
    store: str
    epoch: str
    file_id: str
    execution_id: str
    relative_path: str

    def _wire(self) -> dict: ...
    @classmethod
    def _from_wire(cls, value: Message) -> FileIdentity: ...

@dataclass(frozen=True, slots=True)
class FileUpdate:
    identity: FileIdentity
    committed_offset: int
    phase: str
    sha256: str | None = None

# === Public Functions === #

def follow_file(
    transport: Transport,
    backend: Backend,
    execution_id: str,
    relative_path: str,
    destination: Path,
    *,
    identity: FileIdentity | None = None,
    max_bytes: int | None = None,
) -> AsyncGenerator[FileUpdate]: ...
