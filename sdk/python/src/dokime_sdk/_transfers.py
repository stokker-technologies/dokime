# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Bounded Uploads and Verified Report Downloads."""

# === Imports === #

# Standard Library Imports
from hashlib import sha256
from mimetypes import guess_type
from typing import Protocol
from uuid import uuid4

# Third Party Imports
from grpc.aio import AioRpcError
from harmos import FileMetadata, FileReference, Upload

# Local Imports
from ._errors import SdkError, call, invalid
from ._generated.client import ClientRoot
from ._generated.client_types import ReportSubject
from ._generated.host import HostRoot
from ._generated.shared import UploadsAppendResponse, UploadsCreateResponse, UploadsDeleteResponse, UploadsPublishResponse
from ._values import Artifact
from ._wire.model.v1.evidence_pb2 import ArtifactScope

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Uploads(Protocol):
    async def create(self, metadata: FileMetadata, *, request_id: str | None = None) -> UploadsCreateResponse: ...
    async def append(self, upload: Upload, offset: int, data: bytes, *, request_id: str | None = None) -> UploadsAppendResponse: ...
    async def publish(self, upload: Upload, length: int, sha256: bytes, *, request_id: str | None = None) -> UploadsPublishResponse: ...
    async def delete(self, upload: Upload, *, request_id: str | None = None) -> UploadsDeleteResponse: ...


# === Public Functions === #


async def publish_artifact(
    transport: ClientRoot | HostRoot,
    scope: ArtifactScope,
    name: str,
    data: bytes,
    *,
    mime: str | None = None,
    request_id: str | None = None,
) -> Artifact:
    if request_id == "":
        raise ValueError("Publish Artifact: Supply a Nonempty Request Identity")
    identity = request_id if request_id is not None else str(uuid4())
    media_type = mime if mime is not None else guess_type(name)[0] or "application/octet-stream"
    file = await upload_bytes(
        transport.files.uploads,
        data,
        metadata=FileMetadata(
            scope=f"execution/{scope.execution_id}" if scope.execution_id else f"cycle/{scope.cycle_id}",
            media_type=media_type,
            expected_length=len(data),
        ),
        request_id=identity,
    )
    reply = await call(
        "content.artifacts.create",
        transport.content.artifacts.create(file, name, media_type, scope=scope, request_id=f"{identity}/attach"),
        identity,
    )
    value = reply.artifact
    return Artifact(
        value.artifact_id,
        value.filename,
        value.content_type,
        value.size_bytes,
        bytes.fromhex(value.sha256),
        value.scope.cycle_id or None,
        value.scope.execution_id or None,
    )


async def upload_bytes(uploads: Uploads, data: bytes, *, metadata: FileMetadata, request_id: str | None = None) -> FileReference:
    identity = request_id if request_id is not None else str(uuid4())
    created = await call("uploads.create", uploads.create(metadata, request_id=f"{identity}/create"), identity)
    published_file = False
    handle = Upload.FromString(created.upload.SerializeToString())
    try:
        if not created.HasField("upload") or not 0 < created.chunk_bytes < created.message_bytes:
            raise invalid("uploads.create", "Invalid Upload or Host Limits", request_id=identity)
        for offset in range(0, len(data), created.chunk_bytes):
            chunk = data[offset : offset + created.chunk_bytes]
            acknowledged = await call(
                "uploads.append", uploads.append(handle, offset, chunk, request_id=f"{identity}/append/{offset}"), identity
            )
            if (
                not acknowledged.HasField("upload")
                or acknowledged.upload.id != handle.id
                or acknowledged.upload.history != handle.history
                or acknowledged.upload.writer_generation != handle.writer_generation
                or acknowledged.upload.accepted_length != offset + len(chunk)
            ):
                raise invalid("uploads.append", "The Host Returned a Different Upload Fence or Offset", request_id=identity)
            handle.CopyFrom(acknowledged.upload)
        published = await call(
            "uploads.publish", uploads.publish(handle, len(data), sha256(data).digest(), request_id=f"{identity}/publish"), identity
        )
        published_file = True
        if not published.HasField("upload") or published.upload.length != len(data) or published.upload.sha256 != sha256(data).digest():
            raise invalid("uploads.publish", "Published Identity Differs from the Content", request_id=identity)
        return published.upload
    except BaseException as error:
        try:
            if not published_file and created.HasField("upload"):
                await uploads.delete(handle, request_id=f"{identity}/abandon")
        except (SdkError, AioRpcError, ValueError, OSError, RuntimeError) as cleanup:
            error.add_note(f"Abandon Upload Failed: {cleanup}")
        raise


async def download_report(
    client: ClientRoot, execution: str, *, document: str = "", max_bytes: int, request_id: str | None = None
) -> bytes:
    if max_bytes <= 0:
        raise ValueError("Download Report: Supply a Positive Read Limit")
    report = await call(
        "reports.create", client.reports.create(ReportSubject(execution=execution), document, request_id=request_id), request_id
    )
    if not report.HasField("file"):
        raise invalid("reports.create", "The Host Returned No Report File")
    return await download(client, report.file, max_bytes=max_bytes)


async def download(client: ClientRoot, file: FileReference, *, max_bytes: int | None = None) -> bytes:
    if max_bytes is not None and max_bytes <= 0:
        raise ValueError("Download File: Supply a Positive Read Limit")
    limits = await call("files.get", client.files.get(file))
    if limits.chunk_bytes <= 0 or len(file.sha256) != 32:
        raise invalid("files.get", "The Host Returned Invalid Transfer Bounds or Digest")
    bound = min(limits.chunk_bytes, max_bytes) if max_bytes is not None else limits.chunk_bytes
    data = bytearray()
    while len(data) < file.length:
        page = await call("files.read", client.files.read(file, len(data), bound))
        if page.total_length != file.length or page.sha256 != file.sha256:
            raise invalid("files.read", "The File Identity Changed During Download")
        if not page.data or len(page.data) > bound or len(data) + len(page.data) > file.length:
            raise invalid("files.read", "The Host Returned an Invalid File Range")
        data.extend(page.data)
    if sha256(data).digest() != file.sha256:
        raise invalid("files.read", "The File Digest Does Not Match Its Contents")
    return bytes(data)


# === Private Functions === #

# === Entry Point === #
