# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Frozen Client Evidence Queries"""

# === Imports === #

# Standard Library Imports
import unittest
from hashlib import sha256

from google.protobuf import descriptor_pool, message_factory

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorProto, FileDescriptorSet
from google.protobuf.wrappers_pb2 import BoolValue, UInt64Value
from grpc.aio import ServicerContext, server

# Local Imports
from dokime_sdk._connection import connect
from dokime_sdk._errors import SdkError
from dokime_sdk._wire.host.v1 import executions_pb2
from dokime_sdk._wire.host.v1.evidence import records_pb2, samples_pb2
from dokime_sdk._wire.host.v1.evidence.records_pb2_grpc import RecordsServicer, add_RecordsServicer_to_server
from dokime_sdk._wire.host.v1.evidence.samples_pb2_grpc import SamplesServicer, add_SamplesServicer_to_server
from dokime_sdk._wire.host.v1.executions import schemas_pb2
from dokime_sdk._wire.host.v1.executions.schemas_pb2_grpc import SchemasServicer, add_SchemasServicer_to_server
from dokime_sdk._wire.host.v1.executions_pb2_grpc import ExecutionsServicer, add_ExecutionsServicer_to_server

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ClientEvidenceStory(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.trace: list[str] = []
        self.tokens: list[tuple[str, bytes]] = []
        self.changed_cut = False
        owner = self
        types = FileDescriptorSet()
        BoolValue.DESCRIPTOR.file.CopyToProto(types.file.add())
        digest = sha256(types.SerializeToString()).digest()
        binding = executions_pb2.FrozenSchemas(
            digest=digest,
            records=[{"record_type": "flag", "message_name": BoolValue.DESCRIPTOR.full_name}],
            streams=[{"stream": "counter", "message_name": UInt64Value.DESCRIPTOR.full_name}],
        )
        detail = executions_pb2.ExecutionDetail(
            summary={"execution_id": "execution", "case_id": "case"}, cycle_id="cycle", schemas=binding, resolved_at={"sequence": 7}
        )

        class Executions(ExecutionsServicer):
            async def Get(
                self, request: executions_pb2.ExecutionsGetRequest, context: ServicerContext
            ) -> executions_pb2.ExecutionsGetResponse:
                owner.trace.append("Get")
                return executions_pb2.ExecutionsGetResponse(execution=detail)

        class Schemas(SchemasServicer):
            async def Get(self, request: schemas_pb2.SchemasGetRequest, context: ServicerContext) -> schemas_pb2.SchemasGetResponse:
                owner.trace.append("Schemas")
                return schemas_pb2.SchemasGetResponse(
                    types=types, digest=digest, records=binding.records, streams=binding.streams, resolved_at=request.at
                )

        class Records(RecordsServicer):
            async def List(self, request: records_pb2.RecordsListRequest, context: ServicerContext) -> records_pb2.RecordsListResponse:
                owner.trace.append("Records")
                owner.tokens.append((request.page.page_token, request.at.SerializeToString()))
                index = 2 if request.page.page_token else 1
                record = records_pb2.CapturedRecord(
                    record_id=f"record-{index}",
                    sequence=index,
                    record_type="flag",
                    schema_version=1,
                    timestamp={"seconds": 1, "nanos": index},
                    received_at={"seconds": 2, "nanos": index},
                    source="source",
                    scope={"execution_id": "execution"},
                    payload=BoolValue(value=index == 2).SerializeToString(),
                )
                last = records_pb2.CapturedRecord.FromString(record.SerializeToString())
                last.payload = BoolValue(value=True).SerializeToString()
                return records_pb2.RecordsListResponse(
                    records=[record],
                    count=2,
                    last=last,
                    resolved_at={"positions": [{"sequence": 11 if owner.changed_cut and index == 2 else 10}]},
                    next_page_token="next" if index == 1 else "",
                )

        class Samples(SamplesServicer):
            async def List(self, request: samples_pb2.SamplesListRequest, context: ServicerContext) -> samples_pb2.SamplesListResponse:
                owner.trace.append("Samples")
                owner.tokens.append((request.page.page_token, request.at.SerializeToString()))
                index = 2 if request.page.page_token else 1
                return samples_pb2.SamplesListResponse(
                    samples=[
                        {
                            "raw": {
                                "sample_id": f"sample-{index}",
                                "stream": "counter",
                                "sequence": index,
                                "timestamp": {"seconds": 1, "nanos": index},
                                "received_at": {"seconds": 2, "nanos": index},
                                "scope": {"execution_id": "execution"},
                                "values": {"values": {"value": {"unsigned_integer": 2**64 - 1}}},
                                "payload": UInt64Value(value=2**64 - 1).SerializeToString(),
                            }
                        }
                    ],
                    resolved_at={"positions": [{"sequence": 10}]},
                    next_page_token="next" if index == 1 else "",
                )

        host = server()
        add_ExecutionsServicer_to_server(Executions(), host)
        add_SchemasServicer_to_server(Schemas(), host)
        add_RecordsServicer_to_server(Records(), host)
        add_SamplesServicer_to_server(Samples(), host)
        port = host.add_insecure_port("127.0.0.1:0")
        await host.start()
        self.addAsyncCleanup(host.stop, None)
        self.client = await connect(f"http://127.0.0.1:{port}")
        self.addAsyncCleanup(self.client.close)
        self.execution = await self.client.executions.get("execution")

    async def test_records_decode_frozen_schema_and_retain_the_first_evidence_cut(self) -> None:
        rows = await self.execution.records("flag", row_type=bool).read(limit=2)
        self.assertEqual([row.value for row in rows], [False, True])
        self.assertEqual([row.timestamp_ns for row in rows], [1_000_000_001, 1_000_000_002])
        self.assertEqual(self.trace, ["Get", "Schemas", "Records", "Records"])
        self.assertEqual(self.tokens[0][1], b"")
        self.assertNotEqual(self.tokens[1][1], b"")
        query = self.execution.records("flag", row_type=bool)
        self.assertEqual(await query.count(), 2)
        self.assertTrue(await query.last())
        dynamic = await self.execution.records("flag").read()
        self.assertEqual(dynamic[0].value.DESCRIPTOR.full_name, BoolValue.DESCRIPTOR.full_name)
        self.changed_cut = True
        with self.assertRaises(SdkError):
            await self.execution.records("flag").read(limit=3)
        with self.assertRaises(SdkError):
            await self.execution.records("flag", row_type=int).read()

    async def test_declared_row_type_must_match_the_frozen_fields_not_only_the_name(self) -> None:
        definition = FileDescriptorProto(name="foreign.proto", package="google.protobuf", syntax="proto3")
        row = definition.message_type.add(name="BoolValue")
        row.field.add(name="value", number=1, type=9, label=1)
        pool = descriptor_pool.DescriptorPool()
        pool.Add(definition)
        foreign = message_factory.GetMessageClass(pool.FindMessageTypeByName("google.protobuf.BoolValue"))
        with self.assertRaises(SdkError):
            await self.execution.records("flag", row_type=foreign).read()

    async def test_samples_preserve_nanoseconds_unsigned_values_and_refuse_truncation(self) -> None:
        query = self.execution.samples("counter", fields=["value"])
        rows = await query.collect()
        self.assertEqual([row.values["value"] for row in rows], [2**64 - 1, 2**64 - 1])
        self.assertEqual(rows[0].timestamp_ns, 1_000_000_001)
        self.assertEqual(rows[0].value.DESCRIPTOR.full_name, UInt64Value.DESCRIPTOR.full_name)
        with self.assertRaises(ValueError):
            await query.collect(max_rows=1)
        table = await query.to_pandas()
        frame = await query.to_polars()
        self.assertEqual(table["value"].tolist(), [2**64 - 1, 2**64 - 1])
        self.assertEqual(str(table["value"].dtype), "UInt64")
        self.assertEqual(frame["value"].to_list(), [2**64 - 1, 2**64 - 1])
        self.assertEqual(frame["timestamp"].cast(int).to_list(), [1_000_000_001, 1_000_000_002])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
