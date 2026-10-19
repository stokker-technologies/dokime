# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Download Complete Saved Evidence With Explicit Overwrite"""

# === Imports === #

# Standard Library Imports
import re
from pathlib import Path
from typing import Literal
from uuid import UUID

# Third Party Imports
from google.protobuf.json_format import MessageToDict
from google.protobuf.message import Message

# Local Imports
from . import _wire
from ._transport import Transport
from .errors import ProtocolError
from .history import Download
from .models import Backend

# Type Checking Imports


# === Public Exports === #

# === Constants === #
_MAXIMUM_BYTES = 8 * 1024**3

# === Types === #

# === Classes === #


# === Public Functions === #
async def export_evidence(
    transport: Transport,
    backend: Backend,
    kind: Literal["Cycle", "Execution"],
    identity: str,
    destination: str | Path,
    *,
    overwrite: bool = False,
) -> Download:
    """Download A Sealed Snapshot; Cancellation Never Cancels Execution."""
    identity = str(UUID(identity))
    path = Path(destination)
    _destination(path, overwrite)
    if backend.max_evidence_bytes is None or backend.max_evidence_bytes <= 0:
        raise ProtocolError("backend did not declare evidence export capacity")
    maximum = min(backend.max_evidence_bytes, _MAXIMUM_BYTES)
    subject = {kind: {"field_0": identity}}
    request = _wire.request("Content", "DownloadEvidence", subject=subject)

    def validate(metadata: Message) -> tuple[int, str]:
        if not metadata.HasField("subject") or not _wire.equal(metadata.subject, request.subject) or metadata.schema_version != 1:
            raise ProtocolError("evidence subject or schema differs")
        if re.fullmatch(r"[0-9a-f]{64}", metadata.sha256) is None:
            raise ProtocolError("evidence digest is invalid")
        return metadata.size_bytes, metadata.sha256

    await transport.verify(backend)
    size, sha256, metadata = await transport.download_file(
        request,
        path,
        endpoint=("Content", "DownloadEvidence"),
        validate=validate,
        maximum_bytes=maximum,
        overwrite=overwrite,
    )
    return Download(path, size, sha256, MessageToDict(metadata, preserving_proto_field_name=True))


# === Private Functions === #
def _destination(path: Path, overwrite: bool) -> None:
    if not path.name or not path.parent.is_dir():
        raise ValueError("evidence destination requires a filename and an existing parent directory")
    if path.exists() or path.is_symlink():
        if not overwrite:
            raise FileExistsError("evidence destination exists; select overwrite explicitly")
        if not path.is_file() or path.is_symlink():
            raise ValueError("evidence destination is not a regular file")
    partial = path.with_name(path.name + ".part")
    if partial.exists() or partial.is_symlink():
        raise FileExistsError("evidence download staging file already exists")


# === Entry Point === #
