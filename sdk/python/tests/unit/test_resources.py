# SPDX-License-Identifier: Apache-2.0

"""Plugin Declarations Own Secret Reads and Activation Resource Lifetimes."""

# === Imports === #

# Standard Library Imports
import asyncio
from unittest import IsolatedAsyncioTestCase
from unittest.mock import AsyncMock, Mock

# Third Party Imports
import harmos
from harmos import CLASSIFICATION_REFUSED, plugin
from harmos._wire.model.v1.failure_pb2 import Failure
from harmos._wire.plugin.v1.invocation_pb2 import Request

# Local Imports
from dokime_sdk import Client, Plugin, RegisteredPlugin
from dokime_sdk._generated import client_types
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsGetResponse
from dokime_sdk.operations import Context, OperationError, Resource, Secret

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ResourceTests(IsolatedAsyncioTestCase):
    async def test_secret_identifiers_return_text_and_name_the_recovery_action(self) -> None:
        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            secrets = (Secret("api_token", "Remote API Token"),)

        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        transport._call = AsyncMock(return_value=SecretsGetResponse(secret=b"token"))
        context = Context(transport, owner)
        self.assertEqual(await context.secret("api_token"), "token")
        transport._call.assert_awaited_once()

        for response in (b"", b"\xff"):
            transport._call.return_value = SecretsGetResponse(secret=response)
            with self.assertRaises(OperationError) as raised:
                await context.secret("api_token")
            self.assertEqual(raised.exception.value.classification, CLASSIFICATION_REFUSED)
            self.assertIn("Remote API Token", raised.exception.value.action)
            self.assertIn("then Retry", raised.exception.value.action)

        missing = Failure(code="NOT_FOUND", classification=harmos.CLASSIFICATION_REFUSED)
        transport._call.side_effect = harmos.OperationError(Failure(cause=missing))
        with self.assertRaises(OperationError) as raised:
            await context.secret("api_token")
        self.assertEqual(raised.exception.value.action, "Store the Remote API Token secret for this plugin, then Retry")

        failure = Failure(code="denied", classification=harmos.CLASSIFICATION_REFUSED, action="Request Access")
        transport._call.side_effect = harmos.OperationError(failure)
        with self.assertRaises(harmos.OperationError) as raised_host:
            await context.secret("api_token")
        self.assertEqual(raised_host.exception.value, failure)
        with self.assertRaises(ValueError):
            await context.secret("foreign")

    async def test_typed_resources_open_once_and_reopen_after_reactivation(self) -> None:
        entered, release = asyncio.Event(), asyncio.Event()
        opened: list[Context] = []
        closed: list[Resource] = []

        class RemoteApi(Resource):
            async def open(self, ctx: Context) -> None:
                opened.append(ctx)
                entered.set()
                await release.wait()

            async def close(self) -> None:
                closed.append(self)

        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            resources = (RemoteApi,)

        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        context = Context(transport, owner)
        first = asyncio.create_task(context.resource(RemoteApi))
        await entered.wait()
        second = asyncio.create_task(context.resource(RemoteApi))
        release.set()
        api, same = await asyncio.gather(first, second)
        self.assertIsInstance(api, RemoteApi)
        self.assertIs(api, same)
        self.assertEqual(opened, [context])
        finalize = owner._lifecycle("finalize")
        await finalize(object(), transport)
        await finalize(object(), transport)
        self.assertEqual(closed, [api])
        with self.assertRaises(OperationError):
            await context.resource(RemoteApi)
        await owner._lifecycle("initialize")(object(), transport)
        restarted = await context.resource(RemoteApi)
        self.assertIsNot(restarted, api)
        self.assertEqual(len(opened), 2)
        await finalize(object(), transport)
        self.assertEqual(closed, [api, restarted])

    async def test_registration_release_closes_typed_resources_without_host_finalization(self) -> None:
        closed = AsyncMock()

        class RemoteApi(Resource):
            async def close(self) -> None:
                await closed()

        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            resources = (RemoteApi,)

        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        await Context(transport, owner).resource(RemoteApi)
        client = Mock(spec=Client)
        client.plugins = Mock()
        client.plugins._bindings = {}
        registered = RegisteredPlugin._bind(client, client_types.host.PluginEntry(name=owner.identifier), owner=owner)
        await registered._release()
        await registered._release()
        closed.assert_awaited_once_with()

    async def test_resource_types_are_declared_once_and_foreign_types_are_rejected(self) -> None:
        class RemoteApi(Resource):
            pass

        class ForeignApi(Resource):
            pass

        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            resources = (RemoteApi, RemoteApi)

        with self.assertRaisesRegex(ValueError, "Each Resource Class Once"):
            ConnectionPlugin()
        ConnectionPlugin.resources = (RemoteApi,)
        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        with self.assertRaisesRegex(ValueError, "Declared on This Plugin"):
            await Context(transport, owner).resource(ForeignApi)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
