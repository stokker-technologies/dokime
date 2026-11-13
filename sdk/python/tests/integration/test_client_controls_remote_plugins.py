# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Client Adds, Lists And Removes A Remote Plugin Through Generated RPCs."""

# === Imports === #

# Standard Library Imports
import unittest
from importlib import import_module

# Third Party Imports
import grpc
from google.protobuf import descriptor_pool

# Local Imports
import dokime
from dokime.v1 import message

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ClientStory(unittest.IsolatedAsyncioTestCase):
    async def test_add_list_remove_and_close(self):
        # Implement the Authoritative Services in Process.
        descriptor = descriptor_pool.Default().FindServiceByName("dokime.v1.Plugins")
        status_rpc = import_module("harmos.v1.status_pb2_grpc")
        status_wire = import_module("harmos.v1.status_pb2")
        logs_rpc = import_module("harmos.v1.status.logs_pb2_grpc")
        logs_wire = import_module("harmos.v1.status.logs_pb2")
        plugins_rpc = import_module("dokime.v1.client_plugins_pb2_grpc")
        entries = {}
        authorization = []

        class Plugins(plugins_rpc.PluginsServicer):
            async def Add(self, request, context):
                authorization.append(dict(context.invocation_metadata()).get("authorization"))
                if not request.url.startswith("http://"):
                    await context.abort(grpc.StatusCode.INVALID_ARGUMENT, "Plugin URL is required")
                entry = message("Plugin", entry=f"entry-{len(entries)}", remote={"url": request.url}, state=3)
                entries[entry.entry] = entry
                return entry

            async def List(self, request, context):
                return message("PluginEntries", plugins=entries.values())

            async def Remove(self, request, context):
                if entries.pop(request.entry, None) is None:
                    await context.abort(grpc.StatusCode.NOT_FOUND, "Plugin entry is absent")
                return message("RemovePluginResponse")

        class Status(status_rpc.StatusServicer):
            async def Get(self, request, context):
                if dict(context.invocation_metadata()).get("authorization") != "Bearer test-key":
                    await context.abort(grpc.StatusCode.UNAUTHENTICATED, "Read Status: Supply Credentials")
                return status_wire.StatusGetResponse(
                    status=status_wire.HostStatus(identity=status_wire.HostIdentity(host_id="bench"), available=True)
                )

        class Logs(logs_rpc.LogsServicer):
            async def Watch(self, request, context):
                authorization.append(dict(context.invocation_metadata()).get("authorization"))
                yield logs_wire.LogsWatchResponse()
                yield logs_wire.LogsWatchResponse(events=[logs_wire.Event(message="ready")])

        # Serve The Fake Host In Process And Connect With The Public Client.
        server = grpc.aio.server()
        plugins_rpc.add_PluginsServicer_to_server(Plugins(), server)
        logs_rpc.add_LogsServicer_to_server(Logs(), server)
        status_rpc.add_StatusServicer_to_server(Status(), server)
        port = server.add_insecure_port("127.0.0.1:0")
        await server.start()
        self.addAsyncCleanup(server.stop, None)
        client = await dokime.connect(f"http://127.0.0.1:{port}", access_key="test-key", expected_backend_id="bench")
        self.addAsyncCleanup(client.close)

        # Refuse a Paired Identity That Does Not Match the Served Host.
        with self.assertRaises(ValueError):
            await dokime.connect(f"http://127.0.0.1:{port}", access_key="test-key", expected_backend_id="foreign")

        # Add a Remote Entry and List Its Canonical Placement Through Generated Stubs.
        entry = await client.plugins.add("http://127.0.0.1:7001")
        self.assertEqual((entry.remote.url, entry.state), ("http://127.0.0.1:7001", 3))
        self.assertEqual(authorization, ["Bearer test-key"])
        listed = await client.stubs["dokime.v1.Plugins"].List(message("ListPlugins"))
        self.assertEqual(listed.plugins, [entry])
        self.assertEqual((await client.plugins.list()).plugins, [entry])

        # Drain the Library Watch Through the Composed Root and Retain Credentials.
        frames = [item async for item in client.status.logs.watch(logs_wire.LogsWatchRequest(max_events=8, max_bytes=4096))]
        self.assertEqual(list(frames[0].events), [])
        self.assertEqual(frames[1].events[0].message, "ready")
        self.assertEqual(authorization[-1], "Bearer test-key")

        # Preserve Server Refusals And Validate The Input Descriptor.
        with self.assertRaises(grpc.aio.AioRpcError) as failed:
            await client.call(message("AddRemotePlugin", url="invalid"), method=descriptor.methods_by_name["Add"])
        self.assertEqual(failed.exception.code(), grpc.StatusCode.INVALID_ARGUMENT)
        with self.assertRaises(TypeError):
            client.call(message("RemovePlugin"), method=descriptor.methods_by_name["Add"])

        # Remove The Entry And Observe Its Absence Before Closing.
        await client.plugins.remove(entry.entry)
        self.assertEqual((await client.plugins.list()).plugins, [])
        await client.close()
        await client.close()
        with self.assertRaises(RuntimeError):
            client.call(message("ListPlugins"))


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
