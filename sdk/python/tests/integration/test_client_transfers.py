# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Owned Client Uploads and Artifact Attachments"""

# === Imports === #

# Standard Library Imports
import unittest
from hashlib import sha256

# Third Party Imports
from grpc import StatusCode
from grpc.aio import ServicerContext, server

# Local Imports
from dokime_sdk._client import Cycle
from dokime_sdk._connection import connect
from dokime_sdk._errors import SdkError
from dokime_sdk._wire.host.v1.content import artifacts_pb2
from dokime_sdk._wire.host.v1.content.artifacts_pb2_grpc import ArtifactsServicer, add_ArtifactsServicer_to_server
from dokime_sdk._wire.host.v1.cycles_pb2 import CycleSummary
from dokime_sdk._wire.host.v1.files import uploads_pb2
from dokime_sdk._wire.host.v1.files.uploads_pb2_grpc import UploadsServicer, add_UploadsServicer_to_server

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ClientTransferStory(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.identities: list[str] = []
        self.bytes = bytearray()
        self.fail_append = False
        self.fail_attach = False
        owner = self

        class Uploads(UploadsServicer):
            async def Create(
                self, request: uploads_pb2.UploadsCreateRequest, context: ServicerContext
            ) -> uploads_pb2.UploadsCreateResponse:
                owner.identities.append(request.request_id)
                return uploads_pb2.UploadsCreateResponse(upload={"id": "upload", "writer_generation": 7}, chunk_bytes=2, message_bytes=100)

            async def Append(
                self, request: uploads_pb2.UploadsAppendRequest, context: ServicerContext
            ) -> uploads_pb2.UploadsAppendResponse:
                owner.identities.append(request.request_id)
                if owner.fail_append:
                    await context.abort(StatusCode.UNAVAILABLE, "Upload Lost Its Reply")
                if (
                    request.offset != len(owner.bytes)
                    or request.upload.accepted_length != len(owner.bytes)
                    or request.upload.writer_generation != 7
                ):
                    await context.abort(StatusCode.ABORTED, "Wrong Upload Fence or Offset")
                owner.bytes.extend(request.data)
                reply = uploads_pb2.UploadsAppendResponse(upload=request.upload)
                reply.upload.accepted_length = len(owner.bytes)
                return reply

            async def Publish(
                self, request: uploads_pb2.UploadsPublishRequest, context: ServicerContext
            ) -> uploads_pb2.UploadsPublishResponse:
                owner.identities.append(request.request_id)
                if request.upload.accepted_length != len(owner.bytes):
                    await context.abort(StatusCode.ABORTED, "Wrong Upload Fence")
                return uploads_pb2.UploadsPublishResponse(upload={"length": len(owner.bytes), "sha256": sha256(owner.bytes).digest()})

            async def Delete(
                self, request: uploads_pb2.UploadsDeleteRequest, context: ServicerContext
            ) -> uploads_pb2.UploadsDeleteResponse:
                owner.identities.append(request.request_id)
                return uploads_pb2.UploadsDeleteResponse(upload=request.upload)

        class Artifacts(ArtifactsServicer):
            async def Create(
                self, request: artifacts_pb2.ArtifactsCreateRequest, context: ServicerContext
            ) -> artifacts_pb2.ArtifactsCreateResponse:
                owner.identities.append(request.request_id)
                if owner.fail_attach:
                    await context.abort(StatusCode.FAILED_PRECONDITION, "Execution Is Sealed")
                return artifacts_pb2.ArtifactsCreateResponse(
                    artifact={
                        "artifact_id": "artifact",
                        "filename": request.filename,
                        "content_type": request.content_type,
                        "size_bytes": request.file.length,
                        "sha256": request.file.sha256.hex(),
                        "scope": request.scope,
                    }
                )

        host = server()
        add_UploadsServicer_to_server(Uploads(), host)
        add_ArtifactsServicer_to_server(Artifacts(), host)
        port = host.add_insecure_port("127.0.0.1:0")
        await host.start()
        self.addAsyncCleanup(host.stop, None)
        self.client = await connect(f"http://127.0.0.1:{port}")
        self.addAsyncCleanup(self.client.close)
        self.cycle = Cycle(self.client, CycleSummary(cycle_id="cycle"))

    async def test_upload_preserves_acknowledged_fences_and_child_identities(self) -> None:
        artifact = await self.cycle.artifact("capture.bin", b"abcde", mime="test/data", request_id="attach")
        self.assertEqual(
            self.identities, ["attach/create", "attach/append/0", "attach/append/2", "attach/append/4", "attach/publish", "attach/attach"]
        )
        self.assertEqual(
            (artifact.name, artifact.mime, artifact.size_bytes, artifact.sha256, artifact.cycle),
            ("capture.bin", "test/data", 5, sha256(b"abcde").digest(), "cycle"),
        )
        self.assertIsNone(artifact.execution)

    async def test_failed_upload_abandons_without_changing_the_request_identity(self) -> None:
        self.fail_append = True
        with self.assertRaises(SdkError) as failed:
            await self.cycle.artifact("capture.bin", b"abcde", request_id="failed")
        self.assertEqual(failed.exception.request_id, "failed")
        self.assertEqual(self.identities, ["failed/create", "failed/append/0", "failed/abandon"])

    async def test_failed_attachment_never_deletes_a_published_file(self) -> None:
        self.fail_attach = True
        with self.assertRaises(SdkError) as failed:
            await self.cycle.artifact("capture.bin", b"abcde", request_id="published")
        self.assertEqual(failed.exception.request_id, "published")
        self.assertEqual(self.identities[-2:], ["published/publish", "published/attach"])
        self.assertFalse(any(identity.endswith("/abandon") for identity in self.identities))


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
