# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Script Finds a Sealed Execution, Reads Its Telemetry as Dataframes and Exports Its Evidence."""

# === Imports === #

# Standard Library Imports
import asyncio
import unittest
from importlib.util import module_from_spec, spec_from_file_location
from io import BytesIO
from pathlib import Path
from typing import Protocol, cast
from zipfile import ZipFile

# Third Party Imports
# Local Imports
from dokime_sdk import Client, Plugin, ResultStatus, Suite
from dokime_sdk.operations import SettingDeclaration, SettingValue
from dokime_sdk.templates import Context, TestTemplate, criterion, observer, record, step, stream

# Type Checking Imports

# === Public Exports === #

# === Constants === #

REPOSITORY = Path(__file__).resolve().parents[4]

# === Types === #

# === Classes === #


class CaptureTestTemplate(TestTemplate):
    flag = record(bool, label="Captured Flag")

    @step("Capture")
    async def capture(self, ctx: Context) -> None:
        await ctx.record(self.flag, True)

    @criterion("Captured")
    async def captured(self, ctx: Context) -> bool:
        return await ctx.records(self.flag).last()


class CaptureSuite(Suite):
    templates = (CaptureTestTemplate,)


class CapturePlugin(Plugin):
    version = "1.0.0"
    suites = (CaptureSuite,)


class Host(Protocol):
    @property
    def client(self) -> Client: ...


class HostFactory(Protocol):
    async def start(self, story: unittest.IsolatedAsyncioTestCase) -> Host: ...


class EvidenceQueryStory(unittest.IsolatedAsyncioTestCase):
    async def test_observer_children(self) -> None:
        class ObserverTestTemplate(TestTemplate):
            flag = record(bool)

            def __init__(self) -> None:
                self.written = asyncio.Event()

            @step("Capture")
            async def capture(self, ctx: Context) -> None:
                await ctx.sleep(0.25)
                await self.written.wait()

            @stream("Reading")
            @observer(interval_ms=10)
            async def reading(self, ctx: Context) -> float:
                try:
                    saved = await ctx.settings()
                    await ctx.record(self.flag, saved["sample_count"].integer == 0)
                    return 2.5
                finally:
                    self.written.set()

            @criterion("Captured")
            async def captured(self, ctx: Context) -> bool:
                return await ctx.records(self.flag).last()

        class ObserverSuite(Suite):
            templates = (ObserverTestTemplate,)

        class ObserverPlugin(Plugin):
            version = "1.0.0"
            suites = (ObserverSuite,)
            settings = (
                SettingDeclaration(
                    "sample_count",
                    "Sample Count",
                    "Samples per Capture",
                    SettingValue(integer=0),
                    minimum=SettingValue(integer=0),
                    maximum=SettingValue(integer=10),
                ),
            )

        # Start the Product Host and Bind the Observer's Execution Scope.
        specification = spec_from_file_location("observer_host", REPOSITORY / "sdk/python/tests/support/host.py")
        assert specification is not None and specification.loader is not None
        module = module_from_spec(specification)
        specification.loader.exec_module(module)
        host = await cast(HostFactory, module.Host).start(self)
        client = host.client
        await client.plugins.register(ObserverPlugin())
        cycle = await client.cycle(label="Observer Children")
        cases = await client.cases.materialize(ObserverTestTemplate)
        execution = await cycle.execution(cases[0])

        # Admit Observer Child Writes and Read Their Frozen Records.
        async for progress in execution.run():
            pass
        outcome = await execution.seal()
        self.assertEqual(outcome.status, ResultStatus.PASSED, outcome)
        self.assertTrue(await execution.records(ObserverTestTemplate.flag).last())
        await cycle.seal()

    async def test_bound_lookup_frozen_records_and_retained_archives(self) -> None:
        specification = spec_from_file_location("evidence_starter_host", REPOSITORY / "sdk/python/tests/support/host.py")
        assert specification is not None and specification.loader is not None
        module = module_from_spec(specification)
        specification.loader.exec_module(module)
        host = await cast(HostFactory, module.Host).start(self)
        client = host.client
        await client.plugins.register(CapturePlugin())
        cycle = await client.cycle(label="Evidence Queries")
        cases = await client.cases.materialize(CaptureTestTemplate)
        execution = await cycle.execution(cases[0])
        async for progress in execution.run():
            pass
        self.assertEqual((await execution.seal()).status, ResultStatus.PASSED)
        self.assertEqual((await execution.seal()).status, ResultStatus.PASSED)
        found = [value async for value in client.executions.list(cycle=cycle.identifier, state="sealed")]
        self.assertEqual([value.identifier for value in found], [execution.identifier])
        reopened = await client.executions.get(execution.identifier)
        self.assertEqual(reopened.cycle.identifier, cycle.identifier)
        self.assertTrue(await reopened.records(CaptureTestTemplate.flag).last())
        self.assertEqual(await reopened.records(CaptureTestTemplate.flag).count(), 1)
        await cycle.seal()
        first = await cycle.evidence()
        self.assertEqual(first, await cycle.evidence())
        archive = ZipFile(BytesIO(first))
        try:
            self.assertIn("manifest.json", archive.namelist())
        finally:
            archive.close()
        self.assertEqual(await reopened.evidence(), await reopened.evidence())


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
