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
import json
from pathlib import Path
from uuid import UUID

# Third Party Imports
import pytest

# Local Imports
from dokime.client import Cycle, Execution, _contracts
from dokime.client._transport import Transport
from dokime.client.errors import ProtocolError, RpcError
from dokime.client.exports import export_evidence
from dokime.client.models import Backend
from google.protobuf.message import Message

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
    def __init__(self, reader: Reader, *, kind: str = "Execution") -> None:
        self.reader = reader
        self.kind = kind
        self.verifications = 0
        self.requests = 0

    async def verify(self, expected: Backend, *, live: bool = False) -> None:
        assert expected == BACKEND
        assert not live
        self.verifications += 1

    async def stream(self, service: str, method: str, request: Message):
        assert (service, method) == ("Content", "DownloadEvidence")
        assert getattr(request.subject, self.kind).field_0 == IDENTITY
        self.requests += 1
        return self.reader


# === Public Functions === #


def test_verified_export(tmp_path: Path) -> None:
    destination = tmp_path / "execution.zip"
    destination.write_bytes(b"previous")
    transport = ExportTransport(Reader(_frames()))
    execution = Execution._bind(IDENTITY, transport, BACKEND)
    with pytest.raises(FileExistsError):
        asyncio.run(execution.export(destination))
    assert (transport.verifications, transport.requests) == (0, 0)
    assert destination.read_bytes() == b"previous"
    result = asyncio.run(execution.export(destination, overwrite=True))
    assert result.path == destination
    assert result.size_bytes == len(DATA)
    assert result.sha256 == hashlib.sha256(DATA).hexdigest()
    assert destination.read_bytes() == DATA
    assert transport.requests == 1
    assert transport.reader.closed
    assert not destination.with_suffix(".zip.part").exists()


def test_cycle_export(tmp_path: Path) -> None:
    transport = ExportTransport(Reader(_frames(kind="Cycle")), kind="Cycle")
    cycle = Cycle(IDENTITY, transport, BACKEND, ())
    destination = tmp_path / "cycle.zip"
    saved = asyncio.run(cycle.export(destination))
    assert saved.path.read_bytes() == DATA
    assert saved.sha256 == hashlib.sha256(DATA).hexdigest()
    assert transport.requests == 1
    assert transport.reader.closed


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

    # Retry Transfers the Same Saved Subject Without Admitting Any Execution
    transport.reader = Reader(_frames())
    execution = Execution._bind(IDENTITY, transport, BACKEND)
    saved = asyncio.run(execution.export(destination, overwrite=True))
    assert saved.path.read_bytes() == DATA
    assert transport.requests == 2


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


@pytest.mark.parametrize("report", [None, "cycle.summary"])
def test_lifecycle_save_manifest(tmp_path: Path, report: str | None, monkeypatch: pytest.MonkeyPatch) -> None:
    # Reuse The Existing Chunked Export Fake And Its Real Digest Verification.
    transport = ExportTransport(Reader(_frames()), kind="Execution")
    execution = Execution._bind(IDENTITY, transport, BACKEND)
    cycle = Cycle(IDENTITY, transport, BACKEND, (execution,))
    export = cycle.export

    async def cycle_export(destination):
        transport.kind = "Cycle"
        transport.reader = Reader(_frames(kind="Cycle"))
        return await export(destination)

    async def report_download(destination, *, identifier):
        assert identifier == report
        destination.write_bytes(b"<html>Report</html>")

    monkeypatch.setattr(cycle, "export", cycle_export)
    monkeypatch.setattr(cycle, "download_report", report_download)
    destination = tmp_path / "reports" / IDENTITY
    manifest = asyncio.run(cycle.save(destination, report=report))
    assert json.loads((destination / "manifest.json").read_text()) == manifest
    names = {f"execution-{IDENTITY}.zip", "cycle.zip"}
    if report is not None:
        names.add("report.html")
    assert {entry["path"] for entry in manifest["files"]} == names
    for entry in manifest["files"]:
        content = (destination / entry["path"]).read_bytes()
        assert entry["size"] == len(content)
        assert entry["sha256"] == hashlib.sha256(content).hexdigest()
    assert (destination / "cycle.zip").read_bytes() == DATA
    with pytest.raises(FileExistsError):
        asyncio.run(cycle.save(destination, report=report))


def test_lifecycle_failed_save_has_no_manifest(tmp_path: Path) -> None:
    frames = _frames(kind="Cycle")
    frames[-1].payload.Complete.field_0.sha256 = "0" * 64
    transport = ExportTransport(Reader(frames), kind="Cycle")
    cycle = Cycle(IDENTITY, transport, BACKEND, ())
    with pytest.raises(ProtocolError):
        asyncio.run(cycle.save(tmp_path))
    assert not (tmp_path / "manifest.json").exists()
    assert not (tmp_path / "cycle.zip").exists()


# === Private Functions === #


def _frame(branch: str, fields: dict) -> Message:
    output = _contracts.operation("Content", "DownloadEvidence").output
    assert output is not None
    return _contracts.construct(output.DESCRIPTOR, {"payload": {branch: {"field_0": fields}}})


def _frames(*, kind: str = "Execution") -> list[Message]:
    sha256 = hashlib.sha256(DATA).hexdigest()
    metadata = _frame(
        "Metadata",
        {
            "subject": {kind: {"field_0": IDENTITY}},
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
