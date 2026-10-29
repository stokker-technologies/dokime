# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Follow And Replay Live Records And Telemetry"""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import runpy
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock
from uuid import UUID

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import Cycle, Execution, _wire
from dokime_client.client.execution import handle as execution_module

from .backend import EXECUTION_ID, SUITE_ID, TEMPLATE, collect, prepare, serve

# Type Checking Imports

# === Public Exports === #

# === Constants === #

EXAMPLE = Path(__file__).resolve().parents[3] / "examples/remote/python/control-test.py"

# === Types === #

# === Classes === #

# === Public Functions === #


def test_live_values() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.finish_gate.clear()

            # Planned Parameters Keep Their Wire Values
            suites = [s async for s in client.suites()]
            assert suites[0].templates[0].id == TEMPLATE
            run_cycle = await prepare(
                client,
                suite=SUITE_ID,
                template=TEMPLATE,
                parameters={"integer": 2**60, "flag": True, "array": [1, None]},
            )
            run = run_cycle.executions[0]
            await run.start()
            assert server.overrides is not None
            assert _wire.decode(server.overrides["integer"].values[0], 64) == 2**60
            assert _wire.decode(server.overrides["flag"].values[0], 64) is True
            assert _wire.decode(server.overrides["array"].values[0], 64) == [1, None]

            # Live Readers Follow Until The Run Completes
            following = asyncio.gather(
                collect(run.records(follow=True)),
                collect(run.samples(page_size=1, follow=True)),
                run.wait(poll_interval=0.01),
            )
            await server.records_parked.wait()
            server.finish_gate.set()
            events, samples, status = await following
            assert [r.id for r in events] == ["event-1", "event-2"]
            assert [s.values["voltage"] for s in samples] == [231.0, 232.0]
            assert status.status == "passed"
            summary = await run_cycle.result()
            assert summary.passed == 1
            assert server.stop_calls == 0
            assert any(p.get("voltage") == 2 for p in server.sent_cursors)

    asyncio.run(scenario())


def test_sample_watch_reconnect() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            execution = cycle.executions[0]
            await execution.start()
            await execution.wait(poll_interval=0.01)
            server.interrupt_sample_watch = True
            samples = await collect(execution.samples(follow=True, page_size=1))
            assert [sample.sequence for sample in samples] == [1, 2]
            assert any(cursor.get("voltage") == 1 for cursor in server.sent_cursors)
            assert server.stop_calls == 0

    asyncio.run(scenario())


def test_replay_pages() -> None:
    async def scenario() -> None:
        async with serve() as (_, client):
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            run = run_cycle.executions[0]
            await run.start()
            await run.wait(poll_interval=0.01)
            records = await collect(run_cycle.records(record_types=["fixture.finished"], record_ids=["event-2"]))
            assert [r.id for r in records] == ["event-2"]
            assert await collect(run.records(record_ids=["missing"], follow=True)) == []
            assert len(await collect(run_cycle.samples(page_size=1))) == 2

    asyncio.run(scenario())


def test_example_listeners_drain(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]) -> None:
    run = runpy.run_path(str(EXAMPLE))["run"]
    monkeypatch.chdir(tmp_path)

    async def scenario() -> None:
        async with serve() as (server, client):
            server.sample_page_size = 1
            server.finish_gate.clear()
            for key, value in {
                "DokimeClient": SimpleNamespace(from_pairing=lambda _path: client),
                "TEMPLATE": TEMPLATE,
                "OUTPUT_DIRECTORY": tmp_path,
                "EXECUTION_COUNT": 1,
                "RECORD_TYPES": ("fixture.fin*",),
                "STREAMS": ("volt?ge",),
            }.items():
                monkeypatch.setitem(run.__globals__, key, value)
            monkeypatch.setattr(execution_module, "uuid4", lambda: UUID(EXECUTION_ID))
            saved = SimpleNamespace(path=tmp_path / "bundle.zip", sha256="fixture-checksum")
            execution_export = AsyncMock(return_value=saved)
            cycle_export = AsyncMock(return_value=saved)
            monkeypatch.setattr(Execution, "export", execution_export)
            monkeypatch.setattr(Cycle, "export", cycle_export)

            # The Example Reads Live Before The Run Finishes
            polling = asyncio.create_task(run())
            await server.records_parked.wait()
            server.finish_gate.set()
            assert await polling == 0
            output = capsys.readouterr().out
            assert output.count("Sample: voltage #") == 2
            assert output.count("Record: fixture.finished #") == 1
            assert f"Saved Outcome: passed Execution: {EXECUTION_ID}" in output
            cycle_file = tmp_path / "cycle.json"
            execution_file = tmp_path / "execution-1.json"
            cycle_response = json.loads(cycle_file.read_text())
            execution_response = json.loads(execution_file.read_text())
            assert cycle_response["cycle_id"]
            assert execution_response == {
                "execution_id": EXECUTION_ID,
                "status": "passed",
                "records_by_type": {"fixture.finished": 1},
                "samples_by_stream": {"voltage": 2},
            }
            execution_export.assert_awaited_once_with(tmp_path / "execution-1.zip", overwrite=False)
            cycle_export.assert_awaited_once_with(tmp_path / "cycle.zip", overwrite=False)
            assert any(position.get("voltage") == 1 for position in server.sent_cursors)
            assert server.stop_calls == 0

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
