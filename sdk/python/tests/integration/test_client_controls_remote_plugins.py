# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Client Creates, Lists and Deletes a Composed Library Plugin Entry."""

# === Imports === #

import unittest
from collections.abc import AsyncIterator

from grpc import StatusCode
from grpc.aio import AioRpcError, ServicerContext, server
from harmos._wire.host.v1.plugins_pb2 import (
    PluginsCreateRequest,
    PluginsCreateResponse,
    PluginsDeleteRequest,
    PluginsDeleteResponse,
    PluginsListRequest,
    PluginsListResponse,
)
from harmos._wire.host.v1.plugins_pb2_grpc import PluginsServicer, add_PluginsServicer_to_server
from harmos._wire.host.v1.status.logs_pb2 import Event, LogsWatchRequest, LogsWatchResponse
from harmos._wire.host.v1.status.logs_pb2_grpc import LogsServicer, add_LogsServicer_to_server
from harmos._wire.host.v1.status_pb2 import HostIdentity, HostStatus, StatusGetRequest, StatusGetResponse
from harmos._wire.host.v1.status_pb2_grpc import StatusServicer, add_StatusServicer_to_server
from harmos.client import PluginEntry, PluginPlacement, PluginRemote, PluginValue

from dokime.connection import connect

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ClientStory(unittest.IsolatedAsyncioTestCase):
    async def test_create_list_delete_and_close(self) -> None:
        entries: dict[str, PluginEntry] = {}
        authorization: list[str | bytes | None] = []
        identities: list[str] = []

        class Plugins(PluginsServicer):
            async def Create(self, request: PluginsCreateRequest, context: ServicerContext) -> PluginsCreateResponse:
                authorization.append(dict(context.invocation_metadata() or ()).get("authorization"))
                if not request.placement.remote.url.startswith("http://"):
                    await context.abort(StatusCode.INVALID_ARGUMENT, "Create Plugin: Supply a Remote URL")
                identities.append(request.request_id)
                entry = entries.setdefault(
                    request.name, PluginEntry(name=request.name, placement=request.placement, value=request.value, revision=b"first")
                )
                return PluginsCreateResponse(entry=entry)

            async def List(self, request: PluginsListRequest, context: ServicerContext) -> PluginsListResponse:
                return PluginsListResponse(entries=entries.values())

            async def Delete(self, request: PluginsDeleteRequest, context: ServicerContext) -> PluginsDeleteResponse:
                if entries.pop(request.name, None) is None:
                    await context.abort(StatusCode.NOT_FOUND, "Delete Plugin: Entry Is Absent")
                return PluginsDeleteResponse(receipt={})

        class Status(StatusServicer):
            async def Get(self, request: StatusGetRequest, context: ServicerContext) -> StatusGetResponse:
                if dict(context.invocation_metadata() or ()).get("authorization") != "Bearer test-key":
                    await context.abort(StatusCode.UNAUTHENTICATED, "Read Status: Supply Credentials")
                return StatusGetResponse(status=HostStatus(identity=HostIdentity(host_id="bench"), available=True))

        class Logs(LogsServicer):
            async def Watch(self, request: LogsWatchRequest, context: ServicerContext) -> AsyncIterator[LogsWatchResponse]:
                authorization.append(dict(context.invocation_metadata() or ()).get("authorization"))
                yield LogsWatchResponse()
                yield LogsWatchResponse(events=[Event(sequence=1)])

        # Start the Host Fixture and Connect Through the Public Client.
        host = server()
        add_PluginsServicer_to_server(Plugins(), host)
        add_StatusServicer_to_server(Status(), host)
        add_LogsServicer_to_server(Logs(), host)
        port = host.add_insecure_port("127.0.0.1:0")
        await host.start()
        self.addAsyncCleanup(host.stop, None)
        client = await connect(f"http://127.0.0.1:{port}", access_key="test-key", expected_backend_id="bench")
        self.addAsyncCleanup(client.close)

        # Refuse a Foreign Paired Host.
        with self.assertRaises(ValueError):
            await connect(f"http://127.0.0.1:{port}", access_key="test-key", expected_backend_id="foreign")

        # Create Idempotently and Read the Canonical Library Entry.
        placement = PluginPlacement(remote=PluginRemote(url="http://127.0.0.1:7001"))
        created = await client.plugins.create("example", placement, PluginValue(), request_id="create-example")
        repeated = await client.plugins.create("example", placement, PluginValue(), request_id="create-example")
        self.assertEqual(created.entry, repeated.entry)
        self.assertEqual(list((await client.plugins.list()).entries), [created.entry])
        self.assertEqual(identities, ["create-example", "create-example"])
        self.assertEqual(authorization, ["Bearer test-key", "Bearer test-key"])

        # Drain a Library Watch Through the Same Authenticated Connection.
        frames = [frame async for frame in client.status.logs.watch(max_events=8, max_bytes=4096)]
        self.assertEqual(len(frames), 2)
        self.assertEqual(frames[1].events[0].sequence, 1)
        self.assertEqual(authorization[-1], "Bearer test-key")

        # Preserve Transport Refusals with Their Causes.
        with self.assertRaises(AioRpcError) as failed:
            await client.plugins.create("invalid", PluginPlacement(remote=PluginRemote(url="invalid")), PluginValue())
        self.assertEqual(failed.exception.code(), StatusCode.INVALID_ARGUMENT)
        self.assertIsInstance(failed.exception.__cause__, AioRpcError)

        # Delete the Entry and Close the Owned Connection Twice.
        await client.plugins.delete(created.entry.name, created.entry.revision)
        self.assertEqual(list((await client.plugins.list()).entries), [])
        await client.close()
        await client.close()
        with self.assertRaises((ValueError, AioRpcError)):
            await client.plugins.list()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
