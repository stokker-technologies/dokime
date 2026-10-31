# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Owned Python Client Lifecycle, Terminal Evidence And Saved Manifests"""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
import json
from contextlib import nullcontext
from dataclasses import replace
from datetime import timedelta, timezone
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock
from uuid import UUID

# Third Party Imports
import pytest

# Local Imports
from dokime_client import DokimeClient
from dokime_client.client import Cycle, Execution, ExecutionOutcome, ProtocolError, ResultSummary
from dokime_client.client.history import Checkpoint, StorageCursor, StoreHistory
from dokime_client.client.models import Backend, DatetimeValue
from dokime_client.client.planning import TestCase as MaterializedCase

from ...integration.client.backend import CYCLE_ID, EXECUTION_ID, TEMPLATE, BackendFixture
from ...integration.client.canonical import _message
from ...integration.client.test_exports import BACKEND, DATA, IDENTITY, ExportTransport, Reader, _frames

# Type Checking Imports

# === Public Exports === #

# === Constants === #

BACKEND_ID = Backend("fixture", "epoch-1", "0.6.0", "Fixture", frozenset())
CURSOR = StorageCursor(StoreHistory("fixture", "history"), 1)
POINT = Checkpoint(EXECUTION_ID, CURSOR, b"version", CURSOR)
PASSED = ExecutionOutcome(POINT, "passed", ResultSummary("passed", 1, 0, 0, 0))

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize(
    "status,code",
    [
        ("passed", 0),
        ("failed", 1),
        ("errored", 1),
        ("invalidated", 1),
        ("blocked", 1),
        ("skipped", 1),
        ("cancelled", 130),
        ("interrupted", 130),
    ],
)
def test_lifecycle_exit_codes(status: str, code: int) -> None:
    assert replace(PASSED, status=status).exit_code == code
    if status == "passed":
        assert replace(PASSED, result=replace(PASSED.result, status="failed")).exit_code == 1


@pytest.mark.parametrize("failure", [None, ValueError("body failed"), asyncio.CancelledError(), KeyboardInterrupt()])
def test_lifecycle_seal_every_exit(failure: BaseException | None, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND_ID)
        cycle = Cycle(CYCLE_ID, client._transport, BACKEND_ID, (execution,), client=client)
        created = AsyncMock(return_value=cycle)
        sealed = AsyncMock()
        waited = AsyncMock(return_value=PASSED)
        monkeypatch.setattr(client, "create_cycle", created)
        monkeypatch.setattr(cycle, "seal", sealed)
        monkeypatch.setattr(Execution, "wait", waited)
        expected = pytest.raises(type(failure)) if failure is not None else nullcontext()
        with expected as caught:
            async with client.new_cycle("Campaign", description="Lab") as owned:
                assert owned is cycle
                if failure is not None:
                    raise failure
        if failure is not None:
            assert caught is not None
            assert caught.value is failure
        created.assert_awaited_once_with(label="Campaign", description="Lab", identifier=None, continue_automatically=False)
        sealed.assert_awaited_once_with()
        waited.assert_awaited_once_with()
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("body_failure", [None, ValueError("body"), asyncio.CancelledError(), KeyboardInterrupt()])
def test_lifecycle_cleanup_failure_preserved(body_failure: BaseException | None, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        cycle = Cycle(CYCLE_ID, client._transport, BACKEND_ID, (), client=client)
        cleanup_failure = OSError("seal failed")
        monkeypatch.setattr(cycle, "seal", AsyncMock(side_effect=cleanup_failure))
        expected = BaseExceptionGroup if body_failure is not None else OSError
        with pytest.raises(expected) as caught:
            async with cycle:
                if body_failure is not None:
                    raise body_failure
        if isinstance(caught.value, BaseExceptionGroup):
            assert caught.value.exceptions == (body_failure, cleanup_failure)
        else:
            assert caught.value is cleanup_failure
        await client.close()

    asyncio.run(scenario())


def test_lifecycle_repeated_cancellation_drains(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        cycle = Cycle(CYCLE_ID, client._transport, BACKEND_ID, ())
        entered, sealing, release, finished = (asyncio.Event() for _ in range(4))

        async def seal() -> None:
            sealing.set()
            await release.wait()
            finished.set()

        monkeypatch.setattr(cycle, "seal", seal)

        async def work() -> None:
            async with cycle:
                entered.set()
                await asyncio.Event().wait()

        task = asyncio.create_task(work())
        await entered.wait()
        task.cancel()
        await sealing.wait()
        task.cancel()
        await asyncio.sleep(0)
        assert not task.done()
        release.set()
        with pytest.raises(asyncio.CancelledError):
            await task
        assert finished.is_set()
        await client.close()

    asyncio.run(scenario())


@pytest.mark.parametrize("count", [1, 3])
def test_lifecycle_start_shortcut(count: int, monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        client = DokimeClient()
        transport = client._transport
        cases = tuple(
            MaterializedCase(str(UUID(int=n + 1)), "suite", "template", "Case", "a" * 64, "provider", b"", transport, BACKEND_ID)
            for n in range(count)
        )
        materialize = AsyncMock(return_value=cases)
        lookup = AsyncMock(return_value=SimpleNamespace(materialize=materialize))
        calls = []

        async def unary(service, method, request):
            calls.append(method)
            if method == "AppendExecutions":
                return _message(
                    service,
                    method,
                    execution_ids=[item.execution_id for item in request.executions],
                    observed_cursor={"epoch": BACKEND_ID.epoch},
                )
            return _message(
                service, method, outcome={"Accepted": {"field_0": {"command_id": str(UUID(int=9)), "cursor": {"epoch": BACKEND_ID.epoch}}}}
            )

        monkeypatch.setattr(client, "template", lookup)
        monkeypatch.setattr(transport, "verify", AsyncMock())
        monkeypatch.setattr(transport, "unary", unary)
        cycle = Cycle(CYCLE_ID, transport, BACKEND_ID, (), client=client)
        started = await cycle.start(TEMPLATE, duration_min=10)
        executions = (started,) if isinstance(started, Execution) else started
        assert len(executions) == count
        assert cycle.executions == executions
        assert len({item.id for item in executions}) == count
        assert [item._case for item in executions] == list(cases)
        assert calls == ["AppendExecutions", *(["StartExecution"] * count)]
        lookup.assert_awaited_once_with(TEMPLATE)
        materialize.assert_awaited_once_with(parameters={"duration_min": 10})
        await client.close()

    asyncio.run(scenario())


def test_lifecycle_iterators_end_and_serialize(monkeypatch: pytest.MonkeyPatch) -> None:
    async def scenario() -> None:
        fixture = BackendFixture()
        fixture.publish("finished")
        client = DokimeClient()
        execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND_ID)
        monkeypatch.setattr(Execution, "_snapshot", AsyncMock())
        point = Checkpoint._from_wire(fixture.point, 4096)
        monkeypatch.setattr(Execution, "checkpoint", AsyncMock(return_value=point))
        document = _message(
            "History",
            "GetCheckpointDocument",
            checkpoint=fixture.point,
            acquisition_finished=True,
            document={
                "execution": {"execution_id": EXECUTION_ID, "status": {"Passed": {}}, "result": {"status": {"Passed": {}}, "passed": 1}}
            },
        )
        monkeypatch.setattr(Execution, "_document_at", AsyncMock(return_value=document))

        async def records(_transport, _backend, _scope, types, completion):
            assert types == ("fixture.*",)
            await completion
            yield SimpleNamespace(records=fixture.records)

        async def samples(_transport, _backend, _scope, streams, page_size, follow):
            assert streams == ("voltage",) and follow
            yield SimpleNamespace(samples=fixture.samples)

        monkeypatch.setattr("dokime_client.client.execution.handle.read_records", records)
        monkeypatch.setattr("dokime_client.client.execution.handle.read_samples", samples)
        records_seen = [item async for item in execution.records("fixture.*")]
        samples_seen = [item async for item in execution.samples("voltage")]
        assert len(records_seen) == len(samples_seen) == 1
        assert execution.outcome.status == "passed"
        assert await execution == execution.outcome
        date = DatetimeValue.from_rfc3339("2026-10-30T10:20:30.123456789+00:00")
        record = replace(records_seen[0], timestamp=date, details={"nested": [date, 2**64 - 1]})
        sample = replace(samples_seen[0], values={"at": date})
        assert json.loads(record.to_json())["details"]["nested"] == [date._wire_text, 2**64 - 1]
        assert json.loads(sample.to_json())["values"] == {"at": date._wire_text}
        shifted = (date + timedelta(seconds=1)).astimezone(timezone(timedelta(hours=5, minutes=30)))
        expected_time = "2026-10-30T15:50:31.123456789+05:30"
        record = replace(record, timestamp=shifted, details={"at": shifted})
        sample = replace(sample, values={"at": shifted})
        assert json.loads(record.to_json())["timestamp"] == expected_time
        assert json.loads(record.to_json())["details"] == {"at": expected_time}
        assert json.loads(sample.to_json())["values"] == {"at": expected_time}
        await client.close()

    asyncio.run(scenario())


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


def test_lifecycle_pending_outcome() -> None:
    client = DokimeClient()
    execution = Execution._bind(EXECUTION_ID, client._transport, BACKEND_ID)
    with pytest.raises(RuntimeError, match="await the execution"):
        _ = execution.outcome


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

# === Entry Point === #
