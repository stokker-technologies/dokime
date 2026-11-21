# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Automatic Peer Requirements and the Shared Bound Call Shape."""

# === Imports === #

# Standard Library Imports
import unittest
from datetime import timedelta
from types import ModuleType
from typing import cast
from unittest.mock import AsyncMock

# Third Party Imports
import harmos
from google.protobuf.wrappers_pb2 import Int64Value
from harmos import plugin
from harmos._wire.model.v1.failure_pb2 import Failure

# Local Imports
from dokime_sdk import Plugin, Suite
from dokime_sdk.operations import Context, FailureError, Operation, OperationPolicy, Visibility, operation
from dokime_sdk.templates import TestTemplate, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PEER = Operation("peer.increment", int, int, Visibility.PUBLIC)
PRIVATE = Operation("caller.local", int, int, Visibility.PRIVATE)

# === Types === #

# === Classes === #


class Contracts(ModuleType):
    increment: Operation[int, int]


CONTRACTS = Contracts("peer_contracts")
CONTRACTS.increment = PEER


class PeerTemplate(TestTemplate):
    @step("Increment")
    async def increment(self, ctx: Context) -> None:
        await ctx.operation(PEER)(41)


class PeerSuite(Suite):
    templates = (PeerTemplate,)


class CallerPlugin(Plugin):
    version = "1.0.0"
    suites = (PeerSuite,)


class PeerRequirementsTests(unittest.IsolatedAsyncioTestCase):
    def test_template_import_freezes_the_exact_contract_once(self) -> None:
        owner = CallerPlugin()
        required = [item for item in owner.declarations().requires if item.id == PEER.identifier]
        self.assertEqual(len(required), 1)
        self.assertEqual(required[0].fingerprint, PEER._definition().fingerprint)

    def test_helpers_module_members_closures_and_inherited_members_are_discovered(self) -> None:
        selected = (PEER, PRIVATE)

        async def helper(ctx: Context) -> int:
            return await ctx.operation(CONTRACTS.increment)(1)

        @operation("caller.call")
        async def invoke(ctx: Context, request: int) -> int:
            await ctx.operation(selected[request])(request)
            return await helper(ctx)

        class BasePlugin(Plugin):
            version = "1.0.0"
            target = PEER
            operations = (invoke,)

            async def initialize(self, ctx: Context) -> None:
                await ctx.operation(self.target)(1)

        class ChildPlugin(BasePlugin):
            pass

        required = {item.id: item for item in ChildPlugin().declarations().requires}
        self.assertIn(PEER.identifier, required)
        self.assertNotIn(PRIVATE.identifier, required)

    def test_conflicting_imported_contracts_are_rejected_before_activation(self) -> None:
        conflict = Operation(PEER.identifier, str, str, Visibility.PUBLIC)

        @operation("caller.conflict")
        async def invoke(ctx: Context, request: int) -> int:
            await ctx.operation(PEER)(request)
            await ctx.operation(conflict)("")
            return request

        class ConflictingPlugin(Plugin):
            version = "1.0.0"
            operations = (invoke,)

        with self.assertRaises(ValueError):
            ConflictingPlugin()

    def test_module_reference_is_discovered_without_a_direct_import(self) -> None:
        @operation("caller.module")
        async def invoke(ctx: Context, request: int) -> int:
            return await ctx.operation(CONTRACTS.increment)(request)

        class ModulePlugin(Plugin):
            version = "1.0.0"
            operations = (invoke,)

        self.assertIn(PEER.identifier, {item.id for item in ModulePlugin().declarations().requires})

    def test_helper_default_reference_is_frozen(self) -> None:
        async def helper(ctx: Context, target: Operation[int, int] = PEER) -> int:
            return await ctx.operation(target)(1)

        @operation("caller.default")
        async def invoke(ctx: Context, request: int) -> int:
            return await helper(ctx)

        class DefaultPlugin(Plugin):
            version = "1.0.0"
            operations = (invoke,)

        self.assertIn(PEER.identifier, {item.id for item in DefaultPlugin().declarations().requires})

    async def test_bound_call_packs_values_and_forwards_one_call_policy(self) -> None:
        transport = AsyncMock(spec=plugin.Context)
        transport.attempt = 1
        transport.call.return_value = Int64Value(value=42)
        context = Context(cast(plugin.Context, transport), CallerPlugin())
        policy = OperationPolicy(timeout=timedelta(seconds=3))
        self.assertEqual(await context.operation(PEER)(41, policy=policy), 42)
        transport.call.assert_awaited_once()
        arguments = transport.call.call_args
        self.assertEqual(arguments.args[0].fingerprint, PEER._definition().fingerprint)
        self.assertEqual(arguments.args[1], Int64Value(value=41))
        self.assertEqual(arguments.kwargs["policy"], policy._definition())

    async def test_bound_call_preserves_structured_peer_failures(self) -> None:
        transport = AsyncMock(spec=plugin.Context)
        transport.attempt = 1
        failure = Failure(code="deadline", classification=harmos.CLASSIFICATION_TIMED_OUT, action="Register the Peer")
        transport.call.side_effect = harmos.FailureError(failure)
        context = Context(cast(plugin.Context, transport), CallerPlugin())
        with self.assertRaises(FailureError) as raised:
            await context.operation(PEER)(1)
        self.assertEqual(raised.exception.failure.code, failure.code)
        self.assertEqual(raised.exception.failure.action, failure.action)

    async def test_runtime_created_reference_requires_reactivation(self) -> None:
        transport = AsyncMock(spec=plugin.Context)
        transport.attempt = 1
        transport.call.side_effect = ValueError("Undeclared Requirement")
        context = Context(cast(plugin.Context, transport), CallerPlugin())
        late = Operation("late.increment", int, int, Visibility.PUBLIC)
        with self.assertRaises(FailureError) as raised:
            await context.operation(late)(1)
        self.assertEqual(raised.exception.failure.code, "invalid_operation_contract")
        self.assertEqual(raised.exception.failure.operation, late.identifier)
        self.assertTrue(raised.exception.failure.action)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
