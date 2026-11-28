# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Stop State Delegates to the Library Context."""

# === Imports === #

# Standard Library Imports
from asyncio import Queue
from datetime import UTC, datetime, timedelta
from unittest import IsolatedAsyncioTestCase
from unittest.mock import AsyncMock

# Third Party Imports
from google.protobuf.duration_pb2 import Duration
from google.protobuf.timestamp_pb2 import Timestamp
from harmos import plugin
from harmos._generated.proto.plugin.v1.invocation_pb2 import Request

# Local Imports
from dokime_sdk import Plugin, Suite, templates
from dokime_sdk.operations import Context
from dokime_sdk.templates import TestTemplate

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class WaitTemplate(TestTemplate):
    pass


class WaitSuite(Suite):
    templates = (WaitTemplate,)


class WaitPlugin(Plugin):
    version = "1.0.0"
    suites = (WaitSuite,)


class StopContextTests(IsolatedAsyncioTestCase):
    async def test_clock_calls_delegate_to_the_library_with_product_value_conversion(self) -> None:
        owner = WaitPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        transport.sleep = AsyncMock()
        transport.now = AsyncMock(return_value=Timestamp(seconds=42, nanos=125_000_000))
        context = Context(transport, owner)
        for duration in (1.25, timedelta(seconds=1.25)):
            await context.sleep(duration)
            transport.sleep.assert_awaited_with(Duration(seconds=1, nanos=250_000_000))
        self.assertEqual(await context.now(), datetime.fromtimestamp(42.125, tz=UTC))
        transport.now.assert_awaited_once_with()

    async def test_stop_context(self) -> None:
        owner = WaitPlugin()
        transport = plugin.Context(Request(scope="execution", attempt=1), "wait_plugin", {}, Queue())
        operation = Context(transport, owner)
        template = templates.Context(transport, owner, {}, owner._suites[0].templates[0])
        self.assertFalse(operation.stop_requested)
        self.assertFalse(template.stop_requested)
        transport._wind_down.set()
        self.assertTrue(operation.stop_requested)
        self.assertTrue(template.stop_requested)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
