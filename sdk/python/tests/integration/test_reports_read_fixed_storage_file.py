# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Report Previews Read One Verified File Without Publishing It."""

# === Imports === #

# Standard Library Imports
import unittest
from hashlib import sha256
from importlib import import_module

# Third Party Imports
import grpc

# Local Imports
import dokime
from dokime.v1 import message

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ReportStory(unittest.IsolatedAsyncioTestCase):
    async def test_preview_reads_verified_bytes_and_refuses_changed_files(self) -> None:
        report_rpc = import_module("dokime.client.v1.reports_pb2_grpc")
        storage_rpc = import_module("dokime.client.v1.files_pb2_grpc")
        storage_wire = import_module("dokime.client.v1.files_pb2")
        data = bytes(range(7))
        digest = sha256(data).digest()
        mode = ["valid"]
        offsets = []
        published = []

        class Reports(report_rpc.ReportsServicer):
            async def Create(self, request, context):
                return message("dokime.client.v1.ReportsCreateResponse", file={"length": len(data), "sha256": digest})

            async def Publish(self, request, context):
                published.append(request)
                await context.abort(grpc.StatusCode.UNIMPLEMENTED, "Publish Report: Unexpected Mutation")

        class Files(storage_rpc.FilesServicer):
            async def Read(self, request, context):
                offsets.append(request.offset)
                chunk = data[request.offset : request.offset + 2]
                reply_digest = digest
                if mode[0] == "identity":
                    reply_digest = sha256(b"foreign").digest()
                elif mode[0] == "corrupt":
                    chunk = bytes([255]) * len(chunk)
                elif mode[0] == "gap":
                    chunk = b""
                return storage_wire.FilesReadResponse(data=chunk, total_length=len(data), sha256=reply_digest)

        server = grpc.aio.server()
        report_rpc.add_ReportsServicer_to_server(Reports(), server)
        storage_rpc.add_FilesServicer_to_server(Files(), server)
        port = server.add_insecure_port("127.0.0.1:0")
        await server.start()
        self.addAsyncCleanup(server.stop, None)
        client = await dokime.connect(f"http://127.0.0.1:{port}")
        self.addAsyncCleanup(client.close)

        self.assertEqual(await client.reports.download("execution", document="evidence"), data)
        self.assertEqual(offsets, [0, 2, 4, 6])
        for value in ("identity", "corrupt", "gap"):
            mode[0] = value
            with self.assertRaises(RuntimeError):
                await client.reports.download("execution", document="evidence")
        self.assertEqual(published, [])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
