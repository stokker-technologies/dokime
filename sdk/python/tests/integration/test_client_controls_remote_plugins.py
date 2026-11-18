# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Host-Free Bound Client Compositions"""

# === Imports === #

# Standard Library Imports
import asyncio
import unittest
from collections.abc import AsyncIterator

# Third Party Imports
from grpc import StatusCode
from grpc.aio import ServicerContext, server
from harmos._wire.host.v1 import plugins_pb2
from harmos._wire.host.v1.plugins_pb2_grpc import PluginsServicer, add_PluginsServicer_to_server

# Local Imports
import dokime_sdk.operations
from dokime_sdk._connection import connect
from dokime_sdk._errors import SdkError
from dokime_sdk._plugin import Plugin

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ClientStory(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.entries: dict[str, plugins_pb2.PluginEntry] = {}
        self.trace: list[str] = []
        self.observing = asyncio.Event()
        self.ready = asyncio.Event()
        self.fail_ready = False
        self.refuse_delete: set[str] = set()
        owner = self

        class Plugins(PluginsServicer):
            async def Create(
                self, request: plugins_pb2.PluginsCreateRequest, context: ServicerContext
            ) -> plugins_pb2.PluginsCreateResponse:
                owner.trace.append("Create")
                owner.entries[request.name] = plugins_pb2.PluginEntry(
                    name=request.name,
                    placement=request.placement,
                    value=request.value,
                    state="PLUGIN_STATE_CONNECTING",
                    revision=b"created",
                    description={"id": request.name, "version": "1.0.0"},
                )
                return plugins_pb2.PluginsCreateResponse(entry=owner.entries[request.name])

            async def Watch(
                self, request: plugins_pb2.PluginsWatchRequest, context: ServicerContext
            ) -> AsyncIterator[plugins_pb2.PluginsWatchResponse]:
                owner.trace.append("Watch")
                yield plugins_pb2.PluginsWatchResponse(snapshot={"entries": [owner.entries[request.name]]})
                owner.observing.set()
                await owner.ready.wait()
                entry = owner.entries[request.name]
                if owner.fail_ready:
                    entry.state = plugins_pb2.PLUGIN_STATE_UNAVAILABLE
                    entry.failure.CopyFrom(
                        type(entry.failure)(
                            classification="CLASSIFICATION_REFUSED",
                            code="setup_failed",
                            message="Setup Failed",
                            action="Repair Setup",
                            cause={"classification": "CLASSIFICATION_PERMANENT", "code": "device", "details": {"port": "1"}},
                        )
                    )
                else:
                    entry.state = plugins_pb2.PLUGIN_STATE_READY
                yield plugins_pb2.PluginsWatchResponse(change={"entry": entry})

            async def Get(self, request: plugins_pb2.PluginsGetRequest, context: ServicerContext) -> plugins_pb2.PluginsGetResponse:
                owner.trace.append("Get")
                if request.name not in owner.entries:
                    await context.abort(StatusCode.NOT_FOUND, "Entry Is Absent")
                return plugins_pb2.PluginsGetResponse(entry=owner.entries[request.name])

            async def Delete(
                self, request: plugins_pb2.PluginsDeleteRequest, context: ServicerContext
            ) -> plugins_pb2.PluginsDeleteResponse:
                owner.trace.append("Delete")
                if request.name in owner.refuse_delete:
                    await context.abort(StatusCode.FAILED_PRECONDITION, "Running Execution Refuses Detach")
                if request.expected_revision != owner.entries[request.name].revision:
                    await context.abort(StatusCode.ABORTED, "Revision Changed")
                del owner.entries[request.name]
                return plugins_pb2.PluginsDeleteResponse(receipt={})

        self.host = server()
        add_PluginsServicer_to_server(Plugins(), self.host)
        port = self.host.add_insecure_port("127.0.0.1:0")
        await self.host.start()
        self.addAsyncCleanup(self.host.stop, None)
        self.endpoint = f"http://127.0.0.1:{port}"
        self.client = await connect(self.endpoint)
        self.addAsyncCleanup(self.client.close)

    async def test_register_waits_for_ready_then_close_unregisters_only_owned_entries(self) -> None:
        class ExamplePlugin(Plugin):
            identifier = "example"
            version = "1.0.0"

        registration = asyncio.create_task(self.client.plugins.register(ExamplePlugin(), address="127.0.0.1:0", request_id="register"))
        await self.observing.wait()
        self.assertFalse(registration.done())
        self.ready.set()
        registered = await registration
        self.assertEqual((registered.identifier, registered.version, registered.state), ("example", "1.0.0", "Ready"))
        self.entries["other"] = plugins_pb2.PluginEntry(name="other", revision=b"other", state="PLUGIN_STATE_READY")
        await self.client.plugins.get("other")
        await registered.refresh()
        self.trace.clear()
        await self.client.close()
        await self.client.close()
        self.assertEqual(self.trace, ["Get", "Delete"])
        self.assertEqual(list(self.entries), ["other"])
        self.assertIsNone(registered._endpoint)

    async def test_lookup_of_an_owned_entry_preserves_its_sidecar_lifetime(self) -> None:
        class LookedUpPlugin(Plugin):
            identifier = "lookup"
            version = "1.0.0"
            label = "Display Label"

        self.ready.set()
        registered = await self.client.plugins.register(LookedUpPlugin(), address="127.0.0.1:0")
        looked_up = await self.client.plugins.get(registered.identifier)
        self.assertIs(looked_up, registered)
        self.assertEqual(looked_up.label, "Display Label")
        await looked_up.unregister()
        self.assertIsNone(registered._endpoint)
        self.assertEqual(self.entries, {})
        self.assertEqual(self.client.plugins._owned, {})

    async def test_close_unwinds_registration_that_is_waiting_for_readiness(self) -> None:
        class PendingPlugin(Plugin):
            identifier = "pending"
            version = "1.0.0"

        registration = asyncio.create_task(self.client.plugins.register(PendingPlugin(), address="127.0.0.1:0"))
        await self.observing.wait()
        await self.client.close()
        with self.assertRaises(asyncio.CancelledError):
            await registration
        self.assertEqual(self.entries, {})
        self.assertEqual(self.client.plugins._owned, {})

    async def test_close_reports_refusal_attempts_every_entry_and_closes_the_channel_last(self) -> None:
        class RunningPlugin(Plugin):
            identifier = "running"
            version = "1.0.0"

        class IdlePlugin(Plugin):
            identifier = "idle"
            version = "1.0.0"

        client = await connect(self.endpoint)
        self.addAsyncCleanup(client._transport.close)
        self.ready.set()
        running = await client.plugins.register(RunningPlugin(), address="127.0.0.1:0")
        assert running._endpoint is not None
        self.addAsyncCleanup(running._endpoint.close)
        await client.plugins.register(IdlePlugin(), address="127.0.0.1:0")
        self.refuse_delete.add("running")
        self.trace.clear()
        with self.assertRaises(SdkError) as failed:
            await client.close()
        self.assertEqual(self.trace, ["Get", "Delete", "Get", "Delete"])
        self.assertEqual(list(self.entries), ["running"])
        self.assertIsNotNone(running._endpoint)
        assert running._endpoint is not None
        await running._endpoint.close()
        with self.assertRaises(SdkError) as repeated:
            await client.close()
        self.assertIs(failed.exception, repeated.exception)

    async def test_reported_readiness_failure_keeps_causes_and_unwinds(self) -> None:
        class FailingPlugin(Plugin):
            identifier = "failing"
            version = "1.0.0"

        self.fail_ready = True
        self.ready.set()
        with self.assertRaises(SdkError) as failed:
            await self.client.plugins.register(FailingPlugin(), address="127.0.0.1:0", request_id="register-failure")
        self.assertEqual(failed.exception.request_id, "register-failure")
        cause = failed.exception.failure.cause
        self.assertIsNotNone(cause)
        assert cause is not None
        self.assertEqual(cause.details, {"port": "1"})
        self.assertEqual(self.entries, {})
        self.assertEqual(self.trace[-2:], ["Get", "Delete"])

    async def test_cancelling_registration_releases_its_sidecar_and_entry(self) -> None:
        class WaitingPlugin(Plugin):
            identifier = "waiting"
            version = "1.0.0"

        registration = asyncio.create_task(self.client.plugins.register(WaitingPlugin(), address="127.0.0.1:0"))
        await self.observing.wait()
        registration.cancel()
        with self.assertRaises(asyncio.CancelledError):
            await registration
        self.assertEqual(self.entries, {})
        self.assertEqual(self.client.plugins._owned, {})

    async def test_registration_and_serve_share_the_lifecycle_activation_refusal(self) -> None:
        class LifecyclePlugin(Plugin):
            version = "1.0.0"

            async def initialize(self, ctx: "dokime_sdk.operations.Context") -> None:
                raise AssertionError("Library Activation Must Own This Hook")

        author = LifecyclePlugin()
        with self.assertRaises(ValueError):
            await self.client.plugins.register(author, address="127.0.0.1:0")
        with self.assertRaises(ValueError):
            await author.serve("127.0.0.1:0")
        self.assertEqual(self.trace, [])
        self.assertEqual(self.entries, {})


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
