# SPDX-License-Identifier: Apache-2.0

"""Handler Exception Classification Is Owned by Its Operation Policy."""

# === Imports === #

# Standard Library Imports
import asyncio
from unittest import IsolatedAsyncioTestCase

# Third Party Imports
import harmos
from google.protobuf.wrappers_pb2 import Int64Value
from harmos import plugin
from harmos._wire.plugin.v1.invocation_pb2 import Request

# Local Imports
from dokime_sdk import Failure, Plugin, Suite
from dokime_sdk.operations import Context, FailureError, OperationPolicy, operation
from dokime_sdk.templates import TestTemplate, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class HandlerPolicyTests(IsolatedAsyncioTestCase):
    async def test_handler_exceptions_use_the_predicate_and_preserve_identity(self) -> None:
        for transient in (None, lambda error: isinstance(error, ConnectionError), lambda error: False):
            policy = OperationPolicy(transient=transient)

            @operation("example.read", policy=policy)
            async def read(ctx: Context, request: int) -> int:
                raise ConnectionError("Device Is Disconnected")

            class ReadPlugin(Plugin):
                identifier = "example"
                version = "1.0.0"
                operations = (read,)

            owner = ReadPlugin()
            _, handler = owner._plugin._registrations()[0][0]
            transport = plugin.Context(Request(attempt=1), owner.identifier, {})
            with self.assertRaises(harmos.FailureError) as raised:
                await handler(transport, Int64Value())
            failure = raised.exception.value
            expected = harmos.CLASSIFICATION_TRANSIENT if transient and transient(ConnectionError()) else harmos.CLASSIFICATION_PERMANENT
            self.assertEqual(failure.classification, expected)
            self.assertEqual((failure.operation, failure.plugin), ("example.read", "example"))
            self.assertIn("Device Is Disconnected", failure.message)
            self.assertIn("example.read", failure.message)
            self.assertTrue(failure.action)
            self.assertEqual(policy._definition().ByteSize(), 0)

    async def test_structured_failures_and_cancellation_keep_their_meaning(self) -> None:
        for exception in (FailureError(Failure("Refused", "missing", "Missing", "Store the Secret")), asyncio.CancelledError()):

            @operation("example.read", policy=OperationPolicy(transient=lambda error: True))
            async def read(ctx: Context, request: int) -> int:
                raise exception

            class ReadPlugin(Plugin):
                identifier = "example"
                version = "1.0.0"
                operations = (read,)

            owner = ReadPlugin()
            _, handler = owner._plugin._registrations()[0][0]
            transport = plugin.Context(Request(attempt=1), owner.identifier, {})
            expected = asyncio.CancelledError if isinstance(exception, asyncio.CancelledError) else harmos.FailureError
            with self.assertRaises(expected) as raised:
                await handler(transport, Int64Value())
            if isinstance(raised.exception, harmos.FailureError):
                self.assertEqual(raised.exception.value.classification, harmos.CLASSIFICATION_REFUSED)

    async def test_step_policy_classifies_errors_at_the_template_boundary(self) -> None:
        class ReadTemplate(TestTemplate):
            @step("Read", policy=OperationPolicy(transient=lambda error: isinstance(error, ConnectionError)))
            async def read(self, ctx: Context) -> None:
                raise ConnectionError("Reconnect the Device")

        class ReadSuite(Suite):
            templates = (ReadTemplate,)

        class ReadPlugin(Plugin):
            version = "1.0.0"
            suites = (ReadSuite,)

        owner = ReadPlugin()
        template = owner._suites[0].templates[0]
        case = template.build({}).message
        declaration, handler = next(
            (value, callback) for value, callback in owner._plugin._registrations()[0] if value.id.endswith(".read")
        )
        request = declaration.request(
            execution="execution",
            test_case=case,
            step=case.steps[0].metadata.identifier,
            origin={"plugin_run": {"plugin": owner.identifier, "template": template.identifier, "test_case": case}},
        )
        transport = plugin.Context(Request(scope="execution", attempt=1), owner.identifier, {})
        with self.assertRaises(harmos.FailureError) as raised:
            await handler(transport, request)
        self.assertEqual(raised.exception.value.classification, harmos.CLASSIFICATION_TRANSIENT)
        self.assertEqual(raised.exception.value.operation, declaration.id)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
