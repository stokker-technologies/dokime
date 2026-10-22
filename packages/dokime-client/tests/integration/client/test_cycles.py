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
from dokime_client.client import (
    BackendChanged,
    Execution,
    ExecutionSubmissionUncertain,
    RpcError,
    RunRejected,
    _wire,
)

from .backend import (
    CASE_ID,
    CYCLE_ID,
    EXECUTION_ID,
    SUITE_ID,
    TEMPLATE,
    collect,
    prepare,
    serve,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_materialized_attempts() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            template = await client.template(TEMPLATE)
            (case,) = await template.materialize(parameters={"flag": True, "nested": {"values": [1, 2]}})
            assert case.id == CASE_ID and case.template_id
            assert case.parameters == {"flag": True, "nested": {"values": [1, 2]}}
            copy = case.parameters
            nested = copy["nested"]
            assert isinstance(nested, dict)
            values = nested["values"]
            assert isinstance(values, list)
            values.append(3)
            assert case.parameters["nested"] == {"values": [1, 2]}, "the materialized case stays immutable"

            executions = [Execution(case), Execution(case)]
            assert executions[0].id != executions[1].id
            assert server.materialize_calls == 1
            assert server.created == server.append_calls == server.run_calls == 0
            with pytest.raises(ValueError, match="append this execution"):
                await executions[0].start()
            assert server.run_calls == 0

            cycle = await client.create_cycle(label="Explicit Attempts")
            assert cycle.id == CYCLE_ID and cycle.executions == ()
            await cycle.append(executions)
            await cycle.append(executions)
            assert cycle.executions == tuple(executions)
            assert cycle.executions[0] is executions[0]
            assert server.appended_ids == [execution.id for execution in executions]
            assert server.run_calls == 0
            with pytest.raises(ValueError, match="repeats an execution"):
                await cycle.append([executions[0], executions[0]])
            assert server.append_calls == 2

    asyncio.run(scenario())


def test_explicit_cycle() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            # Create Without Planning
            cycle = await client.create_cycle(label="Bench Run", description="Line 2", continue_automatically=True)
            assert cycle.id == CYCLE_ID and cycle.executions == ()
            assert server.created == 1 and server.append_calls == 0
            request = server.cycle_request
            assert request.metadata.label == "Bench Run"
            assert request.metadata.description == "Line 2"
            assert request.continue_automatically

            # Append Planned Cases
            template = await client.template(TEMPLATE)
            (case,) = await template.materialize(parameters={"flag": True})
            execution = Execution(case)
            await cycle.append([execution])
            assert cycle.executions == (execution,)
            assert server.created == server.append_calls == 1
            assert _wire.decode(server.overrides["flag"].values[0]) is True

            with pytest.raises(ValueError):
                await client.create_cycle(label=" ")
            assert server.created == 1 and server.run_calls == 0

    asyncio.run(scenario())


def test_append_recovery() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            template = await client.template(TEMPLATE)
            (case,) = await template.materialize()
            execution = Execution(case)
            cycle = await client.cycle(CYCLE_ID)
            server.append_allowed.clear()
            appending = asyncio.create_task(cycle.append([execution]))
            await server.append_entered.wait()
            assert execution._cycle_id is None
            assert server.appended_ids == [execution.id]

            appending.cancel()
            with pytest.raises(asyncio.CancelledError):
                await appending
            assert execution._cycle_id is None
            assert server.run_calls == 0

            # Retry The Exact Objects After A Lost Registration Reply
            server.append_allowed.set()
            await cycle.append([execution])
            assert execution._cycle_id == cycle.id
            assert cycle.executions[-1] is execution
            assert server.appended_ids == [execution.id]
            assert server.append_calls == 2
            assert server.run_calls == 0

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


@pytest.mark.parametrize("method", ["CreateCycle", "AppendExecutions", "GetCycle"])
def test_cycle_replay(method: str) -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            if method == "GetCycle":
                cycle = await client.create_cycle(label="Replay")
                server.restart_during = method
                with pytest.raises(BackendChanged):
                    await client.cycle(cycle.id)
            else:
                server.restart_during = method
                with pytest.raises(BackendChanged):
                    await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            assert server.created == 1
            assert server.append_calls == (1 if method == "AppendExecutions" else 0)
            assert server.run_calls == 0

    asyncio.run(scenario())


def test_admission_recovery() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.fail_admission = True
            with pytest.raises(ExecutionSubmissionUncertain) as caught:
                cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
                await cycle.executions[0].start()
            assert caught.value.execution_id == EXECUTION_ID

            recovered = await client.execution(caught.value.execution_id)
            await recovered.start(mode=caught.value.mode)
            assert server.created == 1 and server.run_calls == 2
            assert (await recovered.wait()).status == "passed"

    asyncio.run(scenario())


def test_start_rejected() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.deny_run = True
            with pytest.raises(RunRejected, match="provider_unavailable"):
                cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
                await cycle.executions[0].start()

    asyncio.run(scenario())


def test_unavailable_method() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.methods["Execution"].remove("StartExecution")
            with pytest.raises(RpcError, match="not available"):
                cycle = await client.cycle(CYCLE_ID)
                await cycle.executions[0].start()
            assert server.run_calls == 0

    asyncio.run(scenario())


def test_start_validation() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            for mode in ["", " ", "unknown"]:
                with pytest.raises(ValueError):
                    await cycle.executions[0].start(mode=mode)
            with pytest.raises(ValueError):
                await prepare(client, suite=SUITE_ID, template=TEMPLATE, parameters={"voltage": 2**64})
            assert server.created == 1
            assert server.append_calls == 1
            assert server.run_calls == 0

    asyncio.run(scenario())


def test_rerun_registration() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            template = await client.template(TEMPLATE)
            (case,) = await template.materialize()
            rerun = Execution(case)
            await run_cycle.append([rerun])
            assert rerun.id != run_cycle.executions[0].id
            assert run_cycle.executions[-1] is rerun
            assert server.run_calls == 0

    asyncio.run(scenario())


def test_restart_boundaries() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.finish_gate.clear()
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            run = run_cycle.executions[0]
            await run.start()

            # A Restarted Runtime Refuses The Old Connection's Live Reads
            server.epoch = "epoch-2"
            assert run.id == EXECUTION_ID
            with pytest.raises(BackendChanged):
                await collect(run.samples())
            with pytest.raises(BackendChanged):
                await collect(run.records())
            control = await client.execution(EXECUTION_ID)
            with pytest.raises(BackendChanged):
                await control.stop()

            # Another Installation Refuses Saved Checkpoints Too
            server.backend_id = "different-installation"
            with pytest.raises(BackendChanged):
                await run.checkpoint()

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
