# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Evidence Identity Terminal Status And Atomic Destination Publication"""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
from pathlib import Path
from uuid import UUID

# Third Party Imports
import pytest
from google.protobuf.message import Message

# Local Imports
from dokime.client import _contracts
from dokime.client._transport import Transport
from dokime.client.errors import ProtocolError, RpcError
from dokime.client.exports import export_evidence
from dokime.client.models import Backend

# Type Checking Imports


# === Public Exports === #

# === Constants === #
IDENTITY = str(UUID(int=1))
DATA = b"saved-evidence:" + bytes(range(256)) * 400
BACKEND = Backend("host", "epoch", "0.6", "Export Fixture", frozenset(), max_evidence_bytes=1024 * 1024)

# === Types === #


# === Classes === #
class Reader:
    def __init__(self, frames: list[Message], failure: Exception | None = None, *, stall: bool = False) -> None:
        self.frames = iter(frames)
        self.failure = failure
        self.stall = stall
        self.waiting = asyncio.Event()
        self.closed = False

    def __aiter__(self) -> "Reader":
        return self

    async def __anext__(self) -> Message:
        if frame := next(self.frames, None):
            return frame
        if self.stall:
            self.waiting.set()
            await asyncio.Event().wait()
        if self.failure is not None:
            raise self.failure
        raise StopAsyncIteration

    async def close(self) -> None:
        self.closed = True


class ExportTransport(Transport):
    def __init__(self, reader: Reader) -> None:
        self.reader = reader
        self.verifications = 0
        self.requests = 0

    async def verify(self, expected: Backend, *, live: bool = False) -> None:
        assert expected == BACKEND
        assert not live
        self.verifications += 1

    async def stream(self, service: str, method: str, request: Message):
        assert (service, method) == ("Content", "DownloadEvidence")
        assert request.subject.Execution.field_0 == IDENTITY
        self.requests += 1
        return self.reader


# === Public Functions === #
def test_verified_export(tmp_path: Path) -> None:
    destination = tmp_path / "execution.zip"
    destination.write_bytes(b"previous")
    transport = ExportTransport(Reader(_frames()))
    with pytest.raises(FileExistsError):
        asyncio.run(export_evidence(transport, BACKEND, "Execution", IDENTITY, destination))
    assert (transport.verifications, transport.requests) == (0, 0)
    assert destination.read_bytes() == b"previous"
    result = asyncio.run(export_evidence(transport, BACKEND, "Execution", IDENTITY, destination, overwrite=True))
    assert result.path == destination
    assert result.size_bytes == len(DATA)
    assert result.sha256 == hashlib.sha256(DATA).hexdigest()
    assert destination.read_bytes() == DATA
    assert transport.requests == 1
    assert transport.reader.closed
    assert not destination.with_suffix(".zip.part").exists()


@pytest.mark.parametrize("fault", ["hash", "identity", "size", "trailing", "completion", "status"])
def test_export_refusal(tmp_path: Path, fault: str) -> None:
    destination = tmp_path / "execution.zip"
    destination.write_bytes(b"previous")
    frames = _frames()
    failure = None
    if fault == "hash":
        frames[-1].payload.Complete.field_0.sha256 = "0" * 64
    elif fault == "identity":
        frames[0].payload.Metadata.field_0.subject.Execution.field_0 = str(UUID(int=2))
    elif fault == "size":
        maximum = BACKEND.max_evidence_bytes
        assert maximum is not None
        frames[0].payload.Metadata.field_0.size_bytes = maximum + 1
    elif fault == "trailing":
        frames.append(frames[1])
    elif fault == "completion":
        frames.pop()
    elif fault == "status":
        failure = RpcError("UNAVAILABLE", "final transport status failed")
    transport = ExportTransport(Reader(frames, failure))
    with pytest.raises((ProtocolError, RpcError)):
        asyncio.run(export_evidence(transport, BACKEND, "Execution", IDENTITY, destination, overwrite=True))
    assert destination.read_bytes() == b"previous"
    assert transport.reader.closed
    assert transport.requests == 1
    assert not destination.with_suffix(".zip.part").exists()


def test_export_cancellation(tmp_path: Path) -> None:
    destination = tmp_path / "execution.zip"
    reader = Reader(_frames()[:-1], stall=True)
    transport = ExportTransport(reader)

    async def scenario() -> None:
        task = asyncio.create_task(export_evidence(transport, BACKEND, "Execution", IDENTITY, destination))
        await reader.waiting.wait()
        task.cancel()
        with pytest.raises(asyncio.CancelledError):
            await task

    asyncio.run(scenario())
    assert reader.closed
    assert not destination.exists()
    assert not destination.with_suffix(".zip.part").exists()


# === Private Functions === #
def _frame(branch: str, fields: dict) -> Message:
    output = _contracts.operation("Content", "DownloadEvidence").output
    assert output is not None
    return _contracts.construct(output.DESCRIPTOR, {"payload": {branch: {"field_0": fields}}})


def _frames() -> list[Message]:
    sha256 = hashlib.sha256(DATA).hexdigest()
    metadata = _frame(
        "Metadata",
        {
            "subject": {"Execution": {"field_0": IDENTITY}},
            "schema_version": 1,
            "filename": "execution.zip",
            "size_bytes": len(DATA),
            "sha256": sha256,
        },
    )
    chunks = [_frame("Chunk", {"offset": offset, "data": DATA[offset : offset + 65_536]}) for offset in range(0, len(DATA), 65_536)]
    complete = _frame("Complete", {"size_bytes": len(DATA), "sha256": sha256})
    return [metadata, *chunks, complete]


# === Entry Point === #
