# SPDX-License-Identifier: Apache-2.0

"""Plugin Declarations Own Secret Reads and Activation Resource Lifetimes."""

# === Imports === #

# Standard Library Imports
import asyncio
from unittest import IsolatedAsyncioTestCase
from unittest.mock import AsyncMock, Mock

# Third Party Imports
import harmos
from harmos import plugin
from harmos._wire.model.v1.failure_pb2 import Failure
from harmos._wire.plugin.v1.invocation_pb2 import Request

# Local Imports
from dokime_sdk import Client, Plugin, RegisteredPlugin, SdkError
from dokime_sdk._generated import client_types
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsGetResponse
from dokime_sdk.operations import Context, Resource, Secret

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ResourceTests(IsolatedAsyncioTestCase):
    async def test_secret_attributes_return_text_and_name_the_recovery_action(self) -> None:
        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            api_token = Secret("Remote API Token")

        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        transport._call = AsyncMock(return_value=SecretsGetResponse(secret=b"token"))
        context = Context(transport, owner)
        self.assertEqual(ConnectionPlugin.api_token.identifier, "api_token")
        self.assertEqual(await context.secret(ConnectionPlugin.api_token), "token")
        transport._call.assert_awaited_once()

        for response in (b"", b"\xff"):
            transport._call.return_value = SecretsGetResponse(secret=response)
            with self.assertRaises(SdkError) as raised:
                await context.secret(ConnectionPlugin.api_token)
            self.assertEqual(raised.exception.failure.classification, "Refused")
            self.assertIn("Remote API Token", raised.exception.failure.action)
            self.assertIn("then Retry", raised.exception.failure.action)

        missing = Failure(code="NOT_FOUND", classification=harmos.CLASSIFICATION_REFUSED)
        transport._call.side_effect = harmos.FailureError(Failure(cause=missing))
        with self.assertRaises(SdkError) as raised:
            await context.secret(ConnectionPlugin.api_token)
        self.assertEqual(raised.exception.failure.action, "Store the Remote API Token secret for this plugin, then Retry")

        failure = Failure(code="denied", classification=harmos.CLASSIFICATION_REFUSED, action="Request Access")
        transport._call.side_effect = harmos.FailureError(failure)
        with self.assertRaises(harmos.FailureError) as raised_host:
            await context.secret(ConnectionPlugin.api_token)
        self.assertEqual(raised_host.exception.value, failure)
        with self.assertRaises(ValueError):
            await context.secret(Secret("Foreign"))

    async def test_concurrent_resource_reads_open_once_and_finalize_closes_once(self) -> None:
        entered, release = asyncio.Event(), asyncio.Event()
        opened: list[Context] = []
        closed: list[object] = []
        value = object()

        async def open_connection(ctx: Context) -> object:
            opened.append(ctx)
            entered.set()
            await release.wait()
            return value

        async def close_connection(connection: object) -> None:
            closed.append(connection)

        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            remote = Resource(open=open_connection, close=close_connection)

        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        context = Context(transport, owner)
        first = asyncio.create_task(context.resource(ConnectionPlugin.remote))
        await entered.wait()
        second = asyncio.create_task(context.resource(ConnectionPlugin.remote))
        release.set()
        self.assertEqual(await asyncio.gather(first, second), [value, value])
        self.assertEqual(opened, [context])
        finalize = owner._lifecycle("finalize")
        await finalize(object(), transport)
        await finalize(object(), transport)
        self.assertEqual(closed, [value])
        with self.assertRaises(SdkError):
            await context.resource(ConnectionPlugin.remote)
        await owner._lifecycle("initialize")(object(), transport)
        self.assertIs(await context.resource(ConnectionPlugin.remote), value)
        self.assertEqual(len(opened), 2)
        await finalize(object(), transport)
        self.assertEqual(closed, [value, value])

    async def test_resources_compose_and_all_release_even_after_finalize_failure(self) -> None:
        closed: list[str] = []

        async def open_first(ctx: Context) -> str:
            return "first"

        async def close_first(value: str) -> None:
            closed.append(value)

        async def open_second(ctx: Context) -> str:
            return await ctx.resource(ConnectionPlugin.first) + "/second"

        async def close_second(value: str) -> None:
            closed.append(value)
            raise ValueError("Reconnect Before Closing")

        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            first = Resource(open=open_first, close=close_first)
            second = Resource(open=open_second, close=close_second)

            async def finalize(self, ctx: Context) -> None:
                raise ValueError("Finalize Failed")

        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        context = Context(transport, owner)
        self.assertEqual(await context.resource(ConnectionPlugin.second), "first/second")
        with self.assertRaises(harmos.FailureError) as raised:
            await owner._lifecycle("finalize")(object(), transport)
        self.assertEqual(raised.exception.value.classification, harmos.CLASSIFICATION_PERMANENT)
        self.assertEqual(raised.exception.value.operation, "connection.finalize")
        self.assertEqual(closed, ["first/second", "first"])
        self.assertFalse(owner._resource_values)

    async def test_registration_release_closes_resources_when_the_host_cannot_finalize(self) -> None:
        opened, closed = AsyncMock(return_value="connection"), AsyncMock()

        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            remote = Resource[str](open=opened, close=closed)

        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        await Context(transport, owner).resource(ConnectionPlugin.remote)
        client = Mock(spec=Client)
        client.plugins = Mock()
        client.plugins._bindings = {}
        registered = RegisteredPlugin._bind(client, client_types.host.PluginEntry(name=owner.identifier), owner=owner)
        await registered._release()
        await registered._release()
        closed.assert_awaited_once_with("connection")

    async def test_failed_activation_releases_resources_before_returning_the_failure(self) -> None:
        opened, closed = AsyncMock(return_value="connection"), AsyncMock()

        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            remote = Resource[str](open=opened, close=closed)

            async def initialize(self, ctx: Context) -> None:
                await ctx.resource(ConnectionPlugin.remote)
                raise ValueError("Reconnect the Device")

        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        with self.assertRaises(harmos.FailureError):
            await owner._lifecycle("initialize")(object(), transport)
        closed.assert_awaited_once_with("connection")
        self.assertFalse(owner._resource_values)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
