# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Follow Registered Execution Files With Explicit Resume Identity"""

# === Imports === #

# Standard Library Imports
import hashlib
import os
import re
from collections.abc import AsyncGenerator
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from . import _wire
from .errors import ProtocolError
from .models import Backend

# Type Checking Imports

if TYPE_CHECKING:
    from ._transport import Transport

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True, slots=True)
class FileIdentity:
    store: str
    epoch: str
    file_id: str
    execution_id: str
    relative_path: str

    def _wire(self) -> dict:
        return {
            "history": {"store": self.store, "epoch": self.epoch},
            "file_id": self.file_id,
            "execution_id": self.execution_id,
            "relative_path": self.relative_path,
        }

    @classmethod
    def _from_wire(cls, value: Message) -> "FileIdentity":
        if not value.HasField("history") or not all(
            (
                value.history.store,
                value.history.epoch,
                value.file_id,
                value.execution_id,
                value.relative_path,
            )
        ):
            raise ProtocolError("followed file has no stable original identity")
        return cls(
            value.history.store,
            value.history.epoch,
            value.file_id,
            value.execution_id,
            value.relative_path,
        )


@dataclass(frozen=True, slots=True)
class FileUpdate:
    identity: FileIdentity
    committed_offset: int
    phase: str
    sha256: str | None = None


# === Public Functions === #


async def follow_file(
    transport: "Transport",
    backend: Backend,
    execution_id: str,
    relative_path: str,
    destination: Path,
    *,
    identity: FileIdentity | None = None,
    max_bytes: int = 64 * 1024,
) -> AsyncGenerator[FileUpdate]:
    """Keep Committed Bytes On Cancellation; Resume Only With Their Original Identity"""
    if not 1 <= max_bytes <= 64 * 1024:
        raise ValueError("file chunks must contain 1–65536 bytes")
    if identity is not None and (identity.execution_id, identity.relative_path) != (
        execution_id,
        relative_path,
    ):
        raise ValueError("resume identity belongs to another registered file")
    await transport.verify(backend, live=True)
    # New Downloads Refuse Existing Files. Resume Retains And Hashes The Full Prefix.
    with destination.open("r+b" if identity is not None else "x+b") as output:
        digest = hashlib.sha256()
        offset = 0
        if identity is not None:
            while chunk := output.read(64 * 1024):
                digest.update(chunk)
                offset += len(chunk)
        request = _wire.request(
            "Content",
            "FollowExecutionFile",
            execution_id=execution_id,
            relative_path=relative_path,
            identity=identity._wire() if identity is not None else None,
            offset=offset,
            max_bytes=max_bytes,
        )
        reader = await transport.stream("Content", "FollowExecutionFile", request)
        observed = None
        terminal = None
        try:
            async for frame in reader:
                if terminal is not None or not frame.HasField("payload"):
                    raise ProtocolError("followed file has trailing or empty frames")
                branch = _wire.variant(frame.payload)
                payload = getattr(frame.payload, branch).field_0
                if branch == "Metadata" and observed is None:
                    if not payload.HasField("identity"):
                        raise ProtocolError("followed file metadata has no identity")
                    observed = FileIdentity._from_wire(payload.identity)
                    phase = _wire.enum_name(payload.phase)
                    if (
                        (observed.execution_id, observed.relative_path)
                        != (execution_id, relative_path)
                        or identity is not None
                        and observed != identity
                        or payload.offset != offset
                        or payload.available_length < offset
                        or phase not in ("producing", "finished", "abandoned")
                    ):
                        raise ProtocolError(
                            "followed file identity, phase or prefix differs"
                        )
                    yield FileUpdate(observed, offset, phase)
                elif branch == "Chunk" and observed is not None:
                    if (
                        payload.offset != offset
                        or not payload.data
                        or len(payload.data) > max_bytes
                    ):
                        raise ProtocolError(
                            "followed file chunks are empty, oversized or noncontiguous"
                        )
                    if (
                        output.tell() != offset
                        or os.fstat(output.fileno()).st_size != offset
                    ):
                        raise ProtocolError("local file prefix changed while following")
                    try:
                        written = output.write(payload.data)
                        if written != len(payload.data):
                            raise OSError(
                                "local file write did not commit the complete chunk"
                            )
                        output.flush()
                        os.fsync(output.fileno())
                    except OSError:
                        output.truncate(offset)
                        output.seek(offset)
                        raise
                    digest.update(payload.data)
                    offset += written
                    yield FileUpdate(observed, offset, "producing")
                elif branch == "Terminal" and observed is not None:
                    phase = _wire.enum_name(payload.phase)
                    if (
                        phase not in ("finished", "abandoned")
                        or payload.length != offset
                    ):
                        raise ProtocolError(
                            "followed file terminal length or phase differs"
                        )
                    sha256 = None
                    if phase == "finished":
                        if not payload.HasField("sealed"):
                            raise ProtocolError("finished file has no original seal")
                        seal = payload.sealed
                        if (
                            not seal.HasField("identity")
                            or FileIdentity._from_wire(seal.identity) != observed
                            or not seal.HasField("position")
                            or not seal.position.HasField("history")
                            or (
                                seal.position.history.store,
                                seal.position.history.epoch,
                            )
                            != (observed.store, observed.epoch)
                            or seal.length != offset
                            or re.fullmatch(r"[0-9a-f]{64}", seal.sha256) is None
                            or seal.sha256 != digest.hexdigest()
                        ):
                            raise ProtocolError(
                                "original file seal or complete prefix hash differs"
                            )
                        output.flush()
                        output.seek(0)
                        committed = hashlib.sha256()
                        while chunk := output.read(64 * 1024):
                            committed.update(chunk)
                        if (
                            output.tell() != offset
                            or committed.hexdigest() != seal.sha256
                        ):
                            raise ProtocolError(
                                "committed local prefix differs from the complete file seal"
                            )
                        sha256 = seal.sha256
                    elif payload.HasField("sealed"):
                        raise ProtocolError(
                            "abandoned file unexpectedly carries a seal"
                        )
                    terminal = FileUpdate(observed, offset, phase, sha256)
                else:
                    raise ProtocolError(
                        "followed file requires one metadata frame before data"
                    )
            # The Transport Requires Its Canonical Final Response And Successful Status.
            if terminal is None:
                raise ProtocolError("followed file ended without a terminal frame")
            yield terminal
        finally:
            await reader.close()


# === Private Functions === #

# === Entry Point === #
