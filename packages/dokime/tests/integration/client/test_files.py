# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Original File Identity, Committed Resume Prefixes And Terminal Seals"""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
from pathlib import Path
from types import SimpleNamespace
from typing import Any, cast

# Third Party Imports
import pytest

# Local Imports
from dokime.client import files
from dokime.client._transport import Transport as ClientTransport
from dokime.client.errors import ProtocolError, RpcError
from dokime.client.models import Backend

# Type Checking Imports

# === Public Exports === #

# === Constants === #

DATA = b"registered-output:" + b"a" * 65_536 + b":finished"
IDENTITY = files.FileIdentity(
    "original-store", "original-history", "file-1", "execution-1", "output.bin"
)
BACKEND = Backend(
    id="backend-1",
    epoch="reconnected-runtime",
    api_version="1",
    display_name="Test Backend",
    methods=frozenset({"/dokime.v1.Content/FollowExecutionFile"}),
)

# === Types === #

# === Classes === #


class WireValue(SimpleNamespace):
    """Expose The Message Surface Consumed By The Source Follow Helper"""

    def HasField(self, name: str) -> bool:
        return getattr(self, name, None) is not None


class Variant(WireValue):
    DESCRIPTOR = SimpleNamespace(oneofs=[SimpleNamespace(name="_harmos_variant")])

    def __init__(self, branch: str, payload: WireValue | None = None) -> None:
        super().__init__(**{branch: WireValue(field_0=payload)})
        self.branch = branch

    def WhichOneof(self, name: str) -> str:
        assert name == "_harmos_variant"
        return self.branch


class Reader:
    """Model Transport Final Acceptance Separately From Product Terminal Frames"""

    def __init__(
        self,
        frames: list[WireValue],
        *,
        failure: Exception | None = None,
        stall: bool = False,
    ) -> None:
        self.frames = iter(frames)
        self.failure = failure
        self.stall = stall
        self.waiting = asyncio.Event()
        self.closed = False
        self.final_accepted = False

    def __aiter__(self) -> "Reader":
        return self

    async def __anext__(self) -> WireValue:
        frame = next(self.frames, None)
        if frame is not None:
            return frame
        if self.stall:
            self.waiting.set()
            await asyncio.Event().wait()
        if self.failure is not None:
            raise self.failure
        self.final_accepted = True
        raise StopAsyncIteration

    async def close(self) -> None:
        self.closed = True


class Transport:
    def __init__(self, reader: Reader) -> None:
        self.reader = reader
        self.verified = False
        self.request: dict[str, Any] | None = None

    async def verify(self, backend: Backend, *, live: bool) -> None:
        assert backend == BACKEND
        assert live
        self.verified = True

    async def stream(
        self, service: str, method: str, request: dict[str, Any]
    ) -> Reader:
        assert self.verified
        assert (service, method) == ("Content", "FollowExecutionFile")
        self.request = request
        return self.reader


# === Public Functions === #


@pytest.mark.parametrize("prefix_length", [0, 17, 65_537, len(DATA)])
def test_finished_resume(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, prefix_length: int
) -> None:
    _requests(monkeypatch)
    destination = tmp_path / "output.bin"
    identity = IDENTITY if prefix_length else None
    if identity is not None:
        destination.write_bytes(DATA[:prefix_length])
    reader = Reader(_frames(prefix_length))
    transport = Transport(reader)

    async def scenario() -> list[files.FileUpdate]:
        updates = []
        async for update in files.follow_file(
            cast(ClientTransport, transport),
            BACKEND,
            IDENTITY.execution_id,
            IDENTITY.relative_path,
            destination,
            identity=identity,
        ):
            if update.phase == "finished":
                assert reader.final_accepted
            updates.append(update)
        return updates

    updates = asyncio.run(scenario())

    assert destination.read_bytes() == DATA
    assert transport.request == {
        "execution_id": IDENTITY.execution_id,
        "relative_path": IDENTITY.relative_path,
        "identity": IDENTITY._wire() if identity is not None else None,
        "offset": prefix_length,
        "max_bytes": 65_536,
    }
    assert all(update.identity == IDENTITY for update in updates)
    assert updates[0] == files.FileUpdate(IDENTITY, prefix_length, "producing")
    assert updates[-1] == files.FileUpdate(
        IDENTITY, len(DATA), "finished", hashlib.sha256(DATA).hexdigest()
    )
    assert all(update.sha256 is None for update in updates[:-1])
    assert [update.committed_offset for update in updates] == sorted(
        update.committed_offset for update in updates
    )
    assert reader.closed


@pytest.mark.parametrize("resume", [False, True])
def test_abandoned_prefix(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, resume: bool
) -> None:
    _requests(monkeypatch)
    destination = tmp_path / "output.bin"
    prefix_length = 5 if resume else 0
    partial = DATA[:31]
    if resume:
        destination.write_bytes(partial[:prefix_length])
    frames = [
        _metadata(prefix_length),
        _frame("Chunk", offset=prefix_length, data=partial[prefix_length:]),
        _frame("Terminal", phase=Variant("Abandoned"), length=len(partial)),
    ]
    reader = Reader(frames)
    transport = Transport(reader)

    async def scenario() -> list[files.FileUpdate]:
        return [
            update
            async for update in files.follow_file(
                cast(ClientTransport, transport),
                BACKEND,
                IDENTITY.execution_id,
                IDENTITY.relative_path,
                destination,
                identity=IDENTITY if resume else None,
            )
        ]

    updates = asyncio.run(scenario())

    assert destination.read_bytes() == partial
    assert updates[-1] == files.FileUpdate(IDENTITY, len(partial), "abandoned")
    assert all(update.sha256 is None for update in updates)
    assert reader.final_accepted
    assert reader.closed


@pytest.mark.parametrize(
    "issue",
    [
        "metadata_store",
        "metadata_epoch",
        "metadata_file",
        "metadata_execution",
        "metadata_path",
        "metadata_identity_missing",
        "metadata_history_missing",
        "metadata_identity_empty",
        "metadata_offset",
        "metadata_available",
        "metadata_phase",
        "metadata_duplicate",
        "chunk_before_metadata",
        "chunk_offset",
        "chunk_empty",
        "chunk_oversized",
        "terminal_length",
        "terminal_phase",
        "seal_missing",
        "seal_identity_missing",
        "seal_file",
        "seal_store",
        "seal_epoch",
        "seal_position_missing",
        "seal_history_missing",
        "seal_position_store",
        "seal_position_epoch",
        "seal_length",
        "hash_suffix",
        "hash_prefix_changed",
        "hash_format",
        "abandoned_seal",
        "trailing_frame",
        "empty_frame",
        "terminal_missing",
        "final_response_missing",
        "final_status",
    ],
)
def test_invalid_follow(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, issue: str
) -> None:
    _requests(monkeypatch)
    destination = tmp_path / "output.bin"
    prefix_length = 17
    destination.write_bytes(DATA[:prefix_length])
    frames = _frames(prefix_length)
    metadata = frames[0].payload.Metadata.field_0
    terminal = frames[-1].payload.Terminal.field_0
    seal = terminal.sealed
    failure = None

    identity_fields = {
        "metadata_store": (metadata.identity.history, "store"),
        "metadata_epoch": (metadata.identity.history, "epoch"),
        "metadata_file": (metadata.identity, "file_id"),
        "metadata_execution": (metadata.identity, "execution_id"),
        "metadata_path": (metadata.identity, "relative_path"),
        "seal_file": (seal.identity, "file_id"),
        "seal_store": (seal.identity.history, "store"),
        "seal_epoch": (seal.identity.history, "epoch"),
        "seal_position_store": (seal.position.history, "store"),
        "seal_position_epoch": (seal.position.history, "epoch"),
    }
    if issue in identity_fields:
        target, field = identity_fields[issue]
        setattr(target, field, "different")
    elif issue == "metadata_identity_missing":
        metadata.identity = None
    elif issue == "metadata_history_missing":
        metadata.identity.history = None
    elif issue == "metadata_identity_empty":
        metadata.identity.file_id = ""
    elif issue == "metadata_offset":
        metadata.offset += 1
    elif issue == "metadata_available":
        metadata.available_length = prefix_length - 1
    elif issue == "metadata_phase":
        metadata.phase = Variant("Unspecified")
    elif issue == "metadata_duplicate":
        frames.insert(1, frames[0])
    elif issue == "chunk_before_metadata":
        frames.pop(0)
    elif issue == "chunk_offset":
        frames[1].payload.Chunk.field_0.offset += 1
    elif issue == "chunk_empty":
        frames[1].payload.Chunk.field_0.data = b""
    elif issue == "chunk_oversized":
        frames[1].payload.Chunk.field_0.data = b"x" * 65_537
    elif issue == "terminal_length":
        terminal.length += 1
    elif issue == "terminal_phase":
        terminal.phase = Variant("Producing")
    elif issue == "seal_missing":
        terminal.sealed = None
    elif issue == "seal_identity_missing":
        seal.identity = None
    elif issue == "seal_position_missing":
        seal.position = None
    elif issue == "seal_history_missing":
        seal.position.history = None
    elif issue == "seal_length":
        seal.length += 1
    elif issue == "hash_suffix":
        seal.sha256 = hashlib.sha256(DATA[prefix_length:]).hexdigest()
    elif issue == "hash_prefix_changed":
        destination.write_bytes(b"x" * prefix_length)
    elif issue == "hash_format":
        seal.sha256 = seal.sha256.upper()
    elif issue == "abandoned_seal":
        terminal.phase = Variant("Abandoned")
    elif issue == "trailing_frame":
        frames.append(_frame("Chunk", offset=len(DATA), data=b"trailing"))
    elif issue == "empty_frame":
        frames.insert(1, WireValue())
    elif issue == "terminal_missing":
        frames.pop()
    elif issue == "final_response_missing":
        failure = ProtocolError("operation stream ended without its final response")
    elif issue == "final_status":
        failure = RpcError("DATA_LOSS", "failed after the terminal frame")
    else:
        raise AssertionError(f"unknown scenario: {issue}")

    reader = Reader(frames, failure=failure)
    transport = Transport(reader)
    updates = []

    async def scenario() -> None:
        async for update in files.follow_file(
            cast(ClientTransport, transport),
            BACKEND,
            IDENTITY.execution_id,
            IDENTITY.relative_path,
            destination,
            identity=IDENTITY,
        ):
            updates.append(update)

    expected_error = RpcError if issue == "final_status" else ProtocolError
    with pytest.raises(expected_error):
        asyncio.run(scenario())

    assert reader.closed
    assert not any(update.phase in {"finished", "abandoned"} for update in updates)
    committed_offset = updates[-1].committed_offset if updates else prefix_length
    expected = DATA[:committed_offset]
    if issue == "hash_prefix_changed":
        expected = b"x" * prefix_length + DATA[prefix_length:committed_offset]
    assert destination.read_bytes() == expected


def test_cancel_resume(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    _requests(monkeypatch)
    destination = tmp_path / "output.bin"
    partial = DATA[:23]
    reader = Reader(
        [
            _metadata(0),
            _frame("Chunk", offset=0, data=partial),
        ],
        stall=True,
    )
    transport = Transport(reader)
    updates = []

    async def scenario() -> None:
        async def consume() -> None:
            async for update in files.follow_file(
                cast(ClientTransport, transport),
                BACKEND,
                IDENTITY.execution_id,
                IDENTITY.relative_path,
                destination,
            ):
                updates.append(update)

        task = asyncio.create_task(consume())
        try:
            await asyncio.wait_for(reader.waiting.wait(), timeout=5)
            assert destination.read_bytes() == partial
        finally:
            task.cancel()
            with pytest.raises(asyncio.CancelledError):
                await task

        assert reader.closed
        assert not reader.final_accepted
        assert updates[-1] == files.FileUpdate(IDENTITY, len(partial), "producing")
        assert destination.read_bytes() == partial

        resumed_reader = Reader(_frames(len(partial)))
        resumed_transport = Transport(resumed_reader)
        resumed = [
            update
            async for update in files.follow_file(
                cast(ClientTransport, resumed_transport),
                BACKEND,
                IDENTITY.execution_id,
                IDENTITY.relative_path,
                destination,
                identity=updates[-1].identity,
            )
        ]
        assert resumed_transport.request is not None
        assert resumed_transport.request["offset"] == len(partial)
        assert resumed_transport.request["identity"] == IDENTITY._wire()
        assert resumed[-1] == files.FileUpdate(
            IDENTITY, len(DATA), "finished", hashlib.sha256(DATA).hexdigest()
        )
        assert resumed_reader.closed

    asyncio.run(scenario())
    assert destination.read_bytes() == DATA


# === Private Functions === #


def _requests(monkeypatch: pytest.MonkeyPatch) -> None:
    # Keep Source Scenarios Independent Of Pending Generated Catalog Exports.
    def request(service: str, method: str, **fields: Any) -> dict[str, Any]:
        assert (service, method) == ("Content", "FollowExecutionFile")
        return fields

    monkeypatch.setattr(files._wire, "request", request)


def _identity() -> WireValue:
    return WireValue(
        history=WireValue(store=IDENTITY.store, epoch=IDENTITY.epoch),
        file_id=IDENTITY.file_id,
        execution_id=IDENTITY.execution_id,
        relative_path=IDENTITY.relative_path,
    )


def _frame(branch: str, **fields: Any) -> WireValue:
    return WireValue(payload=Variant(branch, WireValue(**fields)))


def _metadata(offset: int) -> WireValue:
    return _frame(
        "Metadata",
        identity=_identity(),
        phase=Variant("Producing"),
        available_length=len(DATA),
        offset=offset,
    )


def _frames(offset: int) -> list[WireValue]:
    frames = [_metadata(offset)]
    for start in range(offset, len(DATA), 65_536):
        frames.append(_frame("Chunk", offset=start, data=DATA[start : start + 65_536]))
    seal = WireValue(
        identity=_identity(),
        position=WireValue(
            history=WireValue(store=IDENTITY.store, epoch=IDENTITY.epoch),
            sequence=41,
        ),
        length=len(DATA),
        sha256=hashlib.sha256(DATA).hexdigest(),
    )
    frames.append(
        _frame(
            "Terminal",
            phase=Variant("Finished"),
            length=len(DATA),
            sealed=seal,
        )
    )
    return frames


# === Entry Point === #
