# SPDX-License-Identifier: Apache-2.0

"""Operation Calls Delegate JSON Conversion and Failures to the Library Host."""

# === Imports === #

# Standard Library Imports
from datetime import timedelta
from unittest import IsolatedAsyncioTestCase
from unittest.mock import AsyncMock

# Third Party Imports
import harmos
from harmos import plugin
from harmos._wire.model.v1.failure_pb2 import Failure
from harmos._wire.plugin.v1.invocation_pb2 import Request

# Local Imports
import dokime_sdk.operations
from dokime_sdk import Plugin
from dokime_sdk.operations import Context, OperationError, OperationPolicy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CallPlugin(Plugin):
    version = "1.0.0"


class UntypedOperationTests(IsolatedAsyncioTestCase):
    async def test_call_passes_the_id_optional_dict_and_policy_without_peer_declarations(self) -> None:
        owner = CallPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        operation = AsyncMock(return_value={"reading": 42})
        transport.operation = operation
        context = Context(transport, owner)
        policy = OperationPolicy(timeout=timedelta(seconds=3))
        request = {"channel": 1}
        self.assertEqual(await context.operation("example.measure", request, policy=policy), {"reading": 42})
        operation.assert_awaited_once_with("example.measure", request, policy=policy._definition())
        operation.reset_mock()
        await context.operation("example.status")
        operation.assert_awaited_once_with("example.status", None, policy=None)
        self.assertFalse(hasattr(dokime_sdk.operations, "Operation"))
        self.assertFalse(hasattr(dokime_sdk.operations, "declaration"))
        self.assertTrue(all(item.id.startswith(("dokime.", "harmos.")) for item in owner.declarations().requires))

    async def test_library_failure_wording_classification_and_causes_are_preserved(self) -> None:
        owner = CallPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        for classification, expected in (
            (harmos.CLASSIFICATION_REFUSED, "Refused"),
            (harmos.CLASSIFICATION_INVALID, "Invalid"),
            (harmos.CLASSIFICATION_PERMANENT, "Permanent"),
        ):
            failure = Failure(
                classification=classification,
                code="invalid_field",
                operation="example.measure",
                message="example.measure channel: Expected an Integer",
                action="Supply an Integer for channel, Then Retry",
                cause=Failure(code="schema", message="Unknown Field"),
            )
            transport.operation = AsyncMock(side_effect=harmos.OperationError(failure))
            with self.assertRaises(OperationError) as raised:
                await Context(transport, owner).operation("example.measure", {"channel": "wrong"})
            self.assertEqual(raised.exception.value.message, failure.message)
            self.assertEqual(raised.exception.value.action, failure.action)
            self.assertEqual(raised.exception.value.operation, failure.operation)
            self.assertEqual(raised.exception.value.classification, classification)
            cause = raised.exception.value.cause
            assert cause is not None
            self.assertEqual(cause.code, "schema")


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
