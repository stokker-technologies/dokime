# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Report Previews Read One Verified File Without Publishing It."""

# === Imports === #

# Standard Library Imports
import unittest
from collections.abc import AsyncIterator
from hashlib import sha256
from importlib import import_module
from typing import Never

# Third Party Imports
import grpc
from grpc.aio import ServicerContext

# Local Imports
from dokime_sdk._connection import connect
from dokime_sdk._errors import SdkError
from dokime_sdk._generated.client_types import ReportsCreateResponse
from dokime_sdk._transfers import download_report
from dokime_sdk._wire.host.v1.files_pb2 import (
    FilesDeleteRequest,
    FilesGetRequest,
    FilesGetResponse,
    FilesReadRequest,
    FilesReadResponse,
    FilesWatchRequest,
    FilesWatchResponse,
)
from dokime_sdk._wire.host.v1.reports_pb2 import ReportsCreateRequest, ReportsGetRequest, ReportsPublishRequest

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ReportStory(unittest.IsolatedAsyncioTestCase):
    async def test_preview_reads_verified_bytes_and_refuses_changed_files(self) -> None:
        report_rpc = import_module("dokime_sdk._wire.host.v1.reports_pb2_grpc")
        storage_rpc = import_module("dokime_sdk._wire.host.v1.files_pb2_grpc")
        data = bytes(range(7))
        digest = sha256(data).digest()
        mode = ["valid"]
        offsets = []
        published = []

        class Reports(report_rpc.ReportsServicer):
            async def Get(self, request: ReportsGetRequest, context: ServicerContext) -> Never:
                await context.abort(grpc.StatusCode.UNIMPLEMENTED, "Get Is Outside This Fixture")

            async def Create(self, request: ReportsCreateRequest, context: ServicerContext) -> ReportsCreateResponse:
                return ReportsCreateResponse(file={"length": len(data), "sha256": digest})

            async def Publish(self, request: ReportsPublishRequest, context: ServicerContext) -> Never:
                published.append(request)
                await context.abort(grpc.StatusCode.UNIMPLEMENTED, "Publish Report: Unexpected Mutation")

        class Files(storage_rpc.FilesServicer):
            async def Get(self, request: FilesGetRequest, context: ServicerContext) -> FilesGetResponse:
                return FilesGetResponse(chunk_bytes=2)

            def Watch(self, request: FilesWatchRequest, context: ServicerContext) -> AsyncIterator[FilesWatchResponse]:
                return super().Watch(request, context)

            async def Delete(self, request: FilesDeleteRequest, context: ServicerContext) -> Never:
                await context.abort(grpc.StatusCode.UNIMPLEMENTED, "Delete Is Outside This Fixture")

            async def Read(self, request: FilesReadRequest, context: ServicerContext) -> FilesReadResponse:
                offsets.append(request.offset)
                chunk = data[request.offset : request.offset + 2]
                reply_digest = digest
                if mode[0] == "identity":
                    reply_digest = sha256(b"foreign").digest()
                elif mode[0] == "corrupt":
                    chunk = bytes([255]) * len(chunk)
                elif mode[0] == "gap":
                    chunk = b""
                return FilesReadResponse(data=chunk, total_length=len(data), sha256=reply_digest)

        server = grpc.aio.server()
        report_rpc.add_ReportsServicer_to_server(Reports(), server)
        storage_rpc.add_FilesServicer_to_server(Files(), server)
        port = server.add_insecure_port("127.0.0.1:0")
        await server.start()
        self.addAsyncCleanup(server.stop, None)
        client = await connect(f"http://127.0.0.1:{port}")
        self.addAsyncCleanup(client.close)

        self.assertEqual(await download_report(client._transport, "execution", document="evidence", max_bytes=2), data)
        self.assertEqual(offsets, [0, 2, 4, 6])
        for value in ("identity", "corrupt", "gap"):
            mode[0] = value
            with self.assertRaises(SdkError):
                await download_report(client._transport, "execution", document="evidence", max_bytes=2)
        self.assertEqual(published, [])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
