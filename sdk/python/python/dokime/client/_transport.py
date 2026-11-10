# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Async Python Conversion Over The Rust Client."""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
import json
import math
import os
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit

# Third Party Imports
from google.protobuf.json_format import ParseDict
from google.protobuf.message import DecodeError, Message

# Local Imports
from .. import _native
from ..dynamic import from_contract
from ..operation import Operation
from . import _contracts
from .errors import BackendChanged, ClientClosed, ProtocolError, RpcError
from .models import Backend

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class DiscoveredOperation:
    owner: str
    generation: int
    operation: Operation
    grants: tuple[Any, ...]
    conditional_access: bool
    grpc_available: bool


class Transport:
    def __init__(
        self,
        endpoint,
        access_key,
        timeout,
        *,
        connect_timeout_ms,
        endpoints=(),
        expected_backend_id=None,
        certificate_pem=None,
    ):
        self._endpoint = _native.validate_target(endpoint, certificate_pem)
        self._primary_endpoint = self._endpoint
        self._endpoints = tuple(_native.validate_target(item, certificate_pem) for item in endpoints)
        self._access_key = access_key
        self._expected_backend_id = expected_backend_id
        self._ca = certificate_pem
        self.timeout = timeout
        if not math.isfinite(timeout) or timeout <= 0:
            raise ValueError("rpc_timeout must be finite and positive")
        self._settings = {
            "connect_timeout_ms": connect_timeout_ms,
            "request_timeout_ms": int(timeout * 1000),
        }
        self._lock = asyncio.Lock()
        self._channel = None
        self._connecting = None
        self.closed = False
        self.backend = None
        self.query_settings = None
        self._request_limit = 16777216
        self._response_limit = 16777216
        self._verified = {}
        self._operations_cache = {}
        self._streams = set()

    async def connect(self) -> Backend:
        async with self._lock:
            if self.closed:
                raise ClientClosed("DokimeClient is closed")
            if self.backend is not None:
                return self.backend
            settings = {**self._settings, "request_timeout_ms": int(self.timeout * 1000)}
            self._connecting = asyncio.ensure_future(
                _native.Client.connect(
                    self._primary_endpoint,
                    list(self._endpoints),
                    json.dumps(settings),
                    self._access_key,
                    self._expected_backend_id,
                    self._ca,
                )
            )
            try:
                native = await self._connecting
            except asyncio.CancelledError:
                if self.closed:
                    raise ClientClosed("DokimeClient is closed") from None
                raise
            finally:
                self._connecting = None
            if self.closed:
                await native.close()
                raise ClientClosed("DokimeClient is closed")
            data = json.loads(native.backend)
            limits = data["limits"]
            backend = Backend(
                data["backend_id"],
                data["epoch"],
                data["api_version"],
                data["display_name"],
                frozenset(data["methods"]),
                product_version=data["product_version"],
                source_revision=data["source_revision"],
                channel=data["channel"],
                descriptor_sha256=bytes.fromhex(data["descriptor_sha256"]),
                max_evidence_bytes=limits["max_evidence_bytes"],
            )
            self._channel = native
            self._endpoint = native.endpoint.rstrip("/")
            self.backend = backend
            self._request_limit = min(limits["max_request_bytes"], self._request_limit)
            self._response_limit = min(limits["max_response_bytes"], self._response_limit)
            return backend

    def _available(self, operation):
        if self.closed or self._channel is None:
            raise ClientClosed("DokimeClient is closed")

    def _request(self, request, expected):
        if not isinstance(request, expected):
            raise TypeError("request type differs from the selected operation contract")
        payload = request.SerializeToString()
        if len(payload) > self._request_limit:
            raise RpcError("RESOURCE_EXHAUSTED", "request exceeds negotiated message limit")
        return payload

    def _response(self, payload, expected):
        if len(payload) > self._response_limit:
            raise RpcError("RESOURCE_EXHAUSTED", "response exceeds negotiated message limit")
        try:
            return expected.FromString(payload)
        except DecodeError as error:
            raise ProtocolError("invalid canonical protobuf response") from error

    async def _invoke_operation(self, operation, request):
        self._available(operation)
        if operation.mode != "unary":
            raise ValueError("this operation requires its streaming interface")
        response = await self._channel.call(operation.grpc_path, self._request(request, operation.request), self.timeout)
        return self._response(response, operation.response)

    async def unary(self, service, method, request):
        await self.connect()
        selected = await self._select(_contracts.operation(service, method))
        return await self._invoke_operation(selected, request)

    async def stream(self, service, method, request):
        await self.connect()
        reader = _Stream(self, _contracts.operation(service, method), request)
        await reader._open()
        return reader

    async def operations(self):
        await self.connect()
        entries = json.loads(await self._channel.operations())
        entries = [item for item in entries if item["contract"].get("grpc") is not None or item["grpc_available"]]
        reflected = {}

        def describe(item):
            key = (item["generation"], json.dumps(item["contract"], separators=(",", ":")))
            operation = self._operations_cache.get(key)
            if operation is None:
                operation = from_contract(item["contract"], generation=item["generation"])
            reflected[key] = operation
            return DiscoveredOperation(
                item["owner"],
                item["generation"],
                operation,
                tuple(item["grants"]),
                item["conditional_access"],
                item["grpc_available"],
            )

        try:
            result = tuple(describe(item) for item in entries)
        except (ValueError, TypeError, KeyError) as error:
            raise ProtocolError("invalid operation contract") from error
        self._operations_cache = reflected
        return result

    async def show(self, identifier, *, owner=None):
        matches = [item for item in await self.operations() if item.operation.id == identifier and (owner is None or item.owner == owner)]
        if len(matches) != 1:
            raise ValueError("operation must identify exactly one authorized provider")
        return matches[0]

    async def _select(self, operation, *, owner=None):
        await self.connect()
        if isinstance(operation, Operation) and owner is None and self._verified.get(operation.id) == operation.contract:
            return operation
        selected = await self.show(operation if isinstance(operation, str) else operation.id, owner=owner)
        if not selected.grpc_available:
            raise RpcError("UNIMPLEMENTED", "operation has no available gRPC binding")
        if isinstance(operation, Operation):
            if json.loads(operation.contract) != json.loads(selected.operation.contract):
                raise ProtocolError("installed operation contract differs from the live provider")
            if owner is None:
                self._verified[operation.id] = operation.contract
            return operation
        return selected.operation

    async def call(self, operation, request, *, owner=None):
        selected = await self._select(operation, owner=owner)
        prepared = ParseDict(dict(request), selected.request(), ignore_unknown_fields=False) if isinstance(request, Mapping) else request
        return await self._invoke_operation(selected, prepared)

    async def upload(self, operation, request, items):
        if operation.mode != "client_stream":
            raise ValueError("upload requires a client-streaming operation")
        reader = _Stream(self, operation, request, items=items)
        try:
            return await reader.response()
        finally:
            await reader.close()

    def download(self, operation, request):
        if operation.mode != "server_stream":
            raise ValueError("download requires a server-streaming operation")
        return _Stream(self, operation, request)

    def exchange(self, operation, request, items):
        if operation.mode != "bidi":
            raise ValueError("exchange requires a bidirectional operation")
        return _Stream(self, operation, request, items=items)

    async def verify(self, expected: Backend, *, live: bool = False) -> None:
        await self.connect()
        if self.backend.id != expected.id or (live and self.backend.epoch != expected.epoch):
            raise BackendChanged("backend identity changed")
        await self._channel.verify(live)

    async def close(self):
        self.closed = True
        if self._connecting is not None:
            self._connecting.cancel()
        await asyncio.gather(*(reader.close() for reader in tuple(self._streams)))
        if self._channel is not None:
            await self._channel.close()

    async def download_file(
        self,
        request: Message,
        destination: Path,
        expected: tuple[int, str] | None = None,
        *,
        endpoint: tuple[str, str] | None = None,
        validate: Callable[[Message], tuple[int, str]] | None = None,
        maximum_bytes: int | None = None,
        overwrite: bool = True,
    ) -> tuple[int, str, Message]:
        service, method = endpoint or (
            ("History", "DownloadCheckpointArtifact") if expected is not None else ("Reports", "DownloadCycleReport")
        )
        reader = await self.stream(service, method, request)
        partial = destination.with_name(destination.name + ".part")
        owned = False
        try:
            with partial.open("xb", buffering=1_048_576) as output:
                owned = True
                offset = 0
                digest = hashlib.sha256()
                metadata = None
                complete = None
                async for frame in reader:
                    if complete is not None:
                        raise ProtocolError("frames followed completion")
                    if not frame.HasField("payload"):
                        raise ProtocolError("download frame has no payload")
                    kind = _contracts.variant(frame.payload)
                    payload = getattr(frame.payload, kind).field_0
                    kind = kind.lower()
                    if kind in ("metadata", "report") and metadata is None:
                        metadata = payload
                        if validate is not None:
                            expected = validate(metadata)
                        elif expected is not None:
                            if metadata.artifact_id != request.artifact_id or (metadata.size_bytes, metadata.sha256) != expected:
                                raise ProtocolError("artifact identity or inventory changed")
                        elif metadata.test_cycle_id != request.cycle_id or metadata.identifier != request.identifier:
                            raise ProtocolError("report identity differs from request")
                        if expected is not None and maximum_bytes is not None and expected[0] > maximum_bytes:
                            raise ProtocolError("download exceeds its byte limit")
                    elif kind == "chunk" and metadata is not None:
                        chunk = payload
                        if chunk.offset != offset or not chunk.data:
                            raise ProtocolError("chunks are not contiguous")
                        end = offset + len(chunk.data)
                        if maximum_bytes is not None and end > maximum_bytes or expected is not None and end > expected[0]:
                            raise ProtocolError("download exceeds its declared size or byte limit")
                        if output.write(chunk.data) != len(chunk.data):
                            raise OSError("download did not write the complete chunk")
                        offset = end
                        digest.update(chunk.data)
                    elif kind == "complete" and metadata is not None:
                        complete = payload
                        if (complete.size_bytes, complete.sha256) != (
                            offset,
                            digest.hexdigest(),
                        ) or (expected is not None and (offset, digest.hexdigest()) != expected):
                            raise ProtocolError("download size or SHA-256 differs")
                    else:
                        raise ProtocolError("expected one header before file data")
                if complete is None or metadata is None:
                    raise ProtocolError("download ended without metadata or completion")
                output.flush()
                os.fsync(output.fileno())
            if overwrite:
                os.replace(partial, destination)
            else:
                os.link(partial, destination)
                partial.unlink()
            owned = False
            return offset, digest.hexdigest(), metadata
        finally:
            try:
                await reader.close()
            finally:
                if owned:
                    partial.unlink(missing_ok=True)


class _Stream:
    def __init__(self, transport, operation, request, *, items=None):
        self._transport = transport
        self._operation = operation
        self._request = request
        self._items = items
        self._native = None
        self._completion = None
        self._closed = False
        self._open_lock = asyncio.Lock()
        self._sender = None
        self._pending = None
        self._exhausted = False

    async def _open(self):
        async with self._open_lock:
            if self._closed or self._transport.closed:
                raise ClientClosed("operation stream is closed")
            if self._native is not None:
                return
            self._operation = await self._transport._select(self._operation)
            operation = self._operation
            self._transport._available(operation)
            request = operation.rpc_request(request=self._request) if operation.input is not None else self._request
            self._native = await self._transport._channel.stream(
                operation.grpc_path, self._transport._request(request, operation.rpc_request), _stream_timeout(operation)
            )
            self._transport._streams.add(self)
            if self._items is None:
                self._native.finish()
            else:
                self._sender = asyncio.create_task(self._send())

    async def _send(self):
        try:
            assert self._items is not None
            async for item in self._items:
                payload = self._transport._request(self._operation.rpc_request(item=item), self._operation.rpc_request)
                await self._native.send(payload)
        except BaseException:
            self._native.cancel()
            raise
        finally:
            self._native.finish()

    def __aiter__(self):
        return self

    async def __anext__(self):
        if self._exhausted:
            raise StopAsyncIteration
        await self._open()
        try:
            while True:
                async with asyncio.timeout(_stream_timeout(self._operation)):
                    self._pending = asyncio.ensure_future(self._native.next())
                    payload = await self._pending
                if self._closed or self._transport.closed:
                    raise ClientClosed("operation stream is closed")
                if self._sender is not None and self._sender.done():
                    await self._sender
                if payload is None:
                    if self._completion is None:
                        raise ProtocolError("operation stream ended without its final response")
                    self._exhausted = True
                    await self.close()
                    raise StopAsyncIteration
                if self._completion is not None:
                    raise ProtocolError("operation output followed its final response")
                envelope = self._transport._response(payload, self._operation.rpc_response)
                if self._operation.output is None:
                    self._completion = envelope
                    continue
                kind = envelope.WhichOneof(envelope.DESCRIPTOR.oneofs[0].name)
                if kind == "response":
                    self._completion = envelope.response
                    continue
                if kind != "item":
                    raise ProtocolError("operation stream frame has no payload")
                return envelope.item
        except TimeoutError as error:
            await self.close()
            raise RpcError("DEADLINE_EXCEEDED", "stream idle deadline exceeded") from error
        except BaseException:
            await self.close()
            raise
        finally:
            self._pending = None

    async def response(self):
        async for _ in self:
            pass
        if self._completion is None:
            raise ProtocolError("operation has no final response")
        return self._completion

    async def close(self):
        self._closed = True
        self._transport._streams.discard(self)
        if self._native is not None:
            self._native.cancel()
        if self._pending is not None:
            self._pending.cancel()
            await asyncio.gather(self._pending, return_exceptions=True)
        if self._sender is not None:
            self._sender.cancel()
            await asyncio.gather(self._sender, return_exceptions=True)
        if self._native is not None:
            await self._native.close()


# === Public Functions === #

# === Private Functions === #


def _stream_timeout(operation):
    timeout = json.loads(operation.contract)["operation"]["policy"]["timeout"]
    return timeout["secs"] + timeout["nanos"] / 1_000_000_000


def _target(endpoint):
    normalized = _native.validate_target(endpoint)
    parsed = urlsplit(normalized)
    return parsed.netloc, parsed.scheme == "https"


def _cursor(backend: Backend, message: Message, field: str = "observed_cursor") -> None:
    if not message.HasField(field) or not getattr(message, field).epoch:
        raise ProtocolError("reply has no observed epoch")
    if getattr(message, field).epoch != backend.epoch:
        raise BackendChanged("reply belongs to another backend epoch")


# === Entry Point === #
