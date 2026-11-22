# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Capture a Peer Plugin's Deterministic Rows Through the Real Product Host."""

# === Imports === #

# Standard Library Imports
import asyncio
import unittest
from collections.abc import AsyncIterator
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path
from typing import Protocol, cast

# Third Party Imports
import harmos

# Local Imports
from dokime_sdk import Client, Plugin, Suite
from dokime_sdk._generated.client_types import DocumentScope
from dokime_sdk.operations import Context
from dokime_sdk.templates import Document, Section, Stream, TestTemplate, step, stream, time_series

# Type Checking Imports

# === Public Exports === #

# === Constants === #

SYSTEM = Stream("peer_fixture.system")

# === Types === #

# === Classes === #


class PeerFixturePlugin(Plugin):
    version = "1.0.0"
    opened: list[str]
    closed: list[str]
    sampled: asyncio.Event

    @stream("System")
    async def system(self, ctx: Context) -> AsyncIterator[float]:
        try:
            scope = ctx._transport.scope()
        except harmos.FailureError:
            await asyncio.Future()
            return
        self.opened.append(scope)
        try:
            yield 42.0
            self.sampled.set()
            await asyncio.Future()
        finally:
            self.closed.append(scope)


class PeerCheck(TestTemplate):
    uses_streams = (SYSTEM,)
    documents = (Document("Evidence").section(Section("System").plot(time_series(SYSTEM.field()))),)

    @step("Read Peer")
    async def read(self, ctx: Context) -> None:
        caller = cast(CaptureFixturePlugin, ctx.plugin)
        caller.entered.set()
        await caller.proceed.wait()


class CaptureSuite(Suite):
    templates = (PeerCheck,)


class CaptureFixturePlugin(Plugin):
    version = "1.0.0"
    suites = (CaptureSuite,)
    entered: asyncio.Event
    proceed: asyncio.Event


class Host(Protocol):
    @property
    def client(self) -> Client: ...


class HostFactory(Protocol):
    async def start(self, story: unittest.IsolatedAsyncioTestCase) -> Host: ...


class PeerStreamsStory(unittest.IsolatedAsyncioTestCase):
    async def test_peer_stream_rows_survive_seal_and_cancel_releases_demand(self) -> None:
        # Start the Real Product Host and Register Two Independent Fixtures.
        specification = spec_from_file_location("peer_stream_host", Path(__file__).resolve().parents[1] / "support/host.py")
        assert specification is not None and specification.loader is not None
        module = module_from_spec(specification)
        specification.loader.exec_module(module)
        host = await cast(HostFactory, module.Host).start(self)
        client = host.client
        peer = PeerFixturePlugin()
        peer.opened, peer.closed = [], []
        peer.sampled = asyncio.Event()
        registration = await client.plugins.register(peer)
        caller = CaptureFixturePlugin()
        caller.entered, caller.proceed = asyncio.Event(), asyncio.Event()
        await client.plugins.register(caller)
        cycle = await client.cycle(label="Peer Streams")
        cases = await client.cases.materialize(PeerCheck)

        # Complete and Cancel Runs; Frozen Schemas Decode Every Stored Peer Row.
        for cancelled in (False, True):
            peer.sampled.clear()
            caller.entered.clear()
            caller.proceed.clear()
            execution = await cycle.execution(cases[0])
            await execution.start()
            await peer.sampled.wait()
            await caller.entered.wait()
            if cancelled:
                await execution.cancel()
            else:
                caller.proceed.set()
            async for _ in execution.watch():
                pass
            await execution.seal()
            rows = await execution.samples(SYSTEM).collect()
            self.assertEqual(len(rows), 1)
            self.assertEqual(rows[0].values["value"], 42.0)
            self.assertEqual(rows[0].value, 42.0)
            self.assertEqual(peer.opened, peer.closed)
        await registration.unregister()
        await cycle.seal()
        # Reopening After Removing the Producer Uses Only the Frozen Row Types.
        reopened = await client.executions.get(execution.identifier)
        rows = await reopened.samples("peer_fixture.system").collect()
        self.assertEqual(getattr(rows[0].value, "value"), 42.0)
        # Resolve the Template's Document and Render Its Peer Stream Plot From Frozen Evidence.
        template = caller._suites[0].templates[0]
        document = template.definition().documents[0]
        scope = DocumentScope(cycle_id=cycle.identifier, execution_id=execution.identifier)
        frames = await client._transport.documents.get(scope, document.metadata.identifier, False, [], "", 0, 0)
        self.assertFalse(frames.next_page_token)
        plots = [
            frame.frame.binding.result.plot.request
            for frame in frames.frames
            if frame.frame.WhichOneof("kind") == "binding" and frame.frame.binding.result.WhichOneof("kind") == "plot"
        ]
        self.assertEqual(len(plots), 1)
        rendered = await client._transport.plots.snapshots.list(plots)
        self.assertEqual(len(rendered.envelopes), 1)
        self.assertEqual(rendered.envelopes[0].payload.frame.point_count, 1)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
