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
from fractions import Fraction
from hashlib import sha256
from typing import ClassVar

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet
from google.protobuf.message import Message
from google.protobuf.wrappers_pb2 import BoolValue
from grpc import StatusCode
from grpc.aio import Channel, ServicerContext, server
from harmos import OperationPolicy, plugin
from harmos._wire.host.v1 import invocations_pb2, operations_pb2
from harmos._wire.plugin.v1.invocation_pb2 import Request

# Local Imports
from dokime_sdk import Plugin, Suite
from dokime_sdk._client import Case, Cycle, Execution
from dokime_sdk._connection import connect
from dokime_sdk._errors import SdkError
from dokime_sdk._generated import client_types
from dokime_sdk._wire.host.v1 import documents_pb2, evidence_pb2
from dokime_sdk._wire.host.v1.content import artifacts_pb2
from dokime_sdk._wire.host.v1.content.artifacts_pb2_grpc import ArtifactsServicer, add_ArtifactsServicer_to_server
from dokime_sdk._wire.host.v1.cycles_pb2 import CycleSummary
from dokime_sdk._wire.host.v1.documents_pb2_grpc import DocumentsServicer, add_DocumentsServicer_to_server
from dokime_sdk._wire.host.v1.evidence import records_pb2
from dokime_sdk._wire.host.v1.evidence.records_pb2_grpc import RecordsServicer, add_RecordsServicer_to_server
from dokime_sdk._wire.host.v1.evidence_pb2_grpc import EvidenceServicer, add_EvidenceServicer_to_server
from dokime_sdk._wire.host.v1.executions import schemas_pb2
from dokime_sdk._wire.host.v1.executions.schemas_pb2_grpc import SchemasServicer, add_SchemasServicer_to_server
from dokime_sdk._wire.host.v1.files import uploads_pb2
from dokime_sdk._wire.host.v1.files.uploads_pb2_grpc import UploadsServicer, add_UploadsServicer_to_server
from dokime_sdk._wire.host.v1.invocations_pb2_grpc import InvocationsServicer, add_InvocationsServicer_to_server
from dokime_sdk._wire.host.v1.operations_pb2_grpc import OperationsServicer, add_OperationsServicer_to_server
from dokime_sdk.operations import BoundOperation, Operation, Visibility
from dokime_sdk.templates import Context, Document, Section, Termination, TestTemplate, observer, record, stream

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ScopedCarrier(plugin.Context):
    def __init__(self, channel: Channel) -> None:
        super().__init__(Request(scope="execution", attempt=1), "plugin", {})
        self.channel = channel

    async def call[Q: Message, R: Message](
        self, operation: plugin.Operation[Q, R], request: Q, *, policy: OperationPolicy | None = None
    ) -> R:
        if "/" not in operation.id:
            invoke = self.channel.unary_unary(
                "/dokime.host.v1.Invocations/Create",
                request_serializer=invocations_pb2.InvocationsCreateRequest.SerializeToString,
                response_deserializer=invocations_pb2.InvocationsCreateResponse.FromString,
            )
            response = await invoke(
                invocations_pb2.InvocationsCreateRequest(
                    operation=operation.id, fingerprint=operation.fingerprint, payload=request.SerializeToString()
                )
            )
            return operation.response.FromString(response.payload)
        invoke = self.channel.unary_unary(
            f"/{operation.id}",
            request_serializer=operation.request.SerializeToString,
            response_deserializer=operation.response.FromString,
        )
        return await invoke(request)


class ClientTransferStory(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.identities: list[str] = []
        self.requests: list[Message] = []
        self.bytes = bytearray()
        self.fail_append = False
        self.fail_attach = False
        owner = self
        self.operation = Operation("example.echo", bool, bool, Visibility.PUBLIC)

        class Uploads(UploadsServicer):
            async def Create(
                self, request: uploads_pb2.UploadsCreateRequest, context: ServicerContext
            ) -> uploads_pb2.UploadsCreateResponse:
                owner.identities.append(request.request_id)
                owner.requests.append(request)
                return uploads_pb2.UploadsCreateResponse(upload={"id": "upload", "writer_generation": 7}, chunk_bytes=2, message_bytes=100)

            async def Append(
                self, request: uploads_pb2.UploadsAppendRequest, context: ServicerContext
            ) -> uploads_pb2.UploadsAppendResponse:
                owner.identities.append(request.request_id)
                owner.requests.append(request)
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
                owner.requests.append(request)
                if request.upload.accepted_length != len(owner.bytes):
                    await context.abort(StatusCode.ABORTED, "Wrong Upload Fence")
                return uploads_pb2.UploadsPublishResponse(upload={"length": len(owner.bytes), "sha256": sha256(owner.bytes).digest()})

            async def Delete(
                self, request: uploads_pb2.UploadsDeleteRequest, context: ServicerContext
            ) -> uploads_pb2.UploadsDeleteResponse:
                owner.identities.append(request.request_id)
                owner.requests.append(request)
                return uploads_pb2.UploadsDeleteResponse(upload=request.upload)

        class Artifacts(ArtifactsServicer):
            async def Create(
                self, request: artifacts_pb2.ArtifactsCreateRequest, context: ServicerContext
            ) -> artifacts_pb2.ArtifactsCreateResponse:
                owner.identities.append(request.request_id)
                owner.requests.append(request)
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

        types = FileDescriptorSet()
        BoolValue.DESCRIPTOR.file.CopyToProto(types.file.add())
        digest = sha256(types.SerializeToString()).digest()
        bindings = [{"record_type": "capture.capture.capture.flag", "message_name": BoolValue.DESCRIPTOR.full_name}]
        self.detail = client_types.ExecutionDetail(
            summary={"execution_id": "execution", "case_id": "case"},
            cycle_id="cycle",
            schemas={"digest": digest, "records": bindings},
            resolved_at={"sequence": 7},
        )

        class Schemas(SchemasServicer):
            async def Get(self, request: schemas_pb2.SchemasGetRequest, context: ServicerContext) -> schemas_pb2.SchemasGetResponse:
                return schemas_pb2.SchemasGetResponse(types=types, digest=digest, records=bindings, resolved_at=request.at)

        class Records(RecordsServicer):
            async def List(self, request: records_pb2.RecordsListRequest, context: ServicerContext) -> records_pb2.RecordsListResponse:
                owner.requests.append(request)
                index = 2 if request.page.page_token else 1
                row = records_pb2.CapturedRecord(
                    record_id=f"row-{index}",
                    record_type=request.record_type,
                    sequence=index,
                    scope={"execution_id": "execution"},
                    payload=BoolValue(value=index == 2).SerializeToString(),
                    timestamp={"seconds": 1, "nanos": index},
                    received_at={"seconds": 2, "nanos": index},
                )
                last = records_pb2.CapturedRecord.FromString(row.SerializeToString())
                last.sequence = 2
                last.payload = BoolValue(value=True).SerializeToString()
                return records_pb2.RecordsListResponse(
                    records=[row],
                    last=last,
                    count=2,
                    next_page_token="next" if index == 1 else "",
                    resolved_at={"positions": [{"sequence": 9}]},
                )

        class Operations(OperationsServicer):
            async def Get(
                self, request: operations_pb2.OperationsGetRequest, context: ServicerContext
            ) -> operations_pb2.OperationsGetResponse:
                return operations_pb2.OperationsGetResponse(
                    operation={"contract": {"fingerprint": owner.operation._definition().fingerprint, "visibility": "VISIBILITY_PUBLIC"}}
                )

        class Invocations(InvocationsServicer):
            async def Create(
                self, request: invocations_pb2.InvocationsCreateRequest, context: ServicerContext
            ) -> invocations_pb2.InvocationsCreateResponse:
                owner.requests.append(request)
                return invocations_pb2.InvocationsCreateResponse(payload=request.payload)

        class Evidence(EvidenceServicer):
            async def Statistics(
                self, request: evidence_pb2.EvidenceStatisticsRequest, context: ServicerContext
            ) -> evidence_pb2.EvidenceStatisticsResponse:
                owner.requests.append(request)
                return evidence_pb2.EvidenceStatisticsResponse(
                    statistics={
                        "count": 3,
                        "mean": {"exact": {"numerator": "7", "denominator": "3"}},
                        "min": {"integer": 1},
                        "max": {"unsigned_integer": 2**64 - 1},
                        "stddev": 1.25,
                    }
                )

        class Documents(DocumentsServicer):
            async def AppendParts(
                self, request: documents_pb2.DocumentsAppendPartsRequest, context: ServicerContext
            ) -> documents_pb2.DocumentsAppendPartsResponse:
                owner.requests.append(request)
                return documents_pb2.DocumentsAppendPartsResponse(receipt={"request_id": "part"})

        host = server()
        add_UploadsServicer_to_server(Uploads(), host)
        add_EvidenceServicer_to_server(Evidence(), host)
        add_DocumentsServicer_to_server(Documents(), host)
        add_OperationsServicer_to_server(Operations(), host)
        add_InvocationsServicer_to_server(Invocations(), host)
        add_RecordsServicer_to_server(Records(), host)
        add_SchemasServicer_to_server(Schemas(), host)
        add_ArtifactsServicer_to_server(Artifacts(), host)
        port = host.add_insecure_port("127.0.0.1:0")
        await host.start()
        self.addAsyncCleanup(host.stop, None)
        self.client = await connect(f"http://127.0.0.1:{port}")
        self.addAsyncCleanup(self.client.close)
        self.cycle = Cycle._bind(self.client, CycleSummary(cycle_id="cycle"))

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

    async def test_client_and_scoped_callers_share_publication_and_record_queries(self) -> None:
        class CaptureTestTemplate(TestTemplate):
            flag = record(bool)
            report = Document("Report")
            documents: ClassVar[list[Document]] = [report]

            @stream("Voltage")
            @observer()
            async def voltage(self, ctx: Context) -> float:
                return 0.0

        class CaptureSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [CaptureTestTemplate]

        class CapturePlugin(Plugin):
            version = "1.0.0"
            suites: ClassVar[list[type[Suite]]] = [CaptureSuite]

        owner = CapturePlugin()
        channel = self.client._transport._owned_channel
        assert channel is not None
        context = Context(ScopedCarrier(channel), owner, {}, owner._suites[0].templates[0])
        execution = Execution._bind(self.client, self.detail, self.cycle, Case._bind(self.client, "case", "", "", ""))
        client_artifact = await execution.artifact("trace.json", b"{}", request_id="same")
        client_requests = list(self.requests)
        self.bytes.clear()
        self.requests.clear()
        self.identities.clear()
        scoped_artifact = await context.artifact("trace.json", b"{}", request_id="same")
        self.assertEqual(self.requests, client_requests)
        self.assertEqual(client_artifact.mime, "application/json")
        self.assertEqual(scoped_artifact.mime, client_artifact.mime)
        self.assertEqual(scoped_artifact.sha256, client_artifact.sha256)
        self.assertEqual(scoped_artifact.execution, "execution")
        self.assertEqual(scoped_artifact.name, "trace.json")
        self.bytes.clear()
        self.assertEqual((await context.artifact("trace.json", b"{}", mime="test/json")).mime, "test/json")

        self.requests.clear()
        rows = await execution.records("capture.capture.capture.flag", row_type=bool).read(limit=2)
        client_requests = list(self.requests)
        self.requests.clear()
        query = context.records(CaptureTestTemplate.flag)
        self.assertEqual(await query.read(limit=2), rows)
        self.assertEqual(self.requests, client_requests)
        self.assertEqual([row.value for row in rows], [False, True])
        self.assertEqual(await query.count(), 2)
        self.assertTrue(await query.last())
        self.assertIs(context.termination, None)
        self.requests.clear()
        client_call = self.client.operation(self.operation)
        scoped_call = context.operation(self.operation)
        self.assertIs(type(client_call), BoundOperation)
        self.assertIs(type(scoped_call), BoundOperation)
        self.assertTrue(await client_call(True))
        self.assertTrue(await scoped_call(True))
        self.assertEqual(self.requests[0], self.requests[1])
        statistics = await context.statistics(CaptureTestTemplate.voltage)
        self.assertEqual(
            (statistics.count, statistics.min, statistics.max, statistics.mean, statistics.stddev), (3, 1, 2**64 - 1, Fraction(7, 3), 1.25)
        )
        self.assertIsNone(statistics.median)
        self.requests.clear()
        await context.document(CaptureTestTemplate.report, Section("Measured"), request_id="part")
        await context.document("report", Section("Measured"), request_id="part")
        self.assertEqual(self.requests[0], self.requests[1])
        assert isinstance(self.requests[0], documents_pb2.DocumentsAppendPartsRequest)
        self.assertEqual(self.requests[0].document, "capture.capture.capture.report")
        with self.assertRaises(ValueError):
            await context.document("unknown", Section("Measured"))
        with self.assertRaises(SdkError):
            await context.note("Operator Note")
        cleanup = Context(ScopedCarrier(channel), owner, {}, owner._suites[0].templates[0], termination=3)
        self.assertIs(cleanup.termination, Termination.CANCELLED)

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
