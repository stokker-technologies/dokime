# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Start, Control, Await And Seal Executions"""

# === Imports === #

# Standard Library Imports
import asyncio
from typing import Any

# Third Party Imports
import pytest

# Local Imports
from dokime.client import (
    BackendChanged,
    ClientClosed,
    CommandPhase,
    ExecutionSubmissionUncertain,
    ProtocolError,
    RpcError,
    RunRejected,
    _wire,
)

from .backend import (
    COMMAND_ID,
    CYCLE_ID,
    EXECUTION_ID,
    REQUEST_ID,
    SUITE_ID,
    TEMPLATE,
    collect,
    prepare,
    serve,
)
from .canonical import _message

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Each Mutation Breaks One Invariant Of The Sealed Checkpoint Identity.
MALFORMED_CHECKPOINTS = {
    "version_garbage": ("version", b"malformed"),
    "foreign_execution": ("execution_identifier", "invalid"),
    "zero_sequence": ("version", b"\x0dfixture.store\x0dfixture.epoch\x00"),
    "other_sequence": ("version", b"\x0dfixture.store\x0dfixture.epoch\x08"),
    "other_epoch": ("version", b"\x0dfixture.store\x0dfixture.other\x07"),
    "trailing_byte": ("version", None),
    "overlong_length": ("version", b"\x8d\x00fixture.store\x0dfixture.epoch\x07"),
    "overflow_sequence": (
        "version",
        b"\x0dfixture.store\x0dfixture.epoch" + b"\xff" * 10,
    ),
    "invalid_store": ("store", "invalid/history"),
    "invalid_epoch": ("epoch", "invalid/history"),
    "position_zero": ("sequence", 0),
}

# === Types === #

# === Classes === #

# === Public Functions === #


def test_wait_awaits_seal() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.seal_automatically = False
            cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            (handle,) = cycle.executions
            await handle.start()
            await server.terminal.wait()
            assert await handle.checkpoint() is None

            # A Terminal But Unsealed Execution Keeps The Wait Pending
            waiting = asyncio.create_task(handle.wait(poll_interval=0.005))
            server.checkpoint_missed.clear()
            await server.checkpoint_missed.wait()
            assert not waiting.done()
            assert server.stop_calls == 0

            server.sealed.set()
            outcome = await waiting
            assert outcome.status == "passed"
            assert outcome.result.passed == 1
            assert _wire.equal(outcome.checkpoint._wire(), server.point)

    asyncio.run(scenario())


def test_run_drains_callbacks() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            (handle,) = cycle.executions
            entered = asyncio.Event()
            release = asyncio.Event()
            records = []
            samples = []

            async def on_record(item: Any) -> None:
                entered.set()
                await release.wait()
                records.append(item.id)

            async def on_sample(item: Any) -> None:
                samples.append(item.id)

            running = asyncio.create_task(
                handle.run(
                    on_record=on_record,
                    on_sample=on_sample,
                    record_types=("missing.*", "fixture.*"),
                    streams=("missing.*", "volt?ge"),
                    poll_interval=0.005,
                )
            )
            try:
                # The Outcome Is Sealed While A Callback Still Holds Delivery
                await entered.wait()
                outcome = await handle.wait(poll_interval=0.005)
                assert outcome.status == "passed"
                assert not running.done()

                release.set()
                assert (await running) == outcome
                assert records == ["event-1", "event-2"]
                assert samples == ["sample-1", "sample-2"]
                assert server.stop_calls == 0
            finally:
                release.set()
                running.cancel()
                await asyncio.gather(running, return_exceptions=True)

    asyncio.run(scenario())


@pytest.mark.parametrize("pattern", ["", "fixture.[ab]", "fixture.{start,stop}", "^fixture.*$", "fixture/.*", "fixture.é"])
def test_invalid_listener_prevents_start(pattern: str) -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            with pytest.raises(ValueError, match="identifier globs"):
                await handle.run(record_types=(pattern,))
            with pytest.raises(ValueError, match="identifier globs"):
                await handle.run(streams=(pattern,))
            assert server.run_calls == 0
            assert server.created == 0

    asyncio.run(scenario())


def test_callback_failure_isolated() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            (handle,) = cycle.executions

            async def on_record(item: Any) -> None:
                raise RuntimeError("callback failed")

            with pytest.raises(ExceptionGroup) as failed:
                await handle.run(
                    on_record=on_record,
                    poll_interval=0.005,
                )
            assert any(isinstance(error, RuntimeError) for error in failed.value.exceptions)
            assert server.stop_calls == 0

            outcome = await handle.wait(poll_interval=0.005)
            assert outcome.status == "passed"
            assert server.stop_calls == 0

    asyncio.run(scenario())


@pytest.mark.parametrize("patterns", [("x" * 4097,), ("valid",) * 65, "literal"])
def test_filter_vector_limits(patterns: Any) -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            execution = await client.execution(EXECUTION_ID)
            with pytest.raises(ValueError):
                await execution.run(record_types=patterns)
            with pytest.raises(ValueError):
                await execution.run(streams=patterns)
            assert server.run_calls == 0

    asyncio.run(scenario())


@pytest.mark.parametrize("deadline", [0.0, -1.0, float("nan"), float("inf")])
def test_invalid_deadlines(deadline: float) -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            execution = await client.execution(EXECUTION_ID)
            with pytest.raises(ValueError):
                await execution.run(timeout=deadline)
            with pytest.raises(ValueError):
                await execution.run(poll_interval=deadline)
            assert server.run_calls == 0

    asyncio.run(scenario())


@pytest.mark.parametrize("method", ["pause", "resume", "stop"])
def test_execution_controls(method: str) -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            await getattr(handle, method)()
            assert server.execution_control_calls == [f"{method.title()}Execution"]

    asyncio.run(scenario())


@pytest.mark.parametrize("method", ["pause", "resume", "stop"])
def test_control_restart_refused(method: str) -> None:
    async def scenario() -> None:
        rpc = f"{method.title()}Execution"
        async with serve() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            server.restart_during = rpc
            with pytest.raises(BackendChanged):
                await getattr(handle, method)()
            assert server.execution_control_calls == [rpc]
            assert server.run_calls == 0
            assert server.created == 0

    asyncio.run(scenario())


def test_finished_controls() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            run = run_cycle.executions[0]
            await run.start()
            await run.wait(poll_interval=0.01)

            # Stop Is Idempotent; Pause And Resume Refuse A Finished Execution
            control = await client.execution(EXECUTION_ID)
            await control.stop()
            assert server.stop_calls == 0
            with pytest.raises(RpcError) as pause:
                await control.pause()
            assert pause.value.code == "FAILED_PRECONDITION"
            with pytest.raises(RpcError) as resume:
                await control.resume()
            assert resume.value.code == "FAILED_PRECONDITION"

    asyncio.run(scenario())


def test_stop_evidence() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.finish_gate.clear()
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            run = run_cycle.executions[0]
            await run.start()
            records_task = asyncio.create_task(collect(run.records(follow=True)))
            control = await client.execution(EXECUTION_ID)
            await control.stop()
            status = await run.wait(poll_interval=0.01)
            assert server.stop_calls == 1
            assert status.status == "cancelled"
            assert (await records_task)[-1].type == "fixture.stopped"

    asyncio.run(scenario())


def test_close_keeps_run() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.finish_gate.clear()
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            run = run_cycle.executions[0]
            await run.start()
            assert await run.checkpoint() is None

            # Closing Twice Leaves Remote Work Running To Completion
            await client.close()
            await client.close()
            server.finish_gate.set()
            await server.terminal.wait()
            assert server.stop_calls == 0
            with pytest.raises(ClientClosed):
                await run.checkpoint()

    asyncio.run(scenario())


def test_command_receipt_replay() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            assert handle.id == EXECUTION_ID
            assert server.command_reads == 0

            # A Restarted Runtime Still Answers Durable Command Receipts
            server.epoch = "epoch-2"
            receipt = await handle.command_receipt(REQUEST_ID, COMMAND_ID)
            assert receipt.phase is CommandPhase.INTERRUPTED
            assert receipt.terminal
            assert receipt.execution_id == EXECUTION_ID
            assert server.command_reads == 1

            # A Receipt For Another Execution Is Data Loss
            server.command_execution_id = CYCLE_ID
            with pytest.raises(RpcError) as foreign:
                await handle.command_receipt(REQUEST_ID, COMMAND_ID)
            assert foreign.value.code == "DATA_LOSS"
            assert server.command_reads == 2

            # Another Installation Is Refused Before Any Read
            server.backend_id = "different-installation"
            with pytest.raises(BackendChanged):
                await handle.command_receipt(REQUEST_ID, COMMAND_ID)
            assert server.command_reads == 2
            assert server.run_calls == 0
            assert server.created == 0

    asyncio.run(scenario())


def test_execution_mode_recovery() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            invalid_mode: Any = "unsupported"
            with pytest.raises(ValueError):
                await handle.start(mode=invalid_mode)
            assert server.run_calls == 0

            # An Uncertain Admission Retains Execution Identity And Mode
            server.fail_admission = True
            with pytest.raises(ExecutionSubmissionUncertain) as caught:
                await handle.start(mode="interactive")
            failure = caught.value
            assert failure.execution_id == EXECUTION_ID
            assert failure.mode == "interactive"
            assert failure.__cause__ is not None

            await handle.start(mode=failure.mode)
            assert server.created == 0
            assert server.run_calls == 2
            assert server.execution_mode is not None
            assert _wire.enum_name(server.execution_mode) == "interactive"
            with pytest.raises(RunRejected, match="request_identity_conflict"):
                await handle.start(mode="automatic")
            assert server.run_calls == 3
            assert (await handle.wait()).status == "passed"
            assert len(server.samples) == 2

    asyncio.run(scenario())


def test_execution_observation() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            assert "/dokime.v1.Execution/GetRunRequest" not in handle._backend.methods
            assert not hasattr(client, "get_run_request")
            await handle.start()

            # Completion Uses The Execution Seal, Without A Separate Batch Receipt.
            outcome = await handle.wait(poll_interval=0.01)
            assert outcome.status == "passed"
            assert server.run_calls == 1
            assert server.created == 0

    asyncio.run(scenario())


@pytest.mark.parametrize("mutation", list(MALFORMED_CHECKPOINTS))
def test_malformed_checkpoint(mutation: str) -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            server.sealed.set()
            handle = await client.execution(EXECUTION_ID)
            valid = _message("History", "GetExecutionCheckpoint", direction="response", path="")
            valid.CopyFrom(server.point)

            field, value = MALFORMED_CHECKPOINTS[mutation]
            if field in ("store", "epoch"):
                setattr(server.point.position.history, field, value)
            elif field == "sequence":
                server.point.position.sequence = value
            elif value is None:
                server.point.version = valid.version + b"\x00"
            else:
                setattr(server.point, field, value)
            with pytest.raises(ProtocolError):
                await handle.checkpoint()

            # The Refusal Leaves The Handle Usable Once The Seal Is Valid
            server.point.CopyFrom(valid)
            restored = await handle.checkpoint()
            assert restored is not None and _wire.equal(restored._wire(), valid)

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
