# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Live Subscriptions Delegate Typed Demand Without Copying Library Items."""

# === Imports === #

# Standard Library Imports
from asyncio import Queue, create_task, ensure_future
from collections.abc import AsyncIterator
from typing import cast
from unittest import IsolatedAsyncioTestCase
from unittest.mock import AsyncMock, Mock

# Third Party Imports
from google.protobuf.timestamp_pb2 import Timestamp
from google.protobuf.wrappers_pb2 import Int64Value
from harmos import OperationError, plugin
from harmos._wire.plugin.v1.invocation_pb2 import ChildResponse, Request
from harmos._wire.plugin.v1.subscriptions_pb2 import SubscriptionsSubscribeRequest, SubscriptionsSubscribeResponse

# Local Imports
from dokime_sdk import Gap, Plugin, StreamRow, Suite, templates
from dokime_sdk._generated.host import HostOperation, host_requirement
from dokime_sdk.operations import Context
from dokime_sdk.templates import Stream, TestTemplate, stream

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class SubscriptionTemplate(TestTemplate):
    @stream("Readings", row=Int64Value)
    async def readings(self, ctx: templates.Context) -> AsyncIterator[Int64Value]:
        yield Int64Value(value=1)


class SubscriptionSuite(Suite):
    templates = (SubscriptionTemplate,)


class SubscriptionPlugin(Plugin):
    version = "1.0.0"
    suites = (SubscriptionSuite,)

    @stream("Readings", row=Int64Value)
    async def readings(self, ctx: Context) -> AsyncIterator[Int64Value]:
        yield Int64Value(value=1)


class SubscriptionTests(IsolatedAsyncioTestCase):
    async def test_scoped_rows(self) -> None:
        output = Queue()
        contract = host_requirement(HostOperation.SUBSCRIPTIONS_SUBSCRIBE)
        transport = plugin.Context(Request(scope="execution", attempt=1), "subscription", {contract.id: contract.fingerprint}, output)
        context = Context(transport, SubscriptionPlugin())
        opening = create_task(context.subscribe(Stream("peer.readings", Int64Value), max_item_bytes=4096))
        child = (await output.get()).child
        demand = (await output.get()).child_demand
        self.assertEqual(demand.call, child.call)
        self.assertEqual(child.max_response_bytes, 4096)
        request = SubscriptionsSubscribeRequest.FromString(child.payload)
        self.assertEqual(request.stream, "peer.readings")
        self.assertEqual(request.row, Int64Value.DESCRIPTOR.full_name)
        await transport._reply(ChildResponse(call=child.call, item=SubscriptionsSubscribeResponse(opened=True).SerializeToString()))
        subscription = await opening
        self.addAsyncCleanup(subscription.close)
        reading = ensure_future(anext(subscription))
        await output.get()
        source = Timestamp(seconds=3, nanos=123)
        received = Timestamp(seconds=5, nanos=456)
        response = SubscriptionsSubscribeResponse(
            row={"payload": Int64Value(value=7).SerializeToString(), "source": source, "received": received}
        )
        await transport._reply(ChildResponse(call=child.call, item=response.SerializeToString()))
        self.assertEqual(await reading, StreamRow(Int64Value(value=7), source, received))
        reading = ensure_future(anext(subscription))
        await output.get()
        await transport._cancel("Scope Ended")
        self.assertEqual((await output.get()).child_cancel.call, child.call)
        with self.assertRaises(OperationError) as cancelled:
            await reading
        self.assertEqual(cancelled.exception.value.code, "cancelled")
        with self.assertRaises(StopAsyncIteration):
            await anext(subscription)

    async def test_bound_streams(self) -> None:
        owner = SubscriptionPlugin()
        transport = Mock(spec=plugin.Context)
        transport.attempt = 1
        transport.scope.return_value = "execution"
        transport.subscribe = AsyncMock()
        context = Context(transport, owner)
        template_context = templates.Context(transport, owner, {}, owner._suites[0].templates[0])
        plugin_stream = cast(Stream[Int64Value], vars(SubscriptionPlugin.readings)["__dokime_stream__"])
        template_stream = cast(Stream[Int64Value], vars(SubscriptionTemplate.readings)["__dokime_stream__"])
        for caller, declaration, identifier in (
            (context, plugin_stream, "subscription.readings"),
            (template_context, plugin_stream, "subscription.readings"),
            (template_context, template_stream, owner._suites[0].templates[0].resolve(template_stream)),
            (context, Stream("peer.readings", Int64Value), "peer.readings"),
        ):
            subscription = await caller.subscribe(declaration, max_item_bytes=8192)
            self.assertIs(subscription, transport.subscribe.return_value)
            transport.subscribe.assert_awaited_with(identifier, Int64Value, max_item_bytes=8192)
        row = StreamRow(Int64Value(value=7), Timestamp(seconds=3), Timestamp(seconds=5))
        self.assertIs(type(row), plugin.StreamRow)
        self.assertIs(type(Gap(2)), plugin.Gap)

    async def test_unbound_stream(self) -> None:
        owner = SubscriptionPlugin()
        transport = Mock(spec=plugin.Context)
        transport.attempt = 1
        transport.subscribe = AsyncMock()
        context = Context(transport, owner)
        with self.assertRaises(ValueError):
            await context.subscribe(Stream(row=Int64Value), max_item_bytes=8192)
        transport.subscribe.assert_not_awaited()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
