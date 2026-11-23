# SPDX-License-Identifier: Apache-2.0

"""Observer State Follows Library Sampling Demand Across Real Carrier Frames."""

# === Imports === #

from unittest import IsolatedAsyncioTestCase

from google.protobuf.wrappers_pb2 import Int64Value
from grpc.aio import insecure_channel
from harmos._columns import columns_type
from harmos._wire.plugin.v1.instance_pb2 import InstanceSamplesRequest, InstanceSamplesResponse

from dokime_sdk import Plugin
from dokime_sdk.operations import Context
from dokime_sdk.templates import Observer

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ObserverLifecycleStory(IsolatedAsyncioTestCase):
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

        # Retain State Across Windows, Then Release and Reacquire Demand.
        columns = columns_type(Int64Value, "counter.counter")
        for sequence, expected in enumerate(([1, 2], [3, 4], [1, 2])):
            if sequence == 2:
                await call.write(InstanceSamplesRequest(demand={}))
                await call.write(InstanceSamplesRequest(demand={"streams": ["counter.counter"]}))
            await call.write(InstanceSamplesRequest(window={"start": {}, "period": {"seconds": 1}, "count": 2, "sequence": sequence}))
            response = await call.read()
            self.assertIsInstance(response, InstanceSamplesResponse)
            self.assertEqual(list(columns.FromString(response.batch.columns).value), expected)
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
