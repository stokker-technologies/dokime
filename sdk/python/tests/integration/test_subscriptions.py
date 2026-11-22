# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Scoped Live Demand Through the Real Product Host."""

# === Imports === #

# Standard Library Imports
import asyncio
from collections.abc import AsyncIterator
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path
from typing import cast
from unittest import IsolatedAsyncioTestCase

# Third Party Imports
from google.protobuf.timestamp_pb2 import Timestamp
from google.protobuf.wrappers_pb2 import Int64Value
from harmos import plugin

# Local Imports
from dokime_sdk import Gap, Plugin, StreamRow, Suite
from dokime_sdk.operations import Context, StreamSubscription
from dokime_sdk.templates import Stream, TestTemplate, step, stream

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class SubscriptionTemplate(TestTemplate):
    @step("Hold Subscription Scope")
    async def hold(self, ctx: Context) -> None:
        owner = cast(SubscriptionPlugin, ctx.plugin)
        declaration = cast(Stream[Int64Value], vars(SubscriptionPlugin.readings)["__dokime_stream__"])
        owner.slow = await ctx.subscribe(declaration, max_item_bytes=4096)
        owner.fast = await ctx.subscribe(declaration, max_item_bytes=4096)
        owner.opened.set()
        await owner.finish.wait()


class SubscriptionSuite(Suite):
    templates = (SubscriptionTemplate,)


class SubscriptionPlugin(Plugin):
    version = "1.0.0"
    suites = (SubscriptionSuite,)

    def __init__(self) -> None:
        self.rows: asyncio.Queue[Int64Value] = asyncio.Queue()
        self.opened = asyncio.Event()
        self.finish = asyncio.Event()
        self.slow: StreamSubscription[Int64Value]
        self.fast: StreamSubscription[Int64Value]
        super().__init__()

    @stream("Readings", row=Int64Value)
    async def readings(self, ctx: Context) -> AsyncIterator[Int64Value]:
        while True:
            yield await self.rows.get()


class SubscriptionStory(IsolatedAsyncioTestCase):
    async def test_live_subscriptions(self) -> None:
        specification = spec_from_file_location("subscription_host", Path(__file__).resolve().parents[1] / "support/host.py")
        assert specification is not None and specification.loader is not None
        module = module_from_spec(specification)
        specification.loader.exec_module(module)
        host = await module.Host.start(self, manual_clock=True)
        owner = SubscriptionPlugin()
        await host.client.plugins.register(owner)
        cycle = await host.client.cycle(label="Live Subscriptions")
        cases = await host.client.cases.materialize(SubscriptionTemplate)
        execution = await cycle.execution(cases[0])
        await execution.start()
        await owner.opened.wait()
        self.addAsyncCleanup(owner.fast.close)
        self.addAsyncCleanup(owner.slow.close)

        # Lag One Subscriber Behind the Live Window
        for value in range(24):
            await owner.rows.put(Int64Value(value=value))
            item = await anext(owner.fast)
            self.assertIsInstance(item, StreamRow)
            assert isinstance(item, StreamRow)
            self.assertIs(type(item), plugin.StreamRow)
            self.assertEqual(item.value, Int64Value(value=value))
            self.assertIsInstance(item.source, Timestamp)
            self.assertIsInstance(item.received, Timestamp)
            self.assertIsNot(item.source, item.received)
        gap = await anext(owner.slow)
        self.assertIs(type(gap), plugin.Gap)
        self.assertEqual(gap, Gap(16))
        retained = await anext(owner.slow)
        assert isinstance(retained, StreamRow)
        self.assertEqual(retained.value, Int64Value(value=16))

        # Close One Demand
        await owner.slow.close()
        with self.assertRaises(StopAsyncIteration):
            await anext(owner.slow)
        await owner.rows.put(Int64Value(value=24))
        continuing = await anext(owner.fast)
        assert isinstance(continuing, StreamRow)
        self.assertEqual(continuing.value, Int64Value(value=24))

        # Release Demand Before Completing the Scope
        await owner.fast.close()
        with self.assertRaises(StopAsyncIteration):
            await anext(owner.fast)
        owner.finish.set()
        async for _ in execution.watch():
            pass
        await execution.seal()
        await cycle.seal()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
