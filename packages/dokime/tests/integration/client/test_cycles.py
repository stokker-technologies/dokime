# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Plan, Admit And Recover Test Cycles"""

# === Imports === #

# Standard Library Imports
import asyncio

# Third Party Imports
import pytest

# Local Imports
from dokime.client import (
    BackendChanged,
    RpcError,
    RunPhase,
    RunRejected,
    SubmissionUncertain,
    _wire,
)

from .backend import (
    CASE_ID,
    CYCLE_ID,
    EXECUTION_ID,
    REQUEST_ID,
    SUITE_ID,
    TEMPLATE,
    collect,
    serve,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_explicit_cycle() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            # Create Without Planning
            cycle_id = await client.create_cycle(label="Bench Run", description="Line 2", continue_on_failure=True)
            assert cycle_id == CYCLE_ID
            assert server.created == 1 and server.append_calls == 0
            request = server.cycle_request
            assert request.metadata.label == "Bench Run"
            assert request.metadata.description == "Line 2"
            assert request.continue_on_failure

            # Append Planned Cases
            (execution,) = await client.append_cases(cycle_id, suite=SUITE_ID, template=TEMPLATE, parameters={"flag": True})
            assert execution.id == EXECUTION_ID
            assert server.created == server.append_calls == 1
            assert _wire.decode(server.overrides["flag"].values[0]) is True

            with pytest.raises(ValueError):
                await client.create_cycle(label=" ")
            assert server.created == 1 and server.run_calls == 0

    asyncio.run(scenario())


def test_cycle_seal_waits() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            cycle = await client.cycle(CYCLE_ID)
            server.seal_allowed.clear()
            sealing = asyncio.create_task(cycle.seal())
            await server.seal_entered.wait()
            assert not sealing.done(), "sealing waits for the durable reply"
            assert server.run_calls == server.stop_calls == 0
            server.seal_allowed.set()
            await sealing
            await cycle.seal()
            assert server.seal_calls == [CYCLE_ID, CYCLE_ID]
            assert _wire.enum_name((await cycle._snapshot()).phase) == "sealed"
            assert server.run_calls == server.stop_calls == 0

    asyncio.run(scenario())


@pytest.mark.parametrize("method", ["CreateCycle", "AppendPlannedCases", "GetCycle"])
def test_cycle_replay(method: str) -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            if method == "GetCycle":
                cycle_id = await client.create_cycle(label="Replay")
                server.restart_during = method
                with pytest.raises(BackendChanged):
                    await client.cycle(cycle_id)
            else:
                server.restart_during = method
                with pytest.raises(BackendChanged):
                    await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            assert server.created == 1
            assert server.append_calls == (1 if method == "AppendPlannedCases" else 0)
            assert server.run_calls == 0

    asyncio.run(scenario())


def test_ambiguous_cycle() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.fail_admission = True
            with pytest.raises(SubmissionUncertain) as caught:
                cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
                await cycle.run(request_key="durable-key")
            assert caught.value.cycle_id == CYCLE_ID

            run_cycle = await client.cycle(caught.value.cycle_id)
            run = await run_cycle.run(request_key=caught.value.request_key)
            recovered = await client.get_run_request(run.id)
            assert recovered.id == run.id
            assert server.created == 1 and server.run_calls == 2
            assert (await run.wait()).phase is RunPhase.COMPLETED

    asyncio.run(scenario())


def test_run_rejected() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.deny_run = True
            with pytest.raises(RunRejected, match="provider_unavailable"):
                cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
                await cycle.run(request_key="key")

    asyncio.run(scenario())


def test_unavailable_method() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.methods["Execution"].remove("RunCycle")
            with pytest.raises(RpcError, match="not available"):
                cycle = await client.cycle(CYCLE_ID)
                await cycle.run(request_key="key")
            assert server.run_calls == 0

    asyncio.run(scenario())


def test_invalid_run_requests() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            for key in ["", " ", "bad\nkey", "x" * 257]:
                with pytest.raises(ValueError):
                    await cycle.run(request_key=key)
            with pytest.raises(ValueError):
                await client.new_cycle(suite=SUITE_ID, template=TEMPLATE, parameters={"voltage": 2**64})
            assert server.created == 1
            assert server.append_calls == 1
            assert server.run_calls == 0

    asyncio.run(scenario())


def test_rerun_admission() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            run_cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            await run_cycle.schedule_rerun(CASE_ID)
            run = await run_cycle.run_case(CASE_ID, request_key="rerun-1")
            assert run.id == REQUEST_ID
            assert run.cycle_id == CYCLE_ID
            assert server.rerun_calls == [CASE_ID]
            assert server.case_runs == [f"{CASE_ID}:rerun-1"]

            with pytest.raises(ValueError):
                await run_cycle.run_case(CASE_ID, request_key=" ")
            with pytest.raises(ValueError):
                await run_cycle.schedule_rerun("")
            assert server.rerun_calls == [CASE_ID]

    asyncio.run(scenario())


def test_restart_receipts() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.finish_gate.clear()
            run_cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            run = await run_cycle.run(request_key="key")

            # A Restarted Runtime Keeps Receipts But Refuses Live Reads
            server.epoch = "epoch-2"
            assert (await run.status()).id == run.id
            with pytest.raises(BackendChanged):
                await collect(run_cycle.samples(until=run))
            with pytest.raises(BackendChanged):
                await collect(run_cycle.records(until=run))
            control = await client.execution(EXECUTION_ID)
            with pytest.raises(BackendChanged):
                await control.stop()

            # Another Installation Refuses Receipts Too
            server.backend_id = "different-installation"
            with pytest.raises(BackendChanged):
                await run.status()

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
