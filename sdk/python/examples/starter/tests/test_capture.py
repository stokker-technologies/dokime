# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""An Operator Runs The Starter Through A Host And Its Controlled Clock."""

# === Imports === #

# Standard Library Imports
import unittest

# Third Party Imports
from google.protobuf.wrappers_pb2 import BoolValue

# Local Imports
from host import Host
from suite import plugin

import dokime

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CaptureStory(unittest.IsolatedAsyncioTestCase):
    async def test_serve_capture_and_release(self):
        # Serve And Admit The Starter Through A Prepared Product Host.
        host = await Host.start(self)
        endpoint = await dokime.serve(plugin(), address="127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)
        self.addAsyncCleanup(host.close)
        await host.client.plugins.add(endpoint.url)
        suites = await host.client.suites.list()
        suite = await host.client.suites.get(suites[0].metadata.identifier)
        self.assertEqual(suite.metadata.identifier, "example")
        cases = await host.client.templates.materialize(suites[0].metadata.identifier, "example.capture", {})
        case = cases[0].origin.plugin_run.test_case
        self.assertEqual([item.metadata.identifier for item in case.steps], ["confirm"])

        # Await The Operator Question And Answer With The Client Root.
        cycle = await host.client.cycles.start(cases)
        async for progress in cycle.watch():
            if progress.phase == "prompt":
                prompts = await host.client.prompts.pending(cycle.executions[0])
                asked = await host.client.prompts.get(prompts[0].prompt.prompt_id)
                await host.client.prompts.respond(asked.prompt.prompt_id, asked.prompt.definition.actions[0].identifier, accepted=True)
        execution = await host.client.executions.get(cycle.executions[0])
        self.assertEqual(
            execution.outcome.evaluated.status,
            execution.outcome.evaluated.DESCRIPTOR.fields_by_name["status"].enum_type.values_by_name["RESULT_STATUS_PASSED"].number,
        )

        # Inspect Acknowledged Evidence After The Host Has Run Cleanup.
        records = await host.client.records.list(cycle.executions[0])
        self.assertEqual(len(records.records), 1)
        self.assertTrue(all(BoolValue.FromString(record.payload).value for record in records.records))
        cache = dokime.FrozenSchemaCache()
        value = await cache.decode(host.client, execution, records.records[0].record_type, records.records[0].payload)
        self.assertTrue(BoolValue.FromString(value.SerializeToString()).value)
        self.assertFalse(endpoint.plugin._sessions)
        await host.client.plugins.remove((await host.client.plugins.list()).plugins[0].entry)
        self.assertEqual((await host.client.plugins.list()).plugins, [])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
