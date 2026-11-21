# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Two Fixture Plugins Compose Public Operations Through the Product Host."""

# === Imports === #

# Standard Library Imports
import asyncio
import unittest
from datetime import timedelta
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path
from typing import Protocol, cast

# Third Party Imports
import harmos

# Local Imports
from dokime_sdk import Client, Lane, Plugin, ResultStatus, Suite
from dokime_sdk.operations import Context, FailureError, Operation, OperationPolicy, Visibility, operation
from dokime_sdk.templates import TestTemplate, criterion, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #

INCREMENT = Operation("peer_fixture.increment", int, int, Visibility.PUBLIC)
SLOW = Operation("peer_fixture.slow", int, int, Visibility.PUBLIC)
DEVICE = Lane("device", serial=True)

# === Types === #

# === Classes === #


class CalleePlugin(Plugin):
    version = "1.0.0"
    lanes = (DEVICE,)
    scopes: list[str]
    entered: asyncio.Event
    cancelled: asyncio.Event


class PeerTemplate(TestTemplate):
    @step("Call the Peer")
    async def call_peer(self, ctx: Context) -> None:
        caller = cast(CallerPlugin, ctx.plugin)
        if caller.mode == "undeclared":
            late = Operation("late_fixture.increment", int, int, Visibility.PUBLIC)
            try:
                await ctx.operation(late)(1)
            except FailureError as error:
                caller.failure = error
            return
        try:
            caller.result = await ctx.operation(INCREMENT)(41)
        except FailureError as error:
            caller.failure = error
            return
        try:
            await ctx.operation(SLOW)(1)
        except FailureError as error:
            caller.failure = error

    @criterion("Peer Result Recorded")
    async def recorded(self, ctx: Context) -> bool:
        return cast(CallerPlugin, ctx.plugin).failure is not None


class PeerSuite(Suite):
    templates = (PeerTemplate,)


class CallerPlugin(Plugin):
    version = "1.0.0"
    suites = (PeerSuite,)
    mode = "available"
    result: int | None = None
    failure: FailureError | None = None


class Host(Protocol):
    @property
    def client(self) -> Client: ...

    async def advance(self, at_ns: int) -> None: ...


class HostFactory(Protocol):
    async def start(self, story: unittest.IsolatedAsyncioTestCase, *, manual_clock: bool = False) -> Host: ...


class PeerOperationsStory(unittest.IsolatedAsyncioTestCase):
    async def test_template_calls_peer_with_callee_policy_scope_and_actionable_refusals(self) -> None:
        specification = spec_from_file_location("peer_host", Path(__file__).resolve().parents[1] / "support/host.py")
        assert specification is not None and specification.loader is not None
        module = module_from_spec(specification)
        specification.loader.exec_module(module)
        host = await cast(HostFactory, module.Host).start(self, manual_clock=True)
        client = host.client
        callee = CalleePlugin()
        callee.scopes = []
        callee.entered = asyncio.Event()
        callee.cancelled = asyncio.Event()
        registered = await client.plugins.register(callee)
        caller = CallerPlugin()
        await client.plugins.register(caller)
        cycle = await client.cycle(label="Peer Operations")
        cases = await client.cases.materialize(PeerTemplate)
        execution = await cycle.execution(cases[0])
        await execution.start()
        await callee.entered.wait()
        await host.advance(2_000_000_000)
        async for _ in execution.watch():
            pass
        await execution.refresh()
        self.assertEqual((await execution.seal()).status, ResultStatus.PASSED, (caller.result, caller.failure, execution._detail))
        self.assertEqual(caller.result, 42)
        assert caller.failure is not None
        self.assertEqual(caller.failure.failure.classification, "Timed Out")
        await callee.cancelled.wait()
        self.assertEqual(callee.scopes, [execution.identifier, execution.identifier])
        # A Settled Deadline Releases the Callee's Serial Lane.
        self.assertEqual(await client.operation(INCREMENT)(2), 3)
        await registered.unregister()
        for mode in ("unavailable", "undeclared"):
            caller.mode = mode
            caller.result = None
            caller.failure = None
            execution = await cycle.execution(cases[0])
            async for _ in execution.run():
                pass
            self.assertEqual((await execution.seal()).status, ResultStatus.PASSED)
            self.assertIsNone(caller.result)
            assert caller.failure is not None
            self.assertTrue(caller.failure.failure.action)
            self.assertTrue(caller.failure.failure.code)
            if mode == "undeclared":
                self.assertEqual(caller.failure.failure.code, "invalid_operation_contract")
        await cycle.seal()


# === Public Functions === #


@operation(INCREMENT.identifier, visibility=Visibility.PUBLIC, lane=DEVICE)
async def increment(ctx: Context, request: int) -> int:
    callee = cast(CalleePlugin, ctx.plugin)
    # External Client Calls Have No Execution Scope.
    try:
        scope = ctx._transport.scope()
    except harmos.FailureError:
        scope = ""
    if scope:
        callee.scopes.append(scope)
    return request + 1


@operation(SLOW.identifier, visibility=Visibility.PUBLIC, lane=DEVICE, policy=OperationPolicy(timeout=timedelta(seconds=2)))
async def slow(ctx: Context, request: int) -> int:
    callee = cast(CalleePlugin, ctx.plugin)
    callee.scopes.append(ctx._transport.scope())
    callee.entered.set()
    try:
        await ctx.sleep(timedelta(seconds=20))
    finally:
        callee.cancelled.set()
    return request


CalleePlugin.operations = (increment, slow)

# === Private Functions === #

# === Entry Point === #
