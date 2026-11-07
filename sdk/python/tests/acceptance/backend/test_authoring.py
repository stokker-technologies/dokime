# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Exercise Native Python Authoring"""

# === Imports === #

# Standard Library Imports
import asyncio
import threading
import time
import warnings
from collections.abc import Callable
from pathlib import Path
from typing import Any

# Local Imports
import dokime as dk

# Third Party Imports
import numpy as np
import pyarrow as pa
import pytest
from dokime.client import Execution

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_authoring_callbacks_and_columns(tmp_path: Path) -> None:
    async def story() -> None:
        loop_thread = threading.get_ident()
        completed = asyncio.Event()
        cancelled = asyncio.Event()
        started = asyncio.Event()
        template_tick = asyncio.Event()
        child_started = asyncio.Event()
        child_cancelled = asyncio.Event()
        observed: dict[str, Any] = {}

        @dk.section(label="Acquisition")
        class Acquisition:
            gain: float = dk.parameter(default=2.0, min=1.0, max=10.0)

        @dk.settings
        class Settings:
            acquisition: Acquisition

        @dk.operation(id="python.double", response=float, policy=dk.OperationPolicy(5.0, 5.0), replay="idempotent")
        class Double:
            value: float

            def run(self, ctx: dk.Context) -> float:
                observed["worker_thread"] = threading.get_ident()
                if ctx.execution is not None:
                    observed["child_execution"] = ctx.execution
                return self.value * ctx.settings.acquisition.gain

        @dk.operation(id="external.increment", response=int, visibility="public", policy=dk.OperationPolicy(5.0, 5.0))
        class Increment:
            value: int

            async def run(self, ctx: dk.Context) -> int:
                return self.value + 1

        @dk.operation(id="python.fail", response=int, policy=dk.OperationPolicy(5.0, 5.0))
        class Fail:
            async def run(self, ctx: dk.Context) -> int:
                raise ArithmeticError("callback failure sentinel")

        @dk.operation(id="python.slow", response=int, policy=dk.OperationPolicy(5.0, 5.0, 0.1))
        class Slow:
            async def run(self, ctx: dk.Context) -> int:
                started.set()
                if ctx.execution is not None:
                    child_started.set()
                try:
                    await asyncio.sleep(30)
                finally:
                    cancelled.set()
                    if ctx.execution is not None:
                        child_cancelled.set()
                return 1

        @dk.stream(id="python.samples", label="Samples", window=10000)
        class Reading:
            counter: int
            value: float

        @dk.stream(id="python.tags", label="Tags", window=100)
        class Tags:
            labels: list[str]

        @dk.stream(id="python.blobs", label="Blobs", window=100)
        class Blob:
            payload: bytes

        @dk.observer(id="poll", interval_ms=10)
        async def poll(ctx: dk.Context) -> None:
            if completed.is_set():
                return
            observed["loop_thread"] = threading.get_ident()
            try:
                observed["answer"] = await ctx.operation(Double(3.0)).retry(dk.RetryPolicy())
                observed["external"] = await ctx.operation(Increment(4))
            except (RuntimeError, ValueError) as error:
                observed["error"] = str(error)
                completed.set()
                return
            try:
                with pytest.raises(RuntimeError, match="replay"):
                    await ctx.operation(Fail()).retry(dk.RetryPolicy(max_retries=1))
                await ctx.operation(Fail())
            except RuntimeError as error:
                observed["traceback"] = str(error)
            slow = await ctx.operation(Slow()).start()
            await asyncio.wait_for(started.wait(), 5)
            slow.request_cancel()
            try:
                await slow.join()
            except (RuntimeError, asyncio.CancelledError):
                pass
            completed.set()

        @dk.parameters
        class Parameters:
            count: int = dk.parameter(default=3, min=1, max=10)

        @dk.record(id="python.result", label="Result")
        class Result:
            value: float = dk.parameter(unit="V")

        @dk.prompt(id="python.confirm", label="Confirm")
        class Confirm:
            accepted: bool

        @dk.template(
            id="python.capture.run",
            label="Capture",
            parameters=Parameters,
            requires=[Double, Slow],
            streams=[Reading, Tags, Blob],
            prompts=[Confirm],
        )
        class Capture:
            def initialize(self, ctx: dk.Context) -> dict[str, int]:
                return {"ticks": 0}

            @staticmethod
            @dk.observer(interval_ms=10)
            async def sample(ctx: dk.Context) -> None:
                ctx.resources["ticks"] += 1
                template_tick.set()

            @dk.step()
            async def acquire(self, ctx: dk.Context) -> None:
                assert await ctx.operation(Double(4.0)) == 8.0
                assert ctx.parameters.count == 3
                await asyncio.wait_for(template_tick.wait(), 5)
                await ctx.publish(Result(8.0))
                writer = ctx.stream(Reading).open()
                with pytest.raises(ValueError, match="unequal"):
                    await writer.push_columns({"counter": [1, 2], "value": [1.0]})
                with pytest.raises(ValueError, match="required"):
                    await writer.push_columns({"counter": pa.array([0]), "value": pa.array([None], type=pa.float64())})
                await writer.push_columns({"counter": np.arange(3, dtype=np.int64), "value": np.arange(3, dtype=np.float64)})
                await writer.push_columns({"counter": pa.array([3, 4]), "value": pa.array([3.0, 4.0])})
                await ctx.stream(Tags).open().push(Tags(["first", "second"]))
                await ctx.stream(Blob).open().push_columns({"payload": pa.array([b"\x00\xff", b"\x10"], type=pa.binary())})
                self.unjoined = await ctx.operation(Slow()).start()
                await asyncio.wait_for(child_started.wait(), 5)

            @staticmethod
            @dk.criterion()
            def accepted(ctx: dk.Context) -> bool:
                return len(ctx.evidence()["samples"]) == 8

        @dk.suite(id="python.capture", label="Capture", templates=[Capture])
        class CaptureSuite:
            pass

        plugin = dk.Plugin(
            "python",
            "0.1.0",
            catalog=[Double, Fail, Slow, poll, Reading, Tags, Blob, Settings, CaptureSuite, Result, Confirm],
            requires=[Increment],
        )
        external = dk.Plugin("external", "0.1.0", catalog=[Increment])
        async with dk.Backend(port=0, application_root=tmp_path, plugins=[external, plugin]) as backend:
            await asyncio.wait_for(completed.wait(), 15)
            assert "error" not in observed, observed
            await asyncio.wait_for(cancelled.wait(), 5)
            assert observed["answer"] == 6.0
            assert observed["external"] == 5
            assert observed["loop_thread"] == loop_thread
            assert observed["worker_thread"] != loop_thread
            assert "Traceback" in observed["traceback"]
            assert "ArithmeticError: callback failure sentinel" in observed["traceback"]
            async with backend.client() as client:
                suite = await client.suite("python.capture")
                template = await client.template(suite.templates[0].id)
                (case,) = await template.materialize()
                handle = Execution(case)
                cycle = await client.create_cycle(label="Python Capture", continue_automatically=False)
                await cycle.append([handle])
                await handle.start()
                outcome = await handle.wait(timeout=15)
                records = [record async for record in handle.records()]
                assert outcome.exit_code == 0, "\n".join(record.to_json() for record in records)
                samples = [sample async for sample in handle.samples("python.samples")]
                assert [sample.values["counter"] for sample in samples] == list(range(5))
                tags = [sample async for sample in handle.samples("python.tags")]
                assert [sample.values["labels"] for sample in tags] == [["first", "second"]]
                blobs = [sample async for sample in handle.samples("python.blobs")]
                assert [sample.values["payload"] for sample in blobs] == [[0, 255], [16]]
                assert observed["child_execution"] == handle.id
                assert child_cancelled.is_set()

    asyncio.run(story())


def test_authoring_registration_refusal() -> None:
    @dk.operation(id="invalid.request", response=int, policy=dk.OperationPolicy(1.0, 1.0))
    class Invalid:
        value: object

        def run(self, ctx: dk.Context) -> int:
            return 1

    with pytest.raises(dk.RegistrationError, match="unsupported portable"):
        dk.Plugin("invalid", "0.1.0", catalog=[Invalid])


def test_authoring_ignored_worker_cancellation(tmp_path: Path) -> None:
    async def story() -> None:
        loop = asyncio.get_running_loop()
        entered = asyncio.Event()
        exited = asyncio.Event()
        finished = asyncio.Event()
        released = threading.Event()

        @dk.operation(id="worker.wait", response=int, policy=dk.OperationPolicy(5.0, 5.0, 0.01))
        class Wait:
            def run(self, ctx: dk.Context) -> int:
                loop.call_soon_threadsafe(entered.set)
                released.wait()
                assert ctx.cancelled
                loop.call_soon_threadsafe(exited.set)
                return 1

        @dk.observer(id="cancel", interval_ms=10)
        async def cancel(ctx: dk.Context) -> None:
            if finished.is_set():
                return
            worker = await ctx.operation(Wait()).start()
            await entered.wait()
            worker.request_cancel()
            try:
                await worker.join()
            except RuntimeError:
                pass
            finished.set()

        plugin = dk.Plugin("worker", "0.1.0", catalog=[Wait, cancel])
        try:
            async with dk.Backend(port=0, application_root=tmp_path, plugins=[plugin]):
                with warnings.catch_warnings(record=True) as captured:
                    warnings.simplefilter("always", RuntimeWarning)
                    await asyncio.wait_for(finished.wait(), 5)
                    async with asyncio.timeout(5):
                        while not any("ignored cancellation" in str(item.message) for item in captured):
                            await asyncio.sleep(0)
                    assert len([item for item in captured if "ignored cancellation" in str(item.message)]) == 1
                released.set()
                await asyncio.wait_for(exited.wait(), 5)
        finally:
            released.set()

    asyncio.run(story())


def test_authoring_batch_throughput(tmp_path: Path, record_testsuite_property: Callable[[str, object], None]) -> None:
    async def story() -> None:
        completed = asyncio.Event()
        rates: dict[str, float] = {}
        rows = 10000
        batches = {
            "numpy": {"counter": np.arange(1000, dtype=np.int64), "value": np.arange(1000, dtype=np.float64)},
            "arrow": {"counter": pa.array(range(1000)), "value": pa.array([float(index) for index in range(1000)])},
        }

        @dk.stream(id="batch.samples", label="Samples", window=10000)
        class Reading:
            counter: int
            value: float

        @dk.observer(id="publish", interval_ms=100, streams=[Reading])
        async def publish(ctx: dk.Context) -> None:
            if completed.is_set():
                return
            writer = ctx.stream(Reading).open()
            for name, columns in batches.items():
                started = time.perf_counter()
                for _ in range(rows // 1000):
                    await writer.push_columns(columns)
                rates[name] = rows / (time.perf_counter() - started)
            completed.set()

        plugin = dk.Plugin("batch", "0.1.0", catalog=[Reading, publish])
        async with dk.Backend(port=0, application_root=tmp_path, plugins=[plugin]):
            await asyncio.wait_for(completed.wait(), 30)
        print(f"AUTHORING_BATCH rows={rows} batch=1000 numpy_rows_s={rates['numpy']:.0f} arrow_rows_s={rates['arrow']:.0f}")
        for name, rate in rates.items():
            record_testsuite_property(f"authoring_{name}_rows_s", round(rate))

    asyncio.run(story())


# === Private Functions === #

# === Entry Point === #
