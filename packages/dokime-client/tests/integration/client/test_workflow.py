# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify domain ownership, capture recovery, cancellation and bounded progress."""

# === Imports === #

# Standard Library Imports
import asyncio
import json
from pathlib import Path
from typing import Any

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import (
    Capture,
    ExecutionSubmissionUncertain,
    ProtocolError,
    RunPhase,
    RunRequest,
)

from .backend import SUITE_ID, TEMPLATE, collect
from .capture_backend import serve

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


# === Public Functions === #


def test_handles_progress() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            backend.finish_gate.clear()
            cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            request = await cycle.run(request_key="handle-test")
            assert isinstance(request, RunRequest)
            assert not hasattr(request, "cycle")
            assert request.cycle_id == cycle.id
            assert request._transport is cycle._transport
            assert request._backend == cycle._backend
            assert request.id != cycle.id
            # Readers Follow The Run Live, Then Drain After Completion
            following = asyncio.gather(
                collect(cycle.records(until=request)),
                collect(cycle.samples(until=request, page_size=1, poll_interval=0.01)),
                collect(request.updates(poll_interval=0.01)),
            )
            await backend.records_parked.wait()
            backend.finish_gate.set()
            records, samples, progress = await following
            assert len(records) == len(samples) == 2
            assert progress[-1].phase is RunPhase.COMPLETED
            assert backend.stop_calls == 0
            unrelated = await client.cycle("bdda795a-2929-4d27-8c2c-32965d3c2ff3")
            with pytest.raises(ValueError, match="another cycle"):
                await collect(unrelated.records(until=request))
            with pytest.raises(ValueError, match="another cycle"):
                await collect(unrelated.samples(until=request))

    asyncio.run(scenario())


def test_capture_progress(tmp_path: Path, capsys: Any) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            capture = await Capture.prepare(client, output=tmp_path / "capture", suite=SUITE_ID, template=TEMPLATE)
            assert backend.created == 1 and backend.run_calls == 0
            backend.fail_admission = True
            with pytest.raises(ExecutionSubmissionUncertain):
                await capture.start()
            stages = [item.stage for item in await collect(capture.updates())]
            assert stages[0] == "connecting" and stages[-1] == "failed"
            saved_key = backend.key
            recovered = await Capture.open(client, output=capture.output)
            request = await recovered.start()
            assert backend.key == saved_key
            assert backend.created == 1
            result, updates = await asyncio.gather(recovered.collect(request), collect(recovered.updates()))
            assert result.request.id == request.id
            assert result.result.status == "passed"
            assert updates[-1].stage == "complete"
            assert [u.stage for u in updates].count("complete") == 1
            assert recovered.result is result
            assert not list(capture.output.rglob("*.part"))
            late = await collect(recovered.updates())
            assert late[-1].stage == "complete"
            assert not (capture.output / ".collect.lock").exists()

    asyncio.run(scenario())
    assert capsys.readouterr() == ("", "")


def test_failed_retried(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            capture = await Capture.prepare(client, output=tmp_path / "capture", suite=SUITE_ID, template=TEMPLATE)
            request = await capture.start()
            backend.bad_report = "hash"
            updates = asyncio.create_task(collect(capture.updates()))
            with pytest.raises(ProtocolError):
                await capture.collect(request)
            assert (await updates)[-1].stage == "failed"
            with pytest.raises(RuntimeError, match="not completed"):
                _ = capture.result
            assert not json.loads((capture.output / "capture.json").read_text())["complete"]
            backend.bad_report = ""
            await capture.collect(request)
            assert backend.created == backend.run_calls == 1
            assert capture.result.result.status == "passed"

    asyncio.run(scenario())


def test_cancel_keeps_execution(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            _hold_acquisition(backend)
            capture = await Capture.prepare(client, output=tmp_path / "capture", suite=SUITE_ID, template=TEMPLATE)
            request = await capture.start()
            updates = asyncio.create_task(collect(capture.updates()))
            collection = asyncio.create_task(capture.collect(request))
            # Live Files Open Before The Records Watch Starts
            await backend.records_watched.wait()
            assert (capture.output / "records.jsonl.part").exists()
            collection.cancel()
            with pytest.raises(asyncio.CancelledError):
                await collection
            assert backend.stop_calls == 0
            assert not (await request.status()).terminal
            assert (await updates)[-1].stage == "failed"
            assert not (capture.output / ".collect.lock").exists()
            assert not json.loads((capture.output / "capture.json").read_text())["complete"]
            # Stop The Execution And Drain Its Saved Evidence.
            cycle = await client.cycle(request.cycle_id)
            (execution,) = cycle.executions
            await execution.stop()
            recovered = await Capture.open(client, output=capture.output)
            resolved = await recovered.start()
            result = await recovered.collect(resolved)
            assert result.request.phase is RunPhase.STOPPED
            assert backend.created == backend.run_calls == 1

    asyncio.run(scenario())


def test_capture_exclusive(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            _hold_acquisition(backend)
            capture = await Capture.prepare(client, output=tmp_path / "capture", suite=SUITE_ID, template=TEMPLATE)
            request = await capture.start()
            other = await Capture.open(client, output=capture.output)
            other_request = await other.start()
            with pytest.raises(ValueError, match="this capture"):
                await capture.collect(other_request)
            collection = asyncio.create_task(capture.collect(request))
            # Live Files Open Before The Records Watch Starts
            await backend.records_watched.wait()
            assert (capture.output / "records.jsonl.part").exists()
            try:
                with pytest.raises(FileExistsError):
                    await other.collect(other_request)
                with pytest.raises(FileExistsError):
                    await other.start()
                with pytest.raises(RuntimeError, match="already collecting"):
                    await capture.collect(request)
            finally:
                cycle = await client.cycle(request.cycle_id)
                (execution,) = cycle.executions
                await execution.stop()
                await collection

    asyncio.run(scenario())


def test_capture_output_refused(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            existing = tmp_path / "existing"
            existing.mkdir()
            with pytest.raises(FileExistsError):
                await Capture.prepare(client, output=existing, suite=SUITE_ID, template=TEMPLATE)
            assert backend.created == 0
            (existing / "capture.json").write_text(json.dumps({"format_version": 1, "stage": "creating_cycle"}))
            with pytest.raises(ValueError, match="not confirmed"):
                await Capture.open(client, output=existing)
            assert backend.created == backend.run_calls == 0

    asyncio.run(scenario())


# === Private Functions === #


def _hold_acquisition(backend: Any) -> None:
    async def produce() -> None:
        backend.publish("started")
        await asyncio.Event().wait()

    backend.produce = produce


# === Entry Point === #
