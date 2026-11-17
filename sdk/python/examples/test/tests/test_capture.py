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
from uuid import uuid4

# Third Party Imports
from google.protobuf.wrappers_pb2 import BoolValue
from harmos.client import PLUGIN_STATE_READY, PluginPlacement, PluginRemote, PluginValue

# Local Imports
from host import Host
from suite import plugin

from dokime import CapturedRecordSelection, MapValue, Metadata, RecordScope, model
from dokime.client import (
    RESULT_STATUS_PASSED,
    ExecutionBinding,
    RegisteredSelection,
    SelectionPlan,
)
from dokime.schemas import FrozenSchemaCache

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CaptureStory(unittest.IsolatedAsyncioTestCase):
    async def test_serve_capture_and_release(self) -> None:
        # Serve And Admit The Starter Through A Prepared Product Host.
        host = await Host.start(self)
        endpoint = await plugin().start("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)
        self.addAsyncCleanup(host.close)
        created = await host.client.plugins.create(
            "example", PluginPlacement(remote=PluginRemote(url=endpoint.url)), PluginValue(enabled=True)
        )
        entry = created.entry
        watching = host.client.plugins.watch(name=entry.name)
        try:
            while entry.state != PLUGIN_STATE_READY:
                if entry.HasField("failure"):
                    raise RuntimeError(f"Await Example Plugin Ready: {entry.failure}")
                if await anext(watching, None) is None:
                    raise RuntimeError("Await Example Plugin Ready: Watch Closed")
                entry = (await host.client.plugins.get(entry.name)).entry
        finally:
            await watching.aclose()
        suites = (await host.client.suites.list(created.entry.name)).values
        self.assertEqual(len(suites), 1)
        suite = (await host.client.suites.get(suites[0].metadata.identifier, created.entry.name, 1)).value
        plan = SelectionPlan(selections=[RegisteredSelection(suite_id=suite.metadata.identifier, template_id="example.capture")])
        cases = (await host.client.cases.create(plan, False)).cases
        case = cases[0].origin.plugin_run.test_case
        self.assertEqual([item.metadata.identifier for item in case.steps], ["confirm"])

        # Await The Operator Question And Answer With The Client Root.
        cycle = (await host.client.cycles.create([], Metadata(label="Example Capture"), True, "")).cycle
        execution_id = str(uuid4())
        await host.client.executions.create(cycle.cycle_id, [ExecutionBinding(execution_id=execution_id, case_id=cases[0].case_id)])
        finished = False
        async for progress in host.client.executions.progress.watch(execution_id):
            if progress.phase == "prompt":
                prompts = await host.client.prompts.list("", True, execution_id=execution_id)
                self.assertEqual(len(prompts.prompts), 1)
                asked = (await host.client.prompts.get(prompts.prompts[0].prompt.prompt_id)).prompt
                await host.client.prompts.answers.create(asked.prompt.prompt_id, "yes", MapValue(), True)
            finished = progress.finished
        self.assertTrue(finished)
        execution = (await host.client.executions.get(execution_id)).execution
        self.assertEqual(execution.outcome.evaluated.status, RESULT_STATUS_PASSED)

        # Inspect Acknowledged Evidence After The Host Has Run Cleanup.
        records = await host.client.evidence.records.list(
            RecordScope(execution=execution_id), "example.confirmation", [], model.Window(), "", CapturedRecordSelection()
        )
        self.assertEqual(len(records.records), 1)
        self.assertTrue(all(BoolValue.FromString(record.payload).value for record in records.records))
        cache = FrozenSchemaCache()
        value = await cache.decode(host.client, execution, records.records[0].record_type, records.records[0].payload)
        self.assertTrue(BoolValue.FromString(value.SerializeToString()).value)
        self.assertFalse(endpoint.plugin._sessions)
        await host.client.plugins.delete(created.entry.name, created.entry.revision)
        self.assertEqual(list((await host.client.plugins.list()).entries), [])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
