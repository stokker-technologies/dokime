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
import runpy
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime.client import RunPhase, _wire

from .backend import SUITE_ID, TEMPLATE, collect, serve

# Type Checking Imports

# === Public Exports === #

# === Constants === #

EXAMPLE = Path(__file__).resolve().parents[3] / "examples/client/poll_run.py"

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
            run_cycle = await client.new_cycle(
                suite=SUITE_ID,
                template=TEMPLATE,
                parameters={"integer": 2**60, "flag": True, "array": [1, None]},
            )
            run = await run_cycle.run(request_key="lab-order")
            assert server.overrides is not None
            assert _wire.decode(server.overrides["integer"].values[0]) == 2**60
            assert _wire.decode(server.overrides["flag"].values[0]) is True
            assert _wire.decode(server.overrides["array"].values[0]) == [1, None]

            # Live Readers Follow Until The Run Completes
            following = asyncio.gather(
                collect(run_cycle.records(until=run)),
                collect(run_cycle.samples(page_size=1, poll_interval=0.01, until=run)),
                run.wait(poll_interval=0.01),
            )
            await server.records_parked.wait()
            server.finish_gate.set()
            events, samples, status = await following
            assert [r.id for r in events] == ["event-1", "event-2"]
            assert [s.values["voltage"] for s in samples] == [231.0, 232.0]
            assert status.phase is RunPhase.COMPLETED
            summary = await run_cycle.result()
            assert summary.passed == 1
            assert server.stop_calls == 0
            assert any(p.get("voltage") == 2 for p in server.sent_cursors)

    asyncio.run(scenario())


def test_replay_pages() -> None:
    async def scenario() -> None:
        async with serve() as (_, client):
            run_cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            run = await run_cycle.run(request_key="lab-order")
            await run.wait(poll_interval=0.01)
            records = await collect(run_cycle.records(record_types=["fixture.finished"], record_ids=["event-2"]))
            assert [r.id for r in records] == ["event-2"]
            assert await collect(run_cycle.records(record_ids=["missing"], until=run)) == []
            assert len(await collect(run_cycle.samples(page_size=1))) == 2

    asyncio.run(scenario())


def test_polling_example() -> None:
    poll = runpy.run_path(str(EXAMPLE))["poll"]

    async def scenario() -> None:
        async with serve() as (server, client):
            server.sample_page_size = 1
            server.finish_gate.clear()
            run_cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            (handle,) = run_cycle.executions
            await handle.start(request_key="polling-example")

            # The Example Reads Live Before The Run Finishes
            polling = asyncio.create_task(poll(handle, 0.01))
            await server.records_parked.wait()
            server.finish_gate.set()
            summary = await polling
            assert summary["samples_by_stream"] == {"voltage": 2}
            assert summary["events_by_type"] == {
                "fixture.started": 1,
                "fixture.finished": 1,
            }
            assert summary["run"]["result"]["passed"] == 1
            assert any(position.get("voltage") == 1 for position in server.sent_cursors)
            assert server.stop_calls == 0

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
