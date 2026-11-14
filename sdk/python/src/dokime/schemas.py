# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Frozen Execution Descriptors, Cached by Their Verified Digest."""

# === Imports === #

# Standard Library Imports
import asyncio
from hashlib import sha256
from typing import Any

# Third Party Imports
from google.protobuf import descriptor_pb2, descriptor_pool, message_factory
from google.protobuf.message import Message

# Local Imports
from .v1 import message

# Type Checking Imports

# === Public Exports === #

__all__ = ["FrozenSchemaCache"]

# === Constants === #

# === Types === #

# === Classes === #


class FrozenSchemaCache:
    def __init__(self) -> None:
        self._sets: dict[bytes, bytes] = {}
        self._pools: dict[bytes, descriptor_pool.DescriptorPool] = {}
        self._lock = asyncio.Lock()

    async def get(self, client: Any, detail: Any) -> descriptor_pb2.FileDescriptorSet:
        digest = detail.schemas.digest
        if len(digest) != 32 or not detail.summary.execution_id:
            raise ValueError("Read Schemas: Supply an Execution Identity and SHA-256 Digest")
        async with self._lock:
            if digest in self._sets:
                return descriptor_pb2.FileDescriptorSet.FromString(self._sets[digest])
            request = message(
                "dokime.client.v1.executions.SchemasGetRequest",
                name=detail.summary.execution_id,
                at=detail.resolved_at,
                page={"page_size": 1},
            )
            types = descriptor_pb2.FileDescriptorSet()
            tokens: set[str] = set()
            while True:
                page = await client.executions.schemas.get(request)
                if page.digest != digest or not page.HasField("resolved_at") or (request.HasField("at") and request.at != page.resolved_at):
                    raise ValueError("Read Schemas: Descriptor Digest or Read Cut Changed")
                if not page.HasField("types"):
                    raise ValueError("Read Schemas: Missing Descriptor Page")
                if page.records != detail.schemas.records or page.streams != detail.schemas.streams:
                    raise ValueError("Read Schemas: Frozen Message Bindings Changed")
                types.file.extend(page.types.file)
                if not page.next_page_token:
                    break
                if page.next_page_token in tokens:
                    raise ValueError("Read Schemas: Repeated Descriptor Page")
                tokens.add(page.next_page_token)
                request.at.CopyFrom(page.resolved_at)
                request.page.page_token = page.next_page_token
            encoded = types.SerializeToString()
            if sha256(encoded).digest() != digest:
                raise ValueError("Read Schemas: Frozen Descriptor Digest Does Not Match")
            self._sets[digest] = encoded
            return types

    async def decode(self, client: Any, detail: Any, name: str, payload: bytes) -> Message:
        types = await self.get(client, detail)
        digest = detail.schemas.digest
        if digest not in self._pools:
            pool = descriptor_pool.DescriptorPool()
            pending = {item.name: item for item in types.file}
            added: set[str] = set()
            while pending:
                ready = [item for item in pending.values() if all(dependency in added for dependency in item.dependency)]
                if not ready:
                    raise ValueError("Decode Evidence: Missing or Cyclic Descriptor Dependencies")
                for item in ready:
                    pool.Add(item)
                    added.add(item.name)
                    del pending[item.name]
            self._pools[digest] = pool
        bindings = {record.record_type: record.message_name for record in detail.schemas.records}
        bindings.update({stream.stream: stream.message_name for stream in detail.schemas.streams})
        if name not in bindings or not bindings[name]:
            raise ValueError(f"Decode Evidence: Missing Frozen Binding {name}")
        try:
            descriptor = self._pools[digest].FindMessageTypeByName(bindings[name])
        except KeyError as error:
            raise ValueError(f"Decode Evidence: Unknown Frozen Message {name}") from error
        return message_factory.GetMessageClass(descriptor).FromString(payload)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
