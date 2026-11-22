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
from unittest import IsolatedAsyncioTestCase

# Third Party Imports
from harmos import plugin
from harmos._wire.plugin.v1.invocation_pb2 import Request

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
