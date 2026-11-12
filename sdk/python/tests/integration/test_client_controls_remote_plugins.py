# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Client Adds, Watches And Removes A Remote Plugin Through Protobuf RPCs."""

# === Imports === #

# Standard Library Imports
import unittest

import grpc
from google.protobuf import descriptor_pb2, descriptor_pool

# Third Party Imports
import dokime

# Local Imports
from dokime.v1 import message, message_type

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ClientStory(unittest.IsolatedAsyncioTestCase):
    async def test_add_watch_remove_and_close(self):
        # Define The Test Server's Own Service Without Adding Production Protos.
        definition = descriptor_pb2.FileDescriptorProto(
            name="dokime/testing/client.proto", package="dokime.testing", syntax="proto3", dependency=["dokime/v1/plugins.proto"]
        )
        service = definition.service.add(name="Plugins")
        service.method.add(name="Add", input_type=".dokime.v1.AddRemotePlugin", output_type=".dokime.v1.Plugin")
        service.method.add(name="Watch", input_type=".dokime.v1.ListPlugins", output_type=".dokime.v1.Plugin", server_streaming=True)
        service.method.add(name="Remove", input_type=".dokime.v1.RemovePlugin", output_type=".dokime.v1.RemovePluginResponse")
        descriptor_pool.Default().Add(definition)
        descriptor = descriptor_pool.Default().FindServiceByName("dokime.testing.Plugins")
        entries = {}
        authorization = []

        async def add(request, context):
            authorization.append(dict(context.invocation_metadata()).get("authorization"))
            if not request.url.startswith("http://"):
                await context.abort(grpc.StatusCode.INVALID_ARGUMENT, "Plugin URL is required")
            entry = message("Plugin", entry=f"entry-{len(entries)}", remote={"url": request.url}, state=3)
            entries[entry.entry] = entry
            return entry

        async def watch(request, context):
            for entry in entries.values():
                yield entry

        async def remove(request, context):
            if entries.pop(request.entry, None) is None:
                await context.abort(grpc.StatusCode.NOT_FOUND, "Plugin entry is absent")
            return message("RemovePluginResponse")

        # Serve The Fake Host In Process And Connect With The Public Client.
        handlers = {
            "Add": grpc.unary_unary_rpc_method_handler(
                add,
                request_deserializer=message_type("AddRemotePlugin").FromString,
                response_serializer=message_type("Plugin").SerializeToString,
            ),
            "Watch": grpc.unary_stream_rpc_method_handler(
                watch,
                request_deserializer=message_type("ListPlugins").FromString,
                response_serializer=message_type("Plugin").SerializeToString,
            ),
            "Remove": grpc.unary_unary_rpc_method_handler(
                remove,
                request_deserializer=message_type("RemovePlugin").FromString,
                response_serializer=message_type("RemovePluginResponse").SerializeToString,
            ),
        }
        server = grpc.aio.server()
        server.add_generic_rpc_handlers([grpc.method_handlers_generic_handler(descriptor.full_name, handlers)])
        port = server.add_insecure_port("127.0.0.1:0")
        await server.start()
        self.addAsyncCleanup(server.stop, None)
        client = await dokime.connect(f"http://127.0.0.1:{port}", access_key="test-key")
        self.addAsyncCleanup(client.close)

        # Add A Remote Entry And Inspect The Streamed Typed State.
        entry = await client.call(message("AddRemotePlugin", url="http://127.0.0.1:7001"), method=descriptor.methods_by_name["Add"])
        self.assertEqual((entry.remote.url, entry.state), ("http://127.0.0.1:7001", 3))
        self.assertEqual(authorization, ["Bearer test-key"])
        watched = [item async for item in client.call(message("ListPlugins"), method=descriptor.methods_by_name["Watch"])]
        self.assertEqual([item.entry for item in watched], [entry.entry])

        # Preserve Server Refusals And Validate The Input Descriptor.
        with self.assertRaises(grpc.aio.AioRpcError) as failed:
            await client.call(message("AddRemotePlugin", url="invalid"), method=descriptor.methods_by_name["Add"])
        self.assertEqual(failed.exception.code(), grpc.StatusCode.INVALID_ARGUMENT)
        with self.assertRaises(TypeError):
            client.call(message("RemovePlugin"), method=descriptor.methods_by_name["Add"])

        # Remove The Entry And Observe Its Absence Before Closing.
        await client.call(message("RemovePlugin", entry=entry.entry), method=descriptor.methods_by_name["Remove"])
        self.assertEqual([item async for item in client.call(message("ListPlugins"), method=descriptor.methods_by_name["Watch"])], [])
        await client.close()
        await client.close()
        with self.assertRaises(RuntimeError):
            client.call(message("ListPlugins"), method=descriptor.methods_by_name["Watch"])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
