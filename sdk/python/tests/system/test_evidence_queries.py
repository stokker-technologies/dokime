# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Script Finds a Sealed Execution, Reads Its Telemetry as Dataframes and Exports Its Evidence."""

# === Imports === #

# Standard Library Imports
import unittest
from datetime import UTC, datetime
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path
from tempfile import TemporaryDirectory
from uuid import uuid4
from zipfile import ZipFile

# Third Party Imports
from harmos.client import PLUGIN_STATE_READY, PluginPlacement, PluginRemote, PluginValue

# Local Imports
from dokime import MapValue, Metadata, Value
from dokime.client import (
    RESULT_STATUS_PASSED,
    ExecutionBinding,
    RegisteredSelection,
    RegisteredSelectionParametersEntry,
    SelectionPlan,
    SelectionValues,
)
from dokime.evidence import cycle, execution, executions

# Type Checking Imports

# === Public Exports === #

# === Constants === #

REPOSITORY = Path(__file__).resolve().parents[4]
STARTER = REPOSITORY / "sdk/python/examples/test/tests"
THERMAL = REPOSITORY / "sdk/python/examples/thermal/suite.py"
COUNTER = 1_152_921_504_606_847_975

# === Types === #

# === Classes === #


class EvidenceQueryStory(unittest.IsolatedAsyncioTestCase):
    async def test_find_read_and_export_a_sealed_execution(self) -> None:
        # Serve the Shared Thermal Suite Through a Prepared Product Host.
        fixtures = []
        for name, path in (("evidence_starter_host", STARTER / "host.py"), ("evidence_thermal_suite", THERMAL)):
            specification = spec_from_file_location(name, path)
            assert specification is not None and specification.loader is not None
            fixtures.append(module_from_spec(specification))
            specification.loader.exec_module(fixtures[-1])
        starter, thermal = fixtures
        host = await starter.Host.start(self)
        endpoint = await thermal.plugin().start("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)
        self.addAsyncCleanup(host.close)
        client = host.client
        entry = (
            await client.plugins.create("thermal", PluginPlacement(remote=PluginRemote(url=endpoint.url)), PluginValue(enabled=True))
        ).entry
        watching = client.plugins.watch(name=entry.name)
        try:
            while entry.state != PLUGIN_STATE_READY:
                if entry.HasField("failure") or await anext(watching, None) is None:
                    self.fail(f"Await Thermal Plugin Ready: {entry.failure}")
                entry = (await client.plugins.get(entry.name)).entry
        finally:
            await watching.aclose()

        # Run One Thermal Execution and Answer Its Door Prompt.
        suite = (await client.suites.list(entry.name)).values[0].metadata.identifier
        limit = RegisteredSelectionParametersEntry(key="limit", value=SelectionValues(values=[Value(number=85.0)]))
        plan = SelectionPlan(selections=[RegisteredSelection(suite_id=suite, template_id="thermal.cycle", parameters=[limit])])
        cases = (await client.cases.create(plan, False)).cases
        created = (await client.cycles.create([], Metadata(label="Evidence Queries"), True, "")).cycle
        identifier = str(uuid4())
        await client.executions.create(created.cycle_id, [ExecutionBinding(execution_id=identifier, case_id=cases[0].case_id)])
        async for progress in client.executions.progress.watch(identifier):
            if progress.phase == "prompt":
                pending = await client.prompts.list("", True, execution_id=identifier)
                await client.prompts.answers.create(pending.prompts[0].prompt.prompt_id, "yes", MapValue(), True)

        # Find It by Template and Status at One Fixed Cut, Without Following Anything.
        found = [item async for item in executions(client, template="thermal.cycle", status="sealed")]
        self.assertEqual([item.id for item in found], [identifier])
        self.assertEqual([item async for item in executions(client, template="thermal.cycle", status="open")], [])
        self.assertEqual([item async for item in executions(client, template="other.template")], [])
        sealed = found[0]
        self.assertEqual(sealed.detail.outcome.evaluated.status, RESULT_STATUS_PASSED)
        self.assertEqual((await execution(client, identifier)).detail, sealed.detail)
        with self.assertRaises(ValueError):
            [item async for item in executions(client, status="failed")]

        # Read Exact Rows and Their Source Times.
        temperature = sealed.telemetry("thermal.temperature", fields=["celsius"])
        rows = await temperature.collect()
        self.assertEqual(
            [(row.timestamp_ns, row.values) for row in rows], [(1_000_000_000, {"celsius": 84.0}), (2_000_000_000, {"celsius": 84.0})]
        )
        self.assertEqual(rows[0].timestamp, datetime(1970, 1, 1, 0, 0, 1, tzinfo=UTC))
        self.assertIsNotNone(temperature.snapshot)
        with self.assertRaises(ValueError):
            await sealed.telemetry("thermal.temperature").collect(max_rows=1)

        # Keep Exact Unsigned Values, Nanosecond Times and Paged Reads in Polars and Pandas.
        counter = sealed.telemetry("thermal.counter", fields=["value"], page_size=500)
        frame = await counter.to_polars()
        count = frame.height
        self.assertGreater(count, 500)
        self.assertEqual(frame.columns, ["sequence", "timestamp", "value"])
        self.assertEqual(str(frame.schema["value"]), "UInt64")
        self.assertEqual(str(frame.schema["timestamp"]), "Datetime(time_unit='ns', time_zone='UTC')")
        self.assertEqual(frame["value"].to_list(), [COUNTER] * count)
        self.assertEqual(frame["timestamp"].cast(int).to_list()[:2], [1_000_000, 2_000_000])
        table = await sealed.telemetry("thermal.counter", fields=["value"], at=counter.snapshot).to_pandas()
        self.assertEqual(str(table["value"].dtype), "UInt64")
        self.assertEqual(str(table["timestamp"].dtype), "datetime64[ns, UTC]")
        self.assertEqual(table["value"].tolist(), [COUNTER] * count)
        self.assertEqual(table["sequence"].tolist(), frame["sequence"].to_list())

        # Export the Execution and Its Cycle Atomically, Refusing Existing Destinations.
        scratch = TemporaryDirectory(prefix="dokime-evidence-export-")
        self.addCleanup(scratch.cleanup)
        target = Path(scratch.name) / "execution.zip"
        self.assertEqual(await sealed.export(target), target)
        archive = ZipFile(target)
        try:
            names = archive.namelist()
        finally:
            archive.close()
        self.assertIn("manifest.json", names)
        self.assertIn("evidence.pb", names)
        exported = target.read_bytes()
        with self.assertRaises(FileExistsError):
            await sealed.export(target)
        self.assertEqual(target.read_bytes(), exported)
        await sealed.export(target, overwrite=True)
        self.assertEqual(target.read_bytes(), exported)
        bundle = Path(scratch.name) / "cycle.zip"
        await sealed.cycle.export(bundle)
        archive = ZipFile(bundle)
        try:
            members = archive.namelist()
        finally:
            archive.close()
        self.assertIn("manifest.json", members)
        self.assertEqual(len([name for name in members if name.endswith(".zip")]), 1)
        self.assertEqual(cycle(client, created.cycle_id).id, sealed.detail.cycle_id)
        self.assertEqual(sorted(path.name for path in Path(scratch.name).iterdir()), ["cycle.zip", "execution.zip"])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
