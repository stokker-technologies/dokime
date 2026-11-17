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
from io import FileIO
from os import fstat
from pathlib import Path
from typing import Protocol
from uuid import uuid4

# Third Party Imports
from harmos import FileMetadata, FileReference, Upload

from dokime import UploadsAppendResponse, UploadsCreateResponse, UploadsDeleteResponse, UploadsPublishResponse

from .client import ClientRoot, ReportSubject

# Local Imports

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


async def upload_bytes(uploads: Uploads, data: bytes, *, metadata: FileMetadata, request_id: str | None = None) -> FileReference:
    identity = request_id if request_id is not None else str(uuid4())
    created = await uploads.create(metadata, request_id=f"{identity}/create")
    try:
        if not created.HasField("upload") or not 0 < created.chunk_bytes < created.message_bytes:
            raise ValueError("Upload File: Invalid Upload or Host Limits")
        for offset in range(0, len(data), created.chunk_bytes):
            await uploads.append(
                created.upload, offset, data[offset : offset + created.chunk_bytes], request_id=f"{identity}/append/{offset}"
            )
        published = await uploads.publish(created.upload, len(data), sha256(data).digest(), request_id=f"{identity}/publish")
        if not published.HasField("upload") or published.upload.length != len(data) or published.upload.sha256 != sha256(data).digest():
            raise ValueError("Upload File: Published Identity Differs from the Content")
        return published.upload
    except BaseException:
        await uploads.delete(created.upload, request_id=f"{identity}/abandon")
        raise


async def upload_file(
    client: ClientRoot, path: str | Path, *, scope: str = "", media_type: str = "application/octet-stream", request_id: str | None = None
) -> FileReference:
    identity = request_id if request_id is not None else str(uuid4())
    source = FileIO(Path(path), "r")
    try:
        length = fstat(source.fileno()).st_size
        created = await client.files.uploads.create(
            FileMetadata(scope=scope, media_type=media_type, expected_length=length), request_id=f"{identity}/create"
        )
        try:
            if not created.HasField("upload") or not 0 < created.chunk_bytes < created.message_bytes:
                raise ValueError("Upload File: Invalid Upload or Host Limits")
            digest = sha256()
            offset = 0
            while chunk := source.read(created.chunk_bytes):
                await client.files.uploads.append(created.upload, offset, chunk, request_id=f"{identity}/append/{offset}")
                digest.update(chunk)
                offset += len(chunk)
            if offset != length:
                raise ValueError("Upload File: Source Length Changed")
            published = await client.files.uploads.publish(created.upload, offset, digest.digest(), request_id=f"{identity}/publish")
            if not published.HasField("upload") or published.upload.length != length or published.upload.sha256 != digest.digest():
                raise ValueError("Upload File: Published Identity Differs from the Content")
            return published.upload
        except BaseException:
            await client.files.uploads.delete(created.upload, request_id=f"{identity}/abandon")
            raise
    finally:
        source.close()


async def download_report(
    client: ClientRoot, execution: str, *, document: str = "", max_bytes: int, request_id: str | None = None
) -> bytes:
    if max_bytes <= 0:
        raise ValueError("Download Report: Supply a Positive Read Limit")
    report = await client.reports.create(ReportSubject(execution=execution), document, request_id=request_id)
    if not report.HasField("file"):
        raise ValueError("Download Report: Missing File")
    data = bytearray()
    while True:
        page = await client.files.read(report.file, len(data), max_bytes)
        if page.total_length != report.file.length or page.sha256 != report.file.sha256:
            raise ValueError("Download Report: File Identity Changed")
        if len(data) + len(page.data) > page.total_length:
            raise ValueError("Download Report: Invalid File Length")
        data.extend(page.data)
        if len(data) == page.total_length:
            if sha256(data).digest() != page.sha256:
                raise ValueError("Download Report: Digest Mismatch")
            return bytes(data)
        if not page.data:
            raise ValueError("Download Report: Empty File Page")


# === Private Functions === #

# === Entry Point === #
