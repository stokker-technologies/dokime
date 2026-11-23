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
from datetime import timedelta
from importlib.util import module_from_spec, spec_from_file_location
from io import BytesIO
from pathlib import Path
from typing import Protocol, cast
from zipfile import ZipFile

# Third Party Imports
import harmos
from grpc.aio import AioRpcError

# Local Imports
from dokime_sdk import Client, Plugin, ResultStatus, SdkError, Suite
from dokime_sdk._generated.shared import SettingValues
from dokime_sdk.operations import Setting, Settings
from dokime_sdk.templates import Context, TestTemplate, criterion, Observer, record, step, stream

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
    async def test_library_host_enforces_setting_defaults_and_writes(self) -> None:
        class BoundsSettings(Settings):
            count: int = Setting.integer("Count", default=5, min=0, max=10)
            level: float = Setting.number("Level", default=0.5, min=0.0, max=1.0)
            interval: timedelta = Setting.duration("Interval", default=timedelta(seconds=1), min=timedelta(), max=timedelta(seconds=2))

        class SettingsPlugin(Plugin):
            version = "1.0.0"
            settings = (BoundsSettings,)

        class OutsideSettings(Settings):
            count: int = Setting.integer("Count", default=11, min=0, max=10)

        class OutsideBoundsPlugin(Plugin):
            version = "1.0.0"
            settings = (OutsideSettings,)

        # Admit Defaults Through the Library Host Without Native Collectors or Controls.
        specification = spec_from_file_location("settings_host", REPOSITORY / "sdk/python/tests/support/host.py")
        assert specification is not None and specification.loader is not None
        module = module_from_spec(specification)
        specification.loader.exec_module(module)
        host = await cast(HostFactory, module.Host).start(self)
        client = host.client
        with self.assertRaises(SdkError):
            await client.plugins.register(OutsideBoundsPlugin())
        entry = await client.plugins.register(SettingsPlugin())
        initial = await client._transport.plugins.settings.list(name=entry.identifier)

        # Refused Writes Preserve the Complete Saved Value.
        for name, outside in (
            ("count", harmos.SettingValue(integer=-1)),
            ("count", harmos.SettingValue(integer=11)),
            ("level", harmos.SettingValue(number=-0.5)),
            ("level", harmos.SettingValue(number=1.5)),
            ("interval", harmos.SettingValue(duration={"seconds": -1})),
            ("interval", harmos.SettingValue(duration={"seconds": 3})),
        ):
            values = SettingValues()
            values.CopyFrom(initial.value)
            values.values[name].CopyFrom(outside)
            with self.assertRaises(AioRpcError):
                await client._transport.plugins.settings.set(entry.identifier, values, initial.revision)
            current = await client._transport.plugins.settings.list(name=entry.identifier)
            self.assertEqual(current.value, initial.value)

        # Inclusive Bounds Are Saved Through the Same Host Resource.
        revision = initial.revision
        for bound in ("minimum", "maximum"):
            values = SettingValues(values={name: getattr(declaration, bound) for name, declaration in BoundsSettings._fields().items()})
            saved = await client._transport.plugins.settings.set(entry.identifier, values, revision)
            self.assertEqual(saved.value, values)
            revision = saved.revision

    async def test_observer_children(self) -> None:
        class ObserverSettings(Settings):
            sample_count: int = Setting.integer("Sample Count", default=0, min=0, max=10, description="Samples per Capture")

        written = asyncio.Event()

        class ObserverTestTemplateReading(Observer):
            identifier = 'reading'
            label = "Reading"
            row = float
            interval_ms = 10

            async def sample(self, ctx: Context) -> float:
                try:
                    saved = ctx.settings(ObserverSettings)
                    await ctx.record(ObserverTestTemplate.flag, saved.sample_count == 0)
                    return 2.5
                finally:
                    written.set()


        class ObserverTestTemplate(TestTemplate):
            uses_streams = (ObserverTestTemplateReading,)
            flag = record(bool)

            def __init__(self) -> None:
                pass

            @step("Capture")
            async def capture(self, ctx: Context) -> None:
                await ctx.sleep(0.25)
                await written.wait()


            @criterion("Captured")
            async def captured(self, ctx: Context) -> bool:
                return await ctx.records(self.flag).last()

        class ObserverSuite(Suite):
            templates = (ObserverTestTemplate,)

        class ObserverPlugin(Plugin):
            version = "1.0.0"
            suites = (ObserverSuite,)
            settings = (ObserverSettings,)
            observers = (ObserverTestTemplateReading,)

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
