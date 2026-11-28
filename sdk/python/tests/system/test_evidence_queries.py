# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Script Follows a Running Execution, Finds It Sealed, Reads Its Evidence and Exports It."""

# === Imports === #

# Standard Library Imports
import asyncio
import unittest
from datetime import UTC, datetime, timedelta
from importlib.util import module_from_spec, spec_from_file_location
from io import BytesIO
from pathlib import Path
from typing import Protocol, cast
from zipfile import ZipFile

# Third Party Imports
import harmos
from grpc.aio import AioRpcError
from harmos._generated.proto.model.v1.setting_pb2 import SettingValues

# Local Imports
from dokime_sdk import Client, Plugin, ResultStatus, Suite, operations
from dokime_sdk.operations import OperationError, Setting, Settings
from dokime_sdk.templates import Context, Observer, TestTemplate, criterion, record, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #

REPOSITORY = Path(__file__).resolve().parents[4]

# === Types === #

# === Classes === #


class CaptureTestTemplate(TestTemplate):
    flag = record(bool, label="Captured Flag", identifier="flag")

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
    records = (CaptureTestTemplate.flag,)


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
        host = await _start(self)
        client = host.client
        with self.assertRaises(OperationError):
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

        class ObserverTestTemplateReading(Observer[float]):
            identifier = "reading"
            label = "Reading"
            row = float
            interval_ms = 10

            async def sample(self, ctx: operations.Context) -> float:
                try:
                    saved = ctx.settings(ObserverSettings)
                    await ctx.record(ObserverTestTemplate.flag, saved.sample_count == 0)
                    return 2.5
                finally:
                    written.set()

        class ObserverTestTemplate(TestTemplate):
            uses_streams = (ObserverTestTemplateReading,)
            flag = record(bool, identifier="flag")

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
            records = (ObserverTestTemplate.flag,)
            settings = (ObserverSettings,)
            observers = (ObserverTestTemplateReading,)

        # Start the Product Host and Bind the Observer's Execution Scope.
        host = await _start(self)
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
        host = await _start(self)
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

    async def test_client_follows_rows_while_the_execution_runs(self) -> None:
        following = asyncio.Event()
        delivered = asyncio.Event()

        class FollowLevel(Observer[float]):
            identifier = "level"
            label = "Level"
            row = float
            interval_ms = 10

            def __init__(self) -> None:
                self.level = 0.0

            def sample(self, ctx: operations.Context) -> float:
                # Count Only Samples Taken After the Client Started Following.
                if following.is_set():
                    self.level += 1.0
                return self.level

        class FollowTestTemplate(TestTemplate):
            uses_streams = (FollowLevel,)
            event = record(int, label="Event", identifier="event")

            @step("Follow")
            async def follow(self, ctx: Context) -> None:
                await ctx.sleep(1.0)
                await ctx.record(self.event, 1)
                await following.wait()
                await ctx.sleep(1.0)
                await ctx.record(self.event, 2)
                await delivered.wait()

            @criterion("Received")
            async def received(self, ctx: Context) -> bool:
                return await ctx.records(self.event).last() == 2

        class FollowSuite(Suite):
            templates = (FollowTestTemplate,)

        class FollowPlugin(Plugin):
            identifier = "follow"
            version = "1.0.0"
            suites = (FollowSuite,)
            records = (FollowTestTemplate.event,)
            observers = (FollowLevel,)

        # Both Followers Have Yielded Earlier Rows Before the Step Publishes More.
        host = await _start(self)
        client = host.client
        await client.plugins.register(FollowPlugin())
        cycle = await client.cycle(label="Live Follow")
        cases = await client.cases.materialize(FollowTestTemplate)
        execution = await cycle.execution(cases[0])
        await execution.start()
        records = execution.records(FollowTestTemplate.event).follow()
        levels = execution.samples("follow.level", row_type=float).follow()
        first = await anext(records)
        self.assertEqual(first.value, 1)
        followed = [await anext(levels)]
        self.assertEqual(followed[0].value, 0.0)
        following.set()

        # The Step Waits Until the Client Received Its Mid-Run Record and Later Samples.
        second = await anext(records)
        self.assertEqual(second.value, 2)
        self.assertGreater(second.sequence, first.sequence)
        while followed[-1].value < 1.0:
            followed.append(await anext(levels))
        self.assertGreaterEqual(await execution.samples("follow.level", row_type=float).last(), followed[-1].value)
        delivered.set()

        # Both Followers End When the Execution Closes, Matching Its Frozen Rows.
        self.assertEqual([row async for row in records], [])
        followed.extend([row async for row in levels])
        self.assertEqual((await execution.seal()).status, ResultStatus.PASSED)
        frozen = await execution.samples("follow.level", row_type=float).collect()
        self.assertEqual([row.sequence for row in followed], [row.sequence for row in frozen])
        self.assertEqual(await execution.samples("follow.level", row_type=float).last(), frozen[-1].value)
        sealed = [row.sequence async for row in execution.records(FollowTestTemplate.event).follow()]
        self.assertEqual(sealed, [first.sequence, second.sequence])
        with self.assertRaises(OperationError):
            await execution.samples("follow.level", between=(datetime(2100, 1, 1, tzinfo=UTC), None), row_type=float).last()
        await cycle.seal()


# === Public Functions === #

# === Private Functions === #


async def _start(story: unittest.IsolatedAsyncioTestCase) -> Host:
    # The Shared Host Fixture Lives Outside a Package, So Load It by Path.
    specification = spec_from_file_location("evidence_host", REPOSITORY / "sdk/python/tests/support/host.py")
    assert specification is not None and specification.loader is not None
    module = module_from_spec(specification)
    specification.loader.exec_module(module)
    return await cast(HostFactory, module.Host).start(story)


# === Entry Point === #
