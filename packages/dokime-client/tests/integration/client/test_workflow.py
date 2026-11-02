# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Execution Recovery And Independent Observation"""

# === Imports === #

# Standard Library Imports
import asyncio

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import ExecutionEvidence, ExecutionSubmissionUncertain

from .backend import CYCLE_ID, EXECUTION_ID, collect, prepare
from .capture_backend import serve

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_execution_recovery() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            cycle = await prepare(client)
            execution = cycle.executions[0]
            assert backend.created == 1 and backend.run_calls == 0
            backend.fail_admission = True
            with pytest.raises(ExecutionSubmissionUncertain) as caught:
                await execution.start()
            recovered = await client.execution(caught.value.execution_id)
            await recovered.start(mode=caught.value.mode)
            outcome = await recovered.wait(poll_interval=0.01)
            assert recovered.id == EXECUTION_ID
            assert outcome.status == "passed"
            assert backend.created == 1 and backend.run_calls == 2
            assert len(backend.samples) == 2
            assert backend.stop_calls == 0

    asyncio.run(scenario())


def test_cancel_keeps_execution() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            backend.finish_gate.clear()
            cycle = await prepare(client)
            execution = cycle.executions[0]

            await execution.start()
            running = asyncio.create_task(collect(execution.records()))
            await backend.records_parked.wait()
            running.cancel()
            with pytest.raises(asyncio.CancelledError):
                await running
            assert backend.stop_calls == 0
            assert not backend.terminal.is_set()
            assert await execution.checkpoint() is None

            # Rebinding And Stopping Keeps The Original Attempt And Saved Outcome
            recovered = await client.execution(execution.id)
            await recovered.stop()
            outcome = await recovered.wait()
            assert outcome.status == "cancelled"
            assert backend.created == backend.run_calls == backend.stop_calls == 1
            records = await collect(recovered.records())
            assert records[-1].type == "fixture.stopped"

    asyncio.run(scenario())


def test_independent_observers() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            backend.finish_gate.clear()
            cycle = await prepare(client)
            execution = cycle.executions[0]
            await execution.start()
            first = asyncio.create_task(collect(execution.records()))
            second = asyncio.create_task(collect(execution.records()))
            await backend.records_parked.wait()
            first.cancel()
            with pytest.raises(asyncio.CancelledError):
                await first
            assert backend.stop_calls == 0
            assert not second.done()
            backend.finish_gate.set()
            assert [record.type for record in await second] == ["fixture.finished"]
            assert (await execution.wait()).status == "passed"
            assert backend.run_calls == 1

    asyncio.run(scenario())


def test_captured_record_count() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            backend.finish_gate.clear()
            cycle = await prepare(client)
            execution = cycle.executions[0]
            await execution.start()
            await backend.changed.wait()
            backend.records.clear()
            backend.samples.clear()
            records_task = asyncio.create_task(collect(execution.records("example.target.lost")))
            samples_task = asyncio.create_task(collect(execution.samples()))
            await asyncio.sleep(0)
            for _ in range(3):
                backend.publish("lost")
                record = backend.records[-1]
                record.record_type = "example.target.lost"
                record.ClearField("scope")
                backend.samples[-1].ClearField("scope")
                backend.record_captures[record.record_id] = [(CYCLE_ID, EXECUTION_ID)]
            backend.phase = {"Completed": {}}
            backend.terminal.set()
            backend.sealed.set()

            records = await records_task
            assert len(records) == 3
            assert len(await collect(cycle.records(record_types=["example.target.lost"]))) == 3
            assert len(await samples_task) == 3
            assert len(await collect(cycle.samples())) == 3
            point = await execution.checkpoint()
            assert point is not None
            evidence = ExecutionEvidence(execution._transport, point, execution._backend)
            saved = await collect(evidence.records())
            assert sum(item.record.type == "example.target.lost" for item in saved) == 3
            assert len(await collect(evidence.samples())) == 3

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
