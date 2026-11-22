# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Stop Releases a Template's Host Sleep Without Cancelling Its Handler."""

# === Imports === #

# Standard Library Imports
import asyncio
from datetime import timedelta
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path
from typing import cast
from unittest import IsolatedAsyncioTestCase

# Local Imports
from dokime_sdk import Plugin, Suite
from dokime_sdk.templates import Context, Termination, TestTemplate, criterion, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class WaitTemplate(TestTemplate):
    @step("Wait")
    async def wait(self, ctx: Context) -> None:
        owner = cast(WaitPlugin, ctx.plugin)
        owner.entered.set()
        while not ctx.stop_requested:
            await ctx.sleep(timedelta(hours=3))
        owner.completed = True

    @criterion("Completed Normally")
    async def completed(self, ctx: Context) -> bool:
        owner = cast(WaitPlugin, ctx.plugin)
        owner.evaluated = owner.completed
        return owner.completed

    async def cleanup(self, ctx: Context) -> None:
        cast(WaitPlugin, ctx.plugin).termination = ctx.termination


class WaitSuite(Suite):
    templates = (WaitTemplate,)


class WaitPlugin(Plugin):
    version = "1.0.0"
    suites = (WaitSuite,)

    def __init__(self) -> None:
        self.entered = asyncio.Event()
        self.completed = False
        self.evaluated = False
        self.termination: Termination | None = None
        super().__init__()


class StopWaitStory(IsolatedAsyncioTestCase):
    async def test_stop_waits(self) -> None:
        specification = spec_from_file_location("stop_host", Path(__file__).resolve().parents[1] / "support/host.py")
        assert specification is not None and specification.loader is not None
        module = module_from_spec(specification)
        specification.loader.exec_module(module)
        host = await module.Host.start(self, manual_clock=True)
        owner = WaitPlugin()
        await host.client.plugins.register(owner)
        cycle = await host.client.cycle(label="Stop Waits")
        cases = await host.client.cases.materialize(WaitTemplate)
        execution = await cycle.execution(cases[0])
        await execution.start()
        await owner.entered.wait()

        await execution.stop()
        async for _ in execution.watch():
            pass
        outcome = await execution.seal()
        self.assertEqual(outcome.termination, Termination.STOPPED)
        self.assertTrue(owner.completed)
        self.assertTrue(owner.evaluated)
        self.assertEqual(owner.termination, Termination.STOPPED)
        await cycle.seal()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
