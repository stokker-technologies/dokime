# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Frozen Execution Descriptors, Cached by Their Verified Digest."""

# === Imports === #

# Standard Library Imports
import asyncio
from hashlib import sha256
from typing import Protocol

# Third Party Imports
from google.protobuf import descriptor_pb2, descriptor_pool, message_factory
from google.protobuf.message import DecodeError, Message
from harmos import PageRequest, Position

# Local Imports
from ._errors import call, invalid
from ._generated.client_types import ExecutionDetail, SchemasGetResponse

# Type Checking Imports

# === Public Exports === #


# === Constants === #

# === Types === #

# === Classes === #


class SchemasReader(Protocol):
    async def get(self, name: str, *, at: Position | None = None, page: PageRequest | None = None) -> SchemasGetResponse: ...


class ExecutionsReader(Protocol):
    @property
    def schemas(self) -> SchemasReader: ...


class SchemaClient(Protocol):
    @property
    def executions(self) -> ExecutionsReader: ...


class FrozenSchemaCache:
    def __init__(self) -> None:
        self._sets: dict[bytes, bytes] = {}
        self._pools: dict[bytes, descriptor_pool.DescriptorPool] = {}
        self._lock = asyncio.Lock()

    async def get(self, client: SchemaClient, detail: ExecutionDetail) -> descriptor_pb2.FileDescriptorSet:
        digest = detail.schemas.digest
        if len(digest) != 32 or not detail.summary.execution_id:
            raise invalid("executions.schemas.get", "Read Schemas: Supply an Execution Identity and SHA-256 Digest")
        async with self._lock:
            if digest in self._sets:
                return descriptor_pb2.FileDescriptorSet.FromString(self._sets[digest])
            at = Position.FromString(detail.resolved_at.SerializeToString())
            cursor = PageRequest(page_size=1)
            types = descriptor_pb2.FileDescriptorSet()
            tokens: set[str] = set()
            while True:
                page = await call("executions.schemas.get", client.executions.schemas.get(detail.summary.execution_id, at=at, page=cursor))
                if page.digest != digest or not page.HasField("resolved_at") or at != page.resolved_at:
                    raise invalid("executions.schemas.get", "Read Schemas: Descriptor Digest or Read Cut Changed")
                if not page.HasField("types"):
                    raise invalid("executions.schemas.get", "Read Schemas: Missing Descriptor Page")
                if page.records != detail.schemas.records or page.streams != detail.schemas.streams:
                    raise invalid("executions.schemas.get", "Read Schemas: Frozen Message Bindings Changed")
                types.file.extend(page.types.file)
                if not page.next_page_token:
                    break
                if page.next_page_token in tokens:
                    raise invalid("executions.schemas.get", "Read Schemas: Repeated Descriptor Page")
                tokens.add(page.next_page_token)
                at.CopyFrom(page.resolved_at)
                cursor.page_token = page.next_page_token
            encoded = types.SerializeToString()
            if sha256(encoded).digest() != digest:
                raise invalid("executions.schemas.get", "Read Schemas: Frozen Descriptor Digest Does Not Match")
            self._sets[digest] = encoded
            return types

    async def decode(self, client: SchemaClient, detail: ExecutionDetail, name: str, payload: bytes) -> Message:
        types = await self.get(client, detail)
        digest = detail.schemas.digest
        if digest not in self._pools:
            pool = descriptor_pool.DescriptorPool()
            pending = {item.name: item for item in types.file}
            added: set[str] = set()
            while pending:
                ready = [item for item in pending.values() if all(dependency in added for dependency in item.dependency)]
                if not ready:
                    raise invalid("executions.schemas.get", "Decode Evidence: Missing or Cyclic Descriptor Dependencies")
                for item in ready:
                    try:
                        pool.Add(item)
                    except (TypeError, ValueError) as error:
                        raise invalid("executions.schemas.get", "The Frozen Descriptor Set Is Invalid") from error
                    added.add(item.name)
                    del pending[item.name]
            self._pools[digest] = pool
        bindings = {record.record_type: record.message_name for record in detail.schemas.records}
        bindings.update({stream.stream: stream.message_name for stream in detail.schemas.streams})
        if name not in bindings or not bindings[name]:
            raise invalid("executions.schemas.get", f"Decode Evidence: Missing Frozen Binding {name}")
        try:
            descriptor = self._pools[digest].FindMessageTypeByName(bindings[name])
        except KeyError as error:
            raise invalid("executions.schemas.get", f"Decode Evidence: Unknown Frozen Message {name}") from error
        try:
            return message_factory.GetMessageClass(descriptor).FromString(payload)
        except DecodeError as error:
            raise invalid("evidence.decode", "The Retained Payload Does Not Match Its Frozen Message") from error


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
