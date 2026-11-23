# SPDX-License-Identifier: Apache-2.0

"""Observer State Follows Library Sampling Demand Across Real Carrier Frames."""

# === Imports === #

import asyncio
import runpy
from pathlib import Path
from unittest import IsolatedAsyncioTestCase

from google.protobuf.wrappers_pb2 import Int64Value
from grpc.aio import insecure_channel
from harmos._columns import columns_type
from harmos._wire.plugin.v1.instance_pb2 import InstanceSamplesRequest, InstanceSamplesResponse

import dokime_sdk.templates
from dokime_sdk import Plugin, Suite
from dokime_sdk.operations import Context
from dokime_sdk.templates import Observer, TestTemplate, record, step

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ObserverLifecycleStory(IsolatedAsyncioTestCase):
    async def test_observer_record_reaches_two_capturing_executions_only(self) -> None:
        event = record(int, identifier="event")
        entered: asyncio.Queue[str] = asyncio.Queue()
        permits: asyncio.Queue[None] = asyncio.Queue()
        published: asyncio.Queue[tuple[str, ...]] = asyncio.Queue()
        release = asyncio.Event()

        class Counter(Observer[int]):
            row = int

            def __init__(self) -> None:
                self.value = 0

            async def sample(self, ctx: Context) -> int:
                await permits.get()
                self.value += 1
                await ctx.record(event, self.value)
                await published.put(ctx._transport.capturing_executions)
                return self.value

        class Capture(TestTemplate):
            uses_streams = (Counter,)

            @step("Hold Capture")
            async def hold(self, ctx: "dokime_sdk.templates.Context") -> None:
                await entered.put(ctx.execution)
                await release.wait()

        class Ignore(Capture):
            uses_streams = ()

        class CounterSuite(Suite):
            templates = (Capture, Ignore)

        class CounterPlugin(Plugin):
            version = "1.0.0"
            suites = (CounterSuite,)
            observers = (Counter,)
            records = (event,)

        # Register the Plugin With a Manually Advanced Product Host.
        support = Path(__file__).resolve().parents[1] / "support/host.py"
        host = await runpy.run_path(str(support))["Host"].start(self, manual_clock=True)
        await host.client.plugins.register(CounterPlugin())
        cycle = await host.client.cycle(label="Observer Records")
        capturing = (await host.client.cases.materialize(Capture))[0]
        ignoring = (await host.client.cases.materialize(Ignore))[0]
        executions = [await cycle.execution(case) for case in (capturing, capturing, ignoring)]
        try:
            # Enter All Three Executions Before Publishing the Measured Window.
            for execution in executions:
                await execution.start()
            self.assertEqual({await entered.get() for _ in executions}, {execution.identifier for execution in executions})
            await host.advance(1_000_000_000)
            await permits.put(None)
            await published.get()
            await host.advance(2_000_000_000)
            await permits.put(None)
            recipients = await published.get()
            self.assertEqual(set(recipients), {execution.identifier for execution in executions[:2]})
        finally:
            release.set()

        # Seal and Inspect the Host's Persisted Record Fan-Out.
        for execution in executions:
            async for _ in execution.watch():
                pass
            await execution.seal()
        for execution in executions[:2]:
            self.assertEqual(await execution.records(event).last(), 2)
        self.assertEqual(await executions[2].records(event).count(), 0)
        await cycle.seal()

    async def test_sampling_period_retains_state_then_closes_and_recreates_it(self) -> None:
        opened: list[Observer[int]] = []
        closed: list[Observer[int]] = []

        class Counter(Observer[int]):
            row = int

            def __init__(self) -> None:
                self.value = 0
                opened.append(self)

            async def sample(self, ctx: Context) -> int:
                self.value += 1
                return self.value

            async def close(self) -> None:
                closed.append(self)

        class CounterPlugin(Plugin):
            version = "1.0.0"
            observers = (Counter,)

        # Open the Library Carrier.
        endpoint = await CounterPlugin()._activate()
        self.addAsyncCleanup(endpoint.stop)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        samples = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Samples",
            request_serializer=InstanceSamplesRequest.SerializeToString,
            response_deserializer=InstanceSamplesResponse.FromString,
        )
        call = samples()
        self.addCleanup(call.cancel)
        await call.write(InstanceSamplesRequest(open={"stream": "counter.counter"}))
        await call.write(InstanceSamplesRequest(demand={"streams": ["counter.counter"]}))

        started = await call.read()
        assert isinstance(started, InstanceSamplesResponse) and started.HasField("demand")

        # Retain State Across Windows, Then Release and Reacquire Demand.
        columns = columns_type(Int64Value, "counter.counter")
        for sequence, expected in enumerate(([1, 2], [3, 4], [1, 2])):
            if sequence == 2:
                await call.write(InstanceSamplesRequest(demand={}))
                stopped = await call.read()
                assert isinstance(stopped, InstanceSamplesResponse) and stopped.HasField("demand")
                await call.write(InstanceSamplesRequest(demand={"streams": ["counter.counter"]}))
                started = await call.read()
                assert isinstance(started, InstanceSamplesResponse) and started.HasField("demand")
            await call.write(InstanceSamplesRequest(window={"start": {}, "period": {"seconds": 1}, "count": 2, "sequence": sequence}))
            response = await call.read()
            assert isinstance(response, InstanceSamplesResponse)
            self.assertEqual(list(getattr(columns.FromString(response.batch.columns), "value")), expected)
        self.assertEqual(len(opened), 2)
        self.assertEqual(closed, opened[:1])
        self.assertIsNot(opened[0], opened[1])

        # Completing the Carrier Releases the Final Period.
        await call.done_writing()
        await call.read()
        self.assertEqual(closed, opened)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
