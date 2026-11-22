# SPDX-License-Identifier: Apache-2.0

"""Plugin Failure Mapping Preserves Actionable Operation Errors."""

import asyncio
from unittest import IsolatedAsyncioTestCase

import harmos
from google.protobuf.wrappers_pb2 import Int64Value
from harmos import plugin
from harmos._wire.plugin.v1.invocation_pb2 import Request

from dokime_sdk import Plugin
from dokime_sdk.operations import Context, OperationError, operation


def remote_failure(error: Exception) -> OperationError | None:
    if isinstance(error, ConnectionError):
        return OperationError("transient", code="disconnected", message=str(error), action="Reconnect the Device")
    return None


class FailureMappingTests(IsolatedAsyncioTestCase):
    async def test_mapper_preserves_failure_details_and_unmapped_handler_identity(self) -> None:
        for exception in (ConnectionError("Device Is Disconnected"), ValueError("Invalid Channel")):

            @operation("example.read")
            async def read(ctx: Context, request: int) -> int:
                raise exception

            class ReadPlugin(Plugin):
                identifier = "example"
                version = "1.0.0"
                operations = (read,)
                failure = remote_failure

            owner = ReadPlugin()
            _, handler = owner._plugin._registrations()[0][0]
            transport = plugin.Context(Request(attempt=1), owner.identifier, {})
            with self.assertRaises(OperationError) as raised:
                await handler(transport, Int64Value())
            failure = raised.exception
            self.assertIn(str(exception), failure.message)
            self.assertTrue(failure.action)
            if isinstance(exception, ConnectionError):
                self.assertEqual(failure.kind, "transient")
                self.assertEqual(failure.action, "Reconnect the Device")
            else:
                self.assertEqual(failure.kind, "permanent")
                self.assertEqual(failure.operation, "example.read")
                self.assertIn("example.read", failure.message)

    async def test_operation_errors_and_cancellation_are_preserved(self) -> None:
        for exception in (OperationError("refused", message="Missing Secret", action="Store the Secret"), asyncio.CancelledError()):

            @operation("example.read")
            async def read(ctx: Context, request: int) -> int:
                raise exception

            class ReadPlugin(Plugin):
                version = "1.0.0"
                operations = (read,)
                failure = remote_failure

            owner = ReadPlugin()
            _, handler = owner._plugin._registrations()[0][0]
            transport = plugin.Context(Request(attempt=1), owner.identifier, {})
            with self.assertRaises(type(exception)) as raised:
                await handler(transport, Int64Value())
            self.assertIs(raised.exception, exception)
