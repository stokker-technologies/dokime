# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Frozen Descriptor Reads Retain Cuts, Verify Bytes and Reuse Digests."""

# === Imports === #

# Standard Library Imports
import asyncio
import unittest
from hashlib import sha256
from types import SimpleNamespace
from unittest.mock import AsyncMock

# Third Party Imports
from google.protobuf import descriptor_pb2
from google.protobuf.wrappers_pb2 import BoolValue

# Local Imports
import dokime
from dokime.v1 import message

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class SchemaStory(unittest.IsolatedAsyncioTestCase):
    def setUp(self) -> None:
        self.types = descriptor_pb2.FileDescriptorSet()
        BoolValue.DESCRIPTOR.file.CopyToProto(self.types.file.add())
        self.digest = sha256(self.types.SerializeToString()).digest()
        self.detail = message(
            "dokime.client.v1.ExecutionDetail",
            summary={"execution_id": "first"},
            schemas={"digest": self.digest},
            resolved_at={"sequence": 1},
        )
        self.reply = message(
            "dokime.client.v1.executions.SchemasGetResponse", digest=self.digest, types=self.types, resolved_at=self.detail.resolved_at
        )
        self.get = AsyncMock(return_value=self.reply)
        self.client = SimpleNamespace(executions=SimpleNamespace(schemas=SimpleNamespace(get=self.get)))

    async def test_concurrent_reads_reuse_digest_and_return_independent_values(self) -> None:
        cache = dokime.FrozenSchemaCache()
        first, second = await asyncio.gather(cache.get(self.client, self.detail), cache.get(self.client, self.detail))
        first.Clear()
        self.assertEqual(second, self.types)
        self.detail.summary.execution_id = "second"
        self.assertEqual(await cache.get(self.client, self.detail), self.types)
        self.assertEqual(self.get.await_count, 1)
        assert self.get.await_args is not None
        request = self.get.await_args.args[0]
        self.assertEqual(request.name, "first")
        self.assertEqual(request.at, self.detail.resolved_at)
        decoded = await cache.decode(self.client, self.detail, BoolValue.DESCRIPTOR.full_name, BoolValue(value=True).SerializeToString())
        self.assertEqual(decoded.SerializeToString(), BoolValue(value=True).SerializeToString())

    async def test_rejected_pages_do_not_poison_the_cache(self) -> None:
        for change in ("digest", "cut", "missing", "bytes"):
            cache = dokime.FrozenSchemaCache()
            damaged = type(self.reply).FromString(self.reply.SerializeToString())
            if change == "digest":
                damaged.digest = b"x" * 32
            elif change == "cut":
                damaged.resolved_at.sequence = 2
            elif change == "missing":
                damaged.ClearField("types")
            else:
                damaged.types.file[0].package = "changed"
            self.get.return_value = damaged
            with self.assertRaises(ValueError):
                await cache.get(self.client, self.detail)
            self.get.return_value = self.reply
            self.assertEqual(await cache.get(self.client, self.detail), self.types)

    async def test_paging_assembles_dependencies_before_decode(self) -> None:
        consumer = descriptor_pb2.FileDescriptorProto(
            name="consumer.proto", package="example", syntax="proto3", dependency=[self.types.file[0].name]
        )
        row = consumer.message_type.add(name="Row")
        row.field.add(name="flag", number=1, label=1, type=11, type_name=".google.protobuf.BoolValue")
        combined = descriptor_pb2.FileDescriptorSet(file=[consumer, self.types.file[0]])
        self.detail.schemas.digest = sha256(combined.SerializeToString()).digest()
        seen = []

        async def get(request):
            seen.append(type(request).FromString(request.SerializeToString()))
            index = len(seen) - 1
            return message(
                "dokime.client.v1.executions.SchemasGetResponse",
                digest=self.detail.schemas.digest,
                types={"file": [combined.file[index]]},
                resolved_at=self.detail.resolved_at,
                next_page_token="next" if index == 0 else "",
            )

        self.get.side_effect = get
        cache = dokime.FrozenSchemaCache()
        decoded = await cache.decode(self.client, self.detail, "example.Row", b"\x0a\x02\x08\x01")
        self.assertEqual(decoded.SerializeToString(), b"\x0a\x02\x08\x01")
        self.assertEqual([request.page.page_token for request in seen], ["", "next"])
        self.assertTrue(all(request.at == self.detail.resolved_at for request in seen))
        with self.assertRaises(ValueError):
            await cache.decode(self.client, self.detail, "example.Absent", b"")

    async def test_repeated_page_token_is_refused(self) -> None:
        self.reply.next_page_token = "same"
        with self.assertRaises(ValueError):
            await dokime.FrozenSchemaCache().get(self.client, self.detail)
        self.assertEqual(self.get.await_count, 2)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
