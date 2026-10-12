# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Exercise Python authoring through the real backend and gRPC client lifecycle."""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable
import asyncio
from contextlib import asynccontextmanager
from dataclasses import dataclass
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
import dokime
from dokime.client._generated.dokime.v1 import execution_pb2, history_pb2

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize(
    "scenario",
    [
        "complete",
        "cancel",
        "reload",
        "step_exception",
        "invalid_step",
        "invalid_criterion",
        "partial_resources",
        "cleanup_wait",
        "initialization_cancel",
        "resource_cancel",
    ],
)
def test_python_lifecycle(tmp_path: Path, scenario: str) -> None:
    async def run_scenario() -> None:
        events: list[str] = []
        started = asyncio.Event()
        finalizing = asyncio.Event()
        release = asyncio.Event()
        drain = asyncio.Event()

        async def pending_callback(stage: str) -> None:
            events.append(f"{stage}_started")
            started.set()
            try:
                await drain.wait()
            finally:
                events.append(f"{stage}_finalizing")
                finalizing.set()
                await release.wait()
                events.append(f"{stage}_joined")

        @dokime.record(id="python.observation")
        class Observation:
            label: str

        @dataclass(frozen=True)
        class Position:
            axis: str
            coordinate: float

        @dokime.stream(id="python.samples")
        class Sample:
            value: float
            unsigned: dokime.UnsignedValue
            optional: int | None
            values: list[float]
            position: Position

        class Probe(dokime.TestCaseTemplate):
            template_id = "python.probe"
            records = (Observation,)
            streams = (Sample,)

            @dokime.initialize
            async def prepare(self, ctx):
                if scenario == "initialization_cancel":
                    await pending_callback("initialization")
                return await ctx.acquire(self._managed_resources(ctx))

            @asynccontextmanager
            async def _managed_resources(self, ctx):
                events.append("resources_opening")
                try:
                    if scenario == "partial_resources":
                        raise RuntimeError("resource initialization failed")
                    if scenario == "resource_cancel":
                        await pending_callback("resources")
                    yield "device"
                finally:
                    events.append("resources_closed")

            @dokime.step
            async def measure(self, ctx):
                assert ctx.resources == "device"
                assert ctx.plugin == "provider"
                await ctx.info("Python step entered through the shared engine")
                await ctx.publish(Observation("measured"))
                await ctx.publish(
                    Sample(
                        1.5,
                        dokime.UnsignedValue(2**64 - 1),
                        None,
                        [2.0, 3.0],
                        Position("x", 4.0),
                    )
                )
                if scenario == "complete":
                    started.set()
                    await release.wait()
                if scenario in {"cancel", "reload"}:
                    await pending_callback("step")
                if scenario == "step_exception":
                    raise RuntimeError("step failed")
                if scenario == "invalid_step":
                    return True

            @dokime.cleanup
            async def finish(self, ctx):
                assert ctx.resources == "device"
                events.append(f"cleanup_{ctx.reason}")
                if scenario == "cleanup_wait":
                    await pending_callback("cleanup")

            @staticmethod
            @dokime.criterion
            async def accepted(ctx):
                assert "resources_closed" in events
                if scenario == "invalid_criterion":
                    return None
                assert await ctx.records(Observation) == [Observation("measured")]
                assert await ctx.samples(Sample) == [
                    Sample(
                        1.5,
                        dokime.UnsignedValue(2**64 - 1),
                        None,
                        [2.0, 3.0],
                        Position("x", 4.0),
                    )
                ]
                return True

        class Suite(dokime.TestSuite):
            suite_id = "python.checks"
            members = (Probe,)

            @asynccontextmanager
            async def resources(self):
                events.append("provider_opened")
                try:
                    yield "provider"
                finally:
                    events.append("provider_closed")

        root = tmp_path / "application"
        runtime = dokime.Runtime(application_root=root)
        assert not root.exists(), "construction must not start the backend"
        try:
            await runtime.register(Suite)
            assert events == [], "registration before start must be inert"
            assert not root.exists()
            client = await runtime.start()
            assert events == ["provider_opened"]
            with pytest.raises(RuntimeError, match="already started"):
                await runtime.start()
            assert runtime.client is client
            assert [suite.identifier async for suite in client.suites()] == [
                "python.checks"
            ]
            execution_cycle = await client.new_cycle(
                suite="python.checks", template="python.probe"
            )
            (control,) = execution_cycle.executions
            execution = await control.start(request_key=scenario)
            await execution.status()
            if scenario == "complete":
                await started.wait()
                record_feed = execution_cycle.records(
                    record_types=["python.observation"], until=execution
                )
                sample_feed = execution_cycle.samples(
                    streams=["python.samples"], until=execution
                )
                try:
                    record = await anext(record_feed)
                    sample = await anext(sample_feed)
                    assert record.details == {"label": "measured"}
                    assert sample.values["value"] == 1.5
                    assert sample.values["unsigned"] == 2**64 - 1
                    assert isinstance(sample.values["unsigned"], dokime.UnsignedValue)
                    assert sample.values["values"] == [2.0, 3.0]
                    assert sample.values["position"] == {"axis": "x", "coordinate": 4.0}
                    assert not (await execution.status()).terminal
                    selected = [
                        event
                        async for event in execution_cycle.records(
                            record_ids=[record.id]
                        )
                    ]
                    assert [event.id for event in selected] == [record.id]
                finally:
                    await record_feed.aclose()
                    await sample_feed.aclose()
                    release.set()
            replacing = None
            cancelled = scenario in {
                "cancel",
                "initialization_cancel",
                "resource_cancel",
            }
            if cancelled or scenario in {"cleanup_wait", "reload"}:
                await started.wait()
                if scenario == "reload":
                    replacing = asyncio.create_task(runtime.register())
                    await asyncio.sleep(0.05)
                    assert not replacing.done()
                    assert "provider_closed" not in events
                    drain.set()
                elif scenario == "cleanup_wait":
                    drain.set()
                elif cancelled:
                    progress = await execution.status()
                    assert progress.last_execution_id is not None
                    control = await client.execution(progress.last_execution_id)
                    await control.stop()
                await finalizing.wait()
                assert not (await execution.status()).terminal
                assert "provider_closed" not in events
                if scenario not in {"initialization_cancel", "resource_cancel"}:
                    assert "resources_closed" not in events
                release.set()
                if replacing is not None:
                    await replacing
                    assert [suite async for suite in client.suites()] == []

            status = await execution.wait(timeout=15)
            assert status.terminal
            assert status.last_execution_id is not None
            detail = await client._transport.unary(
                "Execution",
                "GetExecution",
                execution_pb2.GetExecutionRequest(
                    execution_id=status.last_execution_id
                ),
            )
            admitted = detail.admitted_plugin
            assert admitted.source == "python"
            assert admitted.WhichOneof("implementation") == "generation"
            assert admitted.generation
            generation = admitted.generation
            summary = await execution_cycle.result()
            expected = (
                "passed"
                if scenario in {"complete", "reload", "cleanup_wait"}
                else "cancelled"
                if cancelled
                else "failed"
            )
            if summary.status != expected:
                for event in [event async for event in execution_cycle.records()]:
                    print(event)
            assert summary.status == expected
            if scenario in {
                "partial_resources",
                "resource_cancel",
                "initialization_cancel",
            }:
                assert not any(event.startswith("cleanup_") for event in events)
            if scenario != "initialization_cancel":
                assert "resources_closed" in events
            if scenario in {"cancel", "reload"}:
                cleanup = "cleanup_cancelled" if cancelled else "cleanup_completed"
                assert events.index("step_joined") < events.index(cleanup)
                assert events.index(cleanup) < events.index("resources_closed")
        finally:
            release.set()
            await runtime.close()
            await runtime.close()

        assert events.count("provider_closed") == 1
        if "resources_closed" in events:
            assert events.index("resources_closed") < events.index("provider_closed")
        reopened = dokime.Runtime(application_root=root)
        try:
            client = await reopened.start()
            assert [suite async for suite in client.suites()] == [], (
                "Python callables cannot survive restart"
            )
            history = await client._transport.unary(
                "History",
                "BrowseExecutions",
                history_pb2.BrowseExecutionsRequest(limit=10),
            )
            entry = next(
                item
                for item in history.entries
                if item.metadata.execution_id == status.last_execution_id
            )
            historical = await client._transport.unary(
                "History",
                "GetCheckpointDocument",
                history_pb2.GetCheckpointDocumentRequest(checkpoint=entry.checkpoint),
            )
            stored_identity = historical.document.execution.snapshot.plugin
            assert stored_identity.generation == generation
            assert stored_identity.WhichOneof("implementation") == "generation"
        finally:
            await reopened.close()

    asyncio.run(asyncio.wait_for(run_scenario(), timeout=45))


def case_sampler_lifetime(
    tmp_path: Path,
) -> None:
    async def run() -> None:
        release_step = asyncio.Event()
        resources_closed = asyncio.Event()
        measured = []

        @dokime.stream(id="python.sampled")
        class Reading:
            index: int
            value: float

        class Probe(dokime.TestCaseTemplate):
            template_id = "python.sampler"

            @dokime.initialize
            async def initialize_handles(self, ctx):
                return await ctx.acquire(self._managed_resources(ctx))

            @asynccontextmanager
            async def _managed_resources(self, ctx):
                try:
                    yield 12.5
                finally:
                    resources_closed.set()

            @dokime.sampler(stream=Reading, interval_ms=10)
            async def measure(self, resources, tick):
                assert not resources_closed.is_set()
                measured.append(tick.index)
                return Reading(tick.index, resources)

            @dokime.step
            async def acquire(self, ctx):
                await release_step.wait()

            @staticmethod
            @dokime.criterion
            async def sampled(ctx):
                assert resources_closed.is_set()
                rows = await ctx.samples(Reading)
                return bool(rows) and all(row.value == 12.5 for row in rows)

        class Suite(dokime.TestSuite):
            suite_id = "python.sampling"
            members = (Probe,)

        runtime = dokime.Runtime(application_root=tmp_path)
        try:
            await runtime.register(Suite)
            client = await runtime.start()
            execution_cycle = await client.new_cycle(
                suite="python.sampling", template="python.sampler"
            )
            execution = await execution_cycle.run(request_key="sampler")
            feed = execution_cycle.samples(streams=["python.sampled"], until=execution)
            try:
                first = await anext(feed)
                assert first.values["value"] == 12.5
                assert not resources_closed.is_set()
            finally:
                await feed.aclose()
                release_step.set()
            await execution.wait(timeout=10)
            assert (await execution_cycle.result()).status == "passed"
            assert measured and measured == sorted(set(measured))
            assert resources_closed.is_set()
        finally:
            release_step.set()
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=25))


def case_python_lease(tmp_path: Path) -> None:
    from datetime import datetime
    from dokime import host

    async def run() -> None:
        release = asyncio.Event()
        observed = []

        class Probe(dokime.TestCaseTemplate):
            template_id = "python.host_probe"
            subscribes = (host.System,)

            @dokime.step
            async def acquire(self, ctx):
                await release.wait()

            @staticmethod
            @dokime.criterion
            async def measured_host(ctx):
                rows = await ctx.samples(host.System)
                observed.extend(rows)
                return bool(rows) and all(
                    isinstance(row.timestamp, datetime) for row in rows
                )

        class Suite(dokime.TestSuite):
            suite_id = "python.host_checks"
            members = (Probe,)

        runtime = dokime.Runtime(application_root=tmp_path)
        try:
            await runtime.register(Suite)
            client = await runtime.start()
            execution_cycle = await client.new_cycle(
                suite="python.host_checks", template="python.host_probe"
            )
            execution = await execution_cycle.run(request_key="host")
            feed = execution_cycle.samples(streams=["host.system"], until=execution)
            try:
                sample = await anext(feed)
                assert sample.stream == "host.system"
            finally:
                await feed.aclose()
                release.set()
            await execution.wait(timeout=10)
            assert (await execution_cycle.result()).status == "passed", [
                (event.type, event.message, event.error)
                async for event in execution_cycle.records()
                if event.severity == "error"
            ]
            assert observed and all(isinstance(row, host.System) for row in observed)
        finally:
            release.set()
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=30))


@pytest.mark.parametrize("cancel", [False, True])
def test_python_release(tmp_path: Path, cancel: bool) -> None:
    from dokime import host

    async def run() -> None:
        listening = asyncio.Event()
        finalizers = []
        received = []

        @dokime.stream(id="python.silent")
        class Silent:
            value: float

        class Probe(dokime.TestCaseTemplate):
            template_id = "python.lease_probe"
            streams = (Silent,)

            @dokime.step
            async def acquire(self, ctx):
                # A timed-out next must relinquish its waiter and allow close.
                silent = await ctx.listen(Silent)
                try:
                    with pytest.raises(TimeoutError):
                        await asyncio.wait_for(anext(silent), timeout=0.02)
                    pending = asyncio.create_task(anext(silent))
                    await ctx.info("A listener can close while a next call is pending")
                    await silent.close()
                    with pytest.raises(StopAsyncIteration):
                        await pending
                finally:
                    await silent.close()
                    await silent.close()
                lease = await ctx.lease(host.System)
                rows = await lease.listen()
                try:
                    listening.set()
                    received.append(await anext(rows))
                    if cancel:
                        await asyncio.Event().wait()
                finally:
                    await rows.close()
                    await lease.close()
                    await lease.close()
                    finalizers.append("released")

            @staticmethod
            @dokime.criterion
            async def has_host_evidence(ctx):
                return bool(await ctx.samples(host.System))

        class Suite(dokime.TestSuite):
            suite_id = "python.lease_checks"
            members = (Probe,)

        runtime = dokime.Runtime(application_root=tmp_path)
        try:
            await runtime.register(Suite)
            client = await runtime.start()
            execution_cycle = await client.new_cycle(
                suite="python.lease_checks", template="python.lease_probe"
            )
            (control,) = execution_cycle.executions
            execution = await control.start(request_key="lease")
            await listening.wait()
            if cancel:
                progress = await execution.status()
                assert progress.last_execution_id is not None
                control = await client.execution(progress.last_execution_id)
                await control.stop()
            await execution.wait(timeout=15)
            expected = "cancelled" if cancel else "passed"
            assert (await execution_cycle.result()).status == expected, [
                event.message
                async for event in execution_cycle.records()
                if event.severity == "error"
            ]
            assert finalizers == ["released"]
            if not cancel:
                assert received and isinstance(received[0], host.System)
        finally:
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=30))


@pytest.mark.parametrize("invalid", [False, True])
def test_traceability_values(tmp_path: Path, invalid: bool) -> None:
    from datetime import datetime, timezone

    from dokime.client import _wire
    from dokime.client._generated.dokime.v1 import common_pb2, observation_pb2

    async def run() -> None:
        stepped = []
        observed_at = datetime(2026, 9, 28, tzinfo=timezone.utc)
        values = {
            "serial": "SN-42",
            "signed": -42,
            "unsigned": dokime.UnsignedValue(2**64 - 1),
            "reading": 1.25,
            "ready": True,
            "observed": observed_at,
            "nested": {"streams": [1, 2]},
        }

        class Probe(dokime.TestCaseTemplate):
            template_id = "trace.probe"

            async def traceability(self, ctx):
                return [
                    dokime.TraceabilitySection(
                        "invalid section" if invalid else "device",
                        "Device",
                        [
                            dokime.TraceabilityRow(
                                name, name, value, source="instrument"
                            )
                            for name, value in values.items()
                        ],
                    )
                ]

            @dokime.step
            async def measure(self, ctx):
                stepped.append(True)

        class Lab(dokime.TestSuite):
            suite_id = "trace.lab"
            members = (Probe,)

        runtime = dokime.Runtime(application_root=tmp_path)
        try:
            await runtime.register(Lab)
            client = await runtime.start()
            execution_cycle = await client.new_cycle(
                suite="trace.lab", template="trace.probe"
            )
            execution = await execution_cycle.run(request_key="trace")
            status = await execution.wait(timeout=15)
            assert (await execution_cycle.result()).status == (
                "failed" if invalid else "passed"
            )
            assert stepped == ([] if invalid else [True])
            page = await client._transport.unary(
                "Observation",
                "GetTraceability",
                observation_pb2.GetTraceabilityRequest(
                    scope=common_pb2.EvidenceScope(
                        execution_id=status.last_execution_id
                    ),
                    limit=100,
                ),
            )
            assert page.complete
            stated = [
                item for item in page.snapshots if item.metadata.label == "Device"
            ]
            if invalid:
                assert not stated
            else:
                assert len(stated) == 1
                assert stated[0].scope.execution_id == status.last_execution_id
                assert stated[0].created_at.seconds > 0
                assert {
                    entry.metadata.label: _wire.decode(entry.value)
                    for entry in stated[0].entries
                } == values
                assert all(
                    entry.source.endswith("instrument") for entry in stated[0].entries
                )
        finally:
            await runtime.close()

    asyncio.run(run())


# === Private Functions === #

# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_sampler_lifetime, id="sampler_lifetime"),
        pytest.param(case_python_lease, id="python_lease"),
    ],
)
def test_contract(scenario: Callable[..., None], tmp_path: Path) -> None:
    scenario(tmp_path)
