# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Exercise Owned Cycle Contexts Through The Existing Scripted Backend."""

# === Imports === #

# Standard Library Imports
import asyncio
from contextlib import nullcontext
from uuid import UUID

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import Execution, _wire

from .backend import CYCLE_ID, EXECUTION_ID, TEMPLATE, BackendFixture, serve

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class LifecycleBackend(BackendFixture):
    async def SealCycle(self, request, context):
        # Model The Existing Native Seal's Stop And Durable Drain Boundary.
        if not self.terminal.is_set():
            await self.StopExecution(_wire.request("Execution", "StopExecution", execution_id=EXECUTION_ID), context)
        return await super().SealCycle(request, context)


# === Public Functions === #


@pytest.mark.parametrize("exit_kind", ["normal", "exception", "cancellation"])
def test_lifecycle_owned_exit(exit_kind: str, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("dokime_client.client.execution.handle.uuid4", lambda: UUID(EXECUTION_ID))

    async def scenario() -> None:
        async with serve(LifecycleBackend()) as (server, client):
            server.finish_gate.clear()
            entered = asyncio.Event()
            handles = []

            async def work() -> None:
                async with client.new_cycle("Owned Campaign") as cycle:
                    run = await cycle.start(TEMPLATE, duration_min=10)
                    assert isinstance(run, Execution)
                    handles.append(run)
                    entered.set()
                    if exit_kind == "exception":
                        raise ValueError("body failure")
                    if exit_kind == "cancellation":
                        await asyncio.Event().wait()

            task = asyncio.create_task(work())
            await entered.wait()
            if exit_kind == "cancellation":
                task.cancel()
            failure = {"exception": ValueError, "cancellation": asyncio.CancelledError}.get(exit_kind)
            expected = pytest.raises(failure) if failure is not None else nullcontext()
            with expected:
                await task
            (run,) = handles
            assert run.outcome.exit_code == 130
            assert run.outcome.status == "cancelled"
            assert server.seal_calls == [CYCLE_ID]
            assert server.stop_calls == 1
            assert server.created == server.materialize_calls == server.append_calls == server.run_calls == 1
            assert [record.type async for record in run.records("fixture.stopped")] == ["fixture.stopped"]
            assert await run == run.outcome

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
