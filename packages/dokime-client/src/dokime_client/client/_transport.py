# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Authenticated asyncio gRPC transport, negotiation and verified downloads."""

import asyncio
import hashlib
import json
import math
import os
import re
import ssl
from collections.abc import AsyncIterator, Callable, Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit

import grpc
from google.protobuf.json_format import ParseDict
from google.protobuf.message import DecodeError, Message

from ..dynamic import from_contract
from ..operation import Operation
from . import _contracts, _wire
from .errors import BackendChanged, ClientClosed, ProtocolError, RpcError
from .models import Backend
from .secrets import _validate_key

CA_BYTES = 1048576
CATALOG_BYTES = 16777216
CATALOG_PAGE_BYTES = 4194304
CATALOG_PAGE_ROWS = 64
CATALOG_ROWS = 4096
DISCOVERY_BYTES = 1048576
KEEPALIVE_INTERVAL_MS = 10000
KEEPALIVE_TIMEOUT_MS = 5000
MAX_MESSAGE_BYTES = 16777216
MAXIMUM_TIMEOUT_MS = 86400000
PROFILE_ID_BYTES = 256
PROJECTION_BYTES = 4194304
PROJECTION_DEFINITIONS = 65536
PROJECTION_WORK = 1048576


class Transport:
    def __init__(
        self,
        endpoint: str,
        token: str | None,
        timeout: float,
        *,
        connect_timeout_ms: int,
        expected_backend_id: str | None = None,
        ca_certificate_pem: str | None = None,
    ) -> None:
        self._target, self._tls = _target(endpoint)
        _validate_key(token)
        if expected_backend_id is not None and (
            not expected_backend_id
            or len(expected_backend_id.encode("utf-8")) > PROFILE_ID_BYTES
            or any(ord(char) < 32 or 127 <= ord(char) <= 159 for char in expected_backend_id)
        ):
            raise ValueError("invalid expected backend identity")
        self._expected_backend_id = expected_backend_id
        if ca_certificate_pem is not None and not self._tls:
            raise ValueError("CA certificates require HTTPS")
        try:
            valid_timeout = type(timeout) in (int, float) and math.isfinite(timeout) and 0 < timeout <= MAXIMUM_TIMEOUT_MS / 1000
        except OverflowError:
            valid_timeout = False
        if not valid_timeout:
            raise ValueError("RPC timeout must be positive and finite")
        self._ca = ca_certificate_pem.encode("utf-8") if ca_certificate_pem is not None else None
        if self._ca is not None and len(self._ca) > CA_BYTES:
            raise ValueError("CA certificate exceeds the configured byte limit")
        if self._ca is not None:
            try:
                ssl.create_default_context(cadata=self._ca.decode("utf-8"))
            except (ssl.SSLError, UnicodeError, ValueError):
                raise ValueError("invalid CA certificate") from None
        if type(connect_timeout_ms) is not int or not 0 < connect_timeout_ms <= MAXIMUM_TIMEOUT_MS:
            raise ValueError("connection timeout must be a positive supported duration")
        self.connect_timeout = connect_timeout_ms / 1000
        self._metadata = (("authorization", f"Bearer {token}"),) if token else ()
        self.timeout = timeout
        self._channel: grpc.aio.Channel | None = None
        self._lock = asyncio.Lock()
        self._calls: set[Any] = set()
        self._readers: set[asyncio.Task] = set()
        self.closed = False
        self.backend: Backend | None = None
        self._request_limit = MAX_MESSAGE_BYTES
        self._response_limit = DISCOVERY_BYTES
        # Installed contracts verified against this backend's live catalog,
        # once each: the backend is fixed for the transport's lifetime.
        self._verified: dict[str, Operation] = {}

    async def connect(self) -> Backend:
        async with self._lock:
            if self.closed:
                raise ClientClosed("DokimeClient is closed")
            if self.backend is not None:
                return self.backend
            options = (
                ("grpc.max_send_message_length", MAX_MESSAGE_BYTES),
                ("grpc.max_receive_message_length", MAX_MESSAGE_BYTES),
                ("grpc.keepalive_time_ms", KEEPALIVE_INTERVAL_MS),
                ("grpc.keepalive_timeout_ms", KEEPALIVE_TIMEOUT_MS),
                ("grpc.keepalive_permit_without_calls", 1),
                ("grpc.enable_retries", 0),
            )
            self._channel = (
                grpc.aio.secure_channel(
                    self._target,
                    grpc.ssl_channel_credentials(self._ca),
                    options=options,
                )
                if self._tls
                else grpc.aio.insecure_channel(self._target, options=options)
            )
            try:
                await asyncio.wait_for(self._channel.channel_ready(), timeout=self.connect_timeout)
                identity = await self._invoke_operation(
                    _contracts.operation("Backend", "GetIdentity"),
                    _contracts.request("Backend", "GetIdentity"),
                )
                if self._expected_backend_id is not None and identity.backend_id != self._expected_backend_id:
                    raise BackendChanged("backend does not match the selected profile identity")
                health = await self._invoke_operation(
                    _contracts.operation("Backend", "GetHealth"),
                    _contracts.request("Backend", "GetHealth"),
                )
                availability = await self._invoke_operation(
                    _contracts.operation("Backend", "GetAvailability"),
                    _contracts.request("Backend", "GetAvailability"),
                )
                if (
                    not identity.HasField("api")
                    or identity.api.package != "dokime.v1"
                    or _release(identity.api.version) != ("0", "6")
                    or not identity.backend_id
                    or not identity.epoch
                    or len(identity.api.descriptor_sha256) != 32
                ):
                    raise ProtocolError("backend API or identity is incompatible")
                if any(item.backend_id != identity.backend_id or item.epoch != identity.epoch for item in (health, availability)):
                    raise ProtocolError("backend identity changed during negotiation")
                if _wire.enum_name(health.readiness) not in ("ready", "degraded"):
                    raise RpcError("UNAVAILABLE", "backend is not ready")
                required = {f"/dokime.v1.Backend/{method}" for method in ("GetIdentity", "GetHealth", "GetAvailability")}
                if not required.issubset(availability.methods) or not availability.HasField("limits"):
                    raise ProtocolError("backend availability is incomplete")
                if not availability.limits.max_request_bytes or not availability.limits.max_response_bytes:
                    raise ProtocolError("backend message limits are invalid")
                self._request_limit = min(availability.limits.max_request_bytes, MAX_MESSAGE_BYTES)
                self._response_limit = min(availability.limits.max_response_bytes, MAX_MESSAGE_BYTES)
                self.backend = Backend(
                    identity.backend_id,
                    identity.epoch,
                    identity.api.version,
                    identity.display_name,
                    frozenset(availability.methods),
                    product_version=identity.product_version,
                    source_revision=identity.source_revision if identity.HasField("source_revision") else None,
                    sdk_revision=identity.sdk_revision if identity.HasField("sdk_revision") else None,
                    descriptor_sha256=bytes(identity.api.descriptor_sha256),
                    max_evidence_bytes=availability.limits.max_evidence_bytes
                    if availability.limits.HasField("max_evidence_bytes")
                    else None,
                )
                return self.backend
            except BaseException:
                await self._channel.close()
                self._channel = None
                raise

    def _available(self, operation: Operation) -> None:
        if self.closed or self._channel is None:
            raise ClientClosed("DokimeClient is closed")
        if operation.codec != "protobuf":
            raise ProtocolError("unsupported operation codec")
        if self.backend is not None and operation.id.startswith("dokime.") and operation.grpc_path not in self.backend.methods:
            raise RpcError("UNIMPLEMENTED", "method is not available to this caller")

    def _request(self, request: Message, expected: type[Message]) -> bytes:
        if not isinstance(request, expected):
            raise TypeError("request type differs from the selected operation contract")
        payload = request.SerializeToString()
        if len(payload) > self._request_limit:
            raise RpcError("RESOURCE_EXHAUSTED", "request exceeds negotiated message limit")
        return payload

    def _response(self, payload: bytes, expected: type[Message]) -> Message:
        if len(payload) > self._response_limit:
            raise RpcError("RESOURCE_EXHAUSTED", "response exceeds negotiated message limit")
        try:
            return expected.FromString(payload)
        except DecodeError as error:
            raise ProtocolError("invalid canonical protobuf response") from error

    async def _verify_expected(self) -> None:
        if self.backend is None or self._expected_backend_id is None:
            return
        identity = await self._invoke_operation(
            _contracts.operation("Backend", "GetIdentity"),
            _contracts.request("Backend", "GetIdentity"),
        )
        if identity.backend_id != self._expected_backend_id:
            raise BackendChanged("backend does not match the pairing identity")

    async def _invoke_operation(self, operation: Operation, request: Message) -> Message:
        if operation.grpc_path != "/dokime.v1.Backend/GetIdentity":
            await self._verify_expected()
        self._available(operation)
        if operation.mode != "unary":
            raise ValueError("this operation requires its streaming interface")
        assert self._channel is not None
        call = self._channel.unary_unary(operation.grpc_path)(
            self._request(request, operation.request),
            metadata=self._metadata,
            timeout=self.timeout,
        )
        self._calls.add(call)
        try:
            return self._response(await call, operation.response)
        except asyncio.CancelledError:
            call.cancel()
            if self.closed:
                raise ClientClosed("DokimeClient is closed") from None
            raise
        except grpc.aio.AioRpcError as error:
            if self.closed:
                raise ClientClosed("DokimeClient is closed") from None
            raise _error(error) from error
        finally:
            self._calls.discard(call)

    async def _invoke(self, service: str, method: str, request: Message) -> Message:
        selected = await self._select(_contracts.operation(service, method))
        return await self._invoke_operation(selected, request)

    async def unary(self, service: str, method: str, request: Message) -> Message:
        await self.connect()
        return await self._invoke(service, method, request)

    async def stream(self, service: str, method: str, request: Message) -> "_Stream":
        await self.connect()
        reader = _Stream(self, _contracts.operation(service, method), request, discover=True)
        await reader._open()
        return reader

    async def operations(self) -> tuple["DiscoveredOperation", ...]:
        await self.connect()
        import dokime_client

        discovery = dokime_client.HOST_CATALOG
        entries = []
        keys = set()
        cursors = set()
        after = None
        used = 0
        seen_rows = 0
        async with asyncio.timeout(self.timeout):
            while True:
                page = await self._invoke_operation(
                    discovery,
                    discovery.request(
                        after=after,
                        rows=CATALOG_PAGE_ROWS,
                        bytes=min(self._response_limit, CATALOG_PAGE_BYTES),
                    ),
                )
                used += len(page.entries)
                if used > CATALOG_BYTES:
                    raise ProtocolError("operation catalog exceeds byte limit")
                try:
                    batch = json.loads(page.entries, object_pairs_hook=_object)
                except (ValueError, UnicodeError) as error:
                    raise ProtocolError("invalid caller operation catalog") from error
                if not isinstance(batch, list) or len(batch) > CATALOG_PAGE_ROWS or seen_rows + len(batch) > CATALOG_ROWS:
                    raise ProtocolError("operation catalog exceeds row limit")
                seen_rows += len(batch)
                for row in batch:
                    if (
                        not isinstance(row, dict)
                        or not isinstance(row.get("owner"), str)
                        or not row["owner"]
                        or type(row.get("generation")) is not int
                        or not 0 <= row["generation"] < 2**64
                        or type(row.get("grpc_available")) is not bool
                        or type(row.get("conditional_access")) is not bool
                        or not isinstance(row.get("grants"), list)
                    ):
                        raise ProtocolError("invalid provider operation metadata")
                    # Native-Only Contracts Have No Protobuf Projection Or Remote Binding.
                    contract = row.get("contract")
                    native_only = isinstance(contract, dict) and "grpc" not in contract and not row["grpc_available"]
                    if native_only:
                        continue
                    try:
                        descriptor = from_contract(
                            row["contract"],
                            max_bytes=PROJECTION_BYTES,
                            max_work=PROJECTION_WORK,
                            max_definitions=PROJECTION_DEFINITIONS,
                        )
                    except (ValueError, TypeError, KeyError) as error:
                        raise ProtocolError("invalid operation contract") from error
                    key = descriptor.id, row["owner"]
                    if key in keys:
                        raise ProtocolError("operation catalog repeats a provider")
                    keys.add(key)
                    entries.append(
                        DiscoveredOperation(
                            row["owner"],
                            row["generation"],
                            descriptor,
                            tuple(row["grants"]),
                            row["conditional_access"],
                            row["grpc_available"],
                        )
                    )
                if not page.HasField("next"):
                    return tuple(entries)
                after = page.next
                if not batch or not after or after in cursors or len(after.encode("utf-8")) > CATALOG_PAGE_BYTES:
                    raise ProtocolError("operation catalog does not progress")
                cursors.add(after)
                used += len(after.encode("utf-8"))

    async def show(self, identifier: str, *, owner: str | None = None) -> "DiscoveredOperation":
        matches = [
            entry for entry in await self.operations() if entry.operation.id == identifier and (owner is None or entry.owner == owner)
        ]
        if len(matches) != 1:
            raise ValueError("operation must identify exactly one authorized provider")
        return matches[0]

    async def _select(self, operation: Operation | str, *, owner: str | None = None) -> Operation:
        if isinstance(operation, Operation) and owner is None:
            verified = self._verified.get(operation.id)
            if verified is not None and verified.contract == operation.contract:
                return operation
        selected = await self.show(operation if isinstance(operation, str) else operation.id, owner=owner)
        if not selected.grpc_available:
            raise RpcError("UNIMPLEMENTED", "operation has no available gRPC binding")
        if isinstance(operation, Operation):
            if json.loads(operation.contract) != json.loads(selected.operation.contract):
                raise ProtocolError("installed operation contract differs from the live provider")
            if owner is None:
                self._verified[operation.id] = operation
            return operation
        return selected.operation

    async def call(
        self,
        operation: Operation | str,
        request: Message | Mapping[str, Any],
        *,
        owner: str | None = None,
    ) -> Message:
        selected = await self._select(operation, owner=owner)
        prepared = ParseDict(dict(request), selected.request(), ignore_unknown_fields=False) if isinstance(request, Mapping) else request
        return await self._invoke_operation(selected, prepared)

    async def upload(self, operation: Operation, request: Message, items: AsyncIterator[Message]) -> Message:
        selected = await self._select(operation)
        if selected.mode != "client_stream":
            raise ValueError("upload requires a client-streaming operation")
        await self._verify_expected()
        self._available(selected)
        self._request(request, selected.request)
        assert self._channel is not None
        call = self._channel.stream_unary(selected.grpc_path)(
            self._inputs(selected, request, items),
            metadata=self._metadata,
            timeout=self.timeout,
        )
        self._calls.add(call)
        try:
            return self._response(await call, selected.response)
        except asyncio.CancelledError:
            call.cancel()
            if self.closed:
                raise ClientClosed("DokimeClient is closed") from None
            raise
        except grpc.aio.AioRpcError as error:
            if self.closed:
                raise ClientClosed("DokimeClient is closed") from None
            raise _error(error) from error
        finally:
            self._calls.discard(call)

    def download(self, operation: Operation, request: Message) -> "_Stream":
        if operation.mode != "server_stream":
            raise ValueError("download requires a server-streaming operation")
        return _Stream(self, operation, request, discover=True)

    def exchange(self, operation: Operation, request: Message, items: AsyncIterator[Message]) -> "_Stream":
        if operation.mode != "bidi":
            raise ValueError("exchange requires a bidirectional operation")
        return _Stream(self, operation, request, items=items, discover=True)

    async def _inputs(self, operation: Operation, request: Message, items: AsyncIterator[Message]) -> AsyncIterator[bytes]:
        self._request(request, operation.request)
        yield self._request(operation.rpc_request(request=request), operation.rpc_request)
        async for item in items:
            if operation.input is None:
                raise ValueError("operation has no input item contract")
            self._request(item, operation.input)
            yield self._request(operation.rpc_request(item=item), operation.rpc_request)

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

    async def verify(self, expected: Backend, *, live: bool = False) -> None:
        identity = await self.unary("Backend", "GetIdentity", _contracts.request("Backend", "GetIdentity"))
        if identity.backend_id != expected.id or (live and identity.epoch != expected.epoch):
            raise BackendChanged("backend identity or live epoch changed; recover published evidence through History")

    async def close(self) -> None:
        async with self._lock:
            self.closed = True
            for call in tuple(self._calls):
                call.cancel()
            readers = tuple(self._readers)
            for reader in readers:
                reader.cancel()
            await asyncio.gather(*readers, return_exceptions=True)
            self._readers.clear()
            if self._channel is not None:
                await self._channel.close()
                self._channel = None
            self._calls.clear()
            self._metadata = ()


@dataclass(frozen=True)
class DiscoveredOperation:
    owner: str
    generation: int
    operation: Operation
    grants: tuple[Any, ...]
    conditional_access: bool
    grpc_available: bool


class _Stream:
    def __init__(
        self,
        transport: Transport,
        operation: Operation,
        request: Message,
        *,
        items: AsyncIterator[Message] | None = None,
        discover: bool = False,
    ) -> None:
        self._transport = transport
        self._operation = operation
        self._request = request
        self._items = items
        self._discover = discover
        self._call = None
        self._completion = None
        self._closed = False
        self._open_lock = asyncio.Lock()
        self._report = operation.grpc_path in (
            "/dokime.v1.Reports/DownloadCycleReport",
            "/dokime.v1.Reports/DownloadExecutionReport",
        )
        self._prefetch = asyncio.Queue(maxsize=3) if self._report else None
        self._frame_limit = 2_097_152
        self._credit = asyncio.Semaphore(1)
        self._reader_task = None
        self._terminal = None

    async def _open(self) -> None:
        async with self._open_lock:
            if self._closed:
                raise ClientClosed("operation stream is closed")
            if self._call is not None:
                return
            await self._transport.connect()
            if self._discover:
                self._operation = await self._transport._select(self._operation)
            operation = self._operation
            await self._transport._verify_expected()
            if operation.mode not in ("server_stream", "bidi"):
                raise ValueError("operation has no output stream")
            self._transport._available(operation)
            channel = self._transport._channel
            assert channel is not None
            if operation.mode == "server_stream":
                payload = self._transport._request(self._request, operation.request)
                self._call = channel.unary_stream(operation.grpc_path)(
                    payload,
                    metadata=self._transport._metadata,
                    timeout=300.0 if self._report else _stream_timeout(operation),
                )
            else:
                if self._items is None:
                    raise ValueError("bidirectional operation requires input items")
                self._transport._request(self._request, operation.request)
                self._call = channel.stream_stream(operation.grpc_path)(
                    self._transport._inputs(operation, self._request, self._items),
                    metadata=self._transport._metadata,
                    timeout=300.0 if self._report else _stream_timeout(operation),
                )
            self._transport._calls.add(self._call)

    def __aiter__(self):
        return self

    async def __anext__(self) -> Message:
        if not self._report:
            return await self._next()
        if self._closed or self._transport.closed:
            raise ClientClosed("operation stream is closed")
        if self._terminal is not None:
            raise self._terminal
        if self._reader_task is None:
            self._reader_task = asyncio.create_task(self._read_ahead())
            self._transport._readers.add(self._reader_task)
            self._reader_task.add_done_callback(self._reader_finished)
        try:
            item = await self._prefetch.get()
            self._credit.release()
        except asyncio.CancelledError:
            await self.close()
            raise
        if isinstance(item, BaseException):
            self._terminal = item
            raise item
        return item

    async def _read_ahead(self) -> None:
        try:
            await self._open()
            self._frame_limit = min(self._transport._response_limit, 2_097_152)
            # Reserve Before Reading; Count The Consumer In The Four-Frame/4-MiB Bound.
            capacity = max(1, min(3, 4_194_304 // self._frame_limit - 1))
            self._credit = asyncio.Semaphore(capacity)
            while True:
                await self._credit.acquire()
                item = await self._next()
                await self._prefetch.put(item)
        except asyncio.CancelledError:
            if self._transport.closed:
                while not self._prefetch.empty():
                    self._prefetch.get_nowait()
                self._prefetch.put_nowait(ClientClosed("DokimeClient is closed"))
            raise
        except Exception as error:
            await self._prefetch.put(error)
            raise

    def _reader_finished(self, task: asyncio.Task) -> None:
        self._transport._readers.discard(task)
        if not task.cancelled():
            # The Queue Transfers The Failure To The Consumer; Observe Task Completion Too.
            task.exception()

    async def _next(self) -> Message:
        await self._open()
        call = self._call
        if call is None:
            raise ProtocolError("operation stream did not open")
        try:
            payload = await call.read()
            if payload is grpc.aio.EOF:
                self._transport._calls.discard(self._call)
                if self._completion is None:
                    raise ProtocolError("operation stream ended without its final response")
                raise StopAsyncIteration
            if self._completion is not None:
                raise ProtocolError("operation output followed its final response")
            if not isinstance(payload, bytes):
                raise ProtocolError("operation stream returned invalid frame bytes")
            if self._report and len(payload) > self._frame_limit:
                raise ProtocolError("report frame exceeds its transfer limit")
            envelope = self._transport._response(payload, self._operation.rpc_response)
            kind = envelope.WhichOneof(envelope.DESCRIPTOR.oneofs[0].name)
            if kind == "response":
                self._completion = envelope.response
                return await self._next()
            if kind != "item":
                raise ProtocolError("operation stream frame has no payload")
            return envelope.item
        except asyncio.CancelledError:
            self._cancel_call()
            if self._transport.closed:
                raise ClientClosed("DokimeClient is closed") from None
            raise
        except grpc.aio.AioRpcError as error:
            self._cancel_call()
            if self._transport.closed:
                raise ClientClosed("DokimeClient is closed") from None
            raise _error(error) from error
        except ProtocolError:
            self._cancel_call()
            raise

    async def response(self) -> Message:
        async for _ in self:
            pass
        if self._completion is None:
            raise ProtocolError("operation has no final response")
        return self._completion

    async def close(self) -> None:
        self._closed = True
        if self._reader_task is not None and self._reader_task is not asyncio.current_task():
            self._reader_task.cancel()
            await asyncio.gather(self._reader_task, return_exceptions=True)
        if self._prefetch is not None:
            while not self._prefetch.empty():
                self._prefetch.get_nowait()
            self._prefetch.put_nowait(ClientClosed("operation stream is closed"))
        self._cancel_call()

    def _cancel_call(self) -> None:
        if self._call is not None:
            self._call.cancel()
            self._transport._calls.discard(self._call)


def _stream_timeout(operation: Operation) -> float:
    try:
        timeout = json.loads(operation.contract)["operation"]["policy"]["timeout"]
        seconds, nanos = timeout["secs"], timeout["nanos"]
    except (ValueError, KeyError, TypeError) as error:
        raise ProtocolError("operation has no declared stream deadline") from error
    if type(seconds) is not int or type(nanos) is not int or seconds < 0 or not 0 <= nanos < 1_000_000_000:
        raise ProtocolError("operation stream deadline is invalid")
    result = seconds + nanos / 1_000_000_000
    if result <= 0 or not math.isfinite(result):
        raise ProtocolError("operation stream deadline is invalid")
    return result


def _cursor(backend: Backend, message: Message, field: str = "observed_cursor") -> None:
    if not message.HasField(field) or not getattr(message, field).epoch:
        raise ProtocolError("reply has no observed epoch")
    if getattr(message, field).epoch != backend.epoch:
        raise BackendChanged("reply belongs to another backend epoch")


def _error(error: grpc.aio.AioRpcError) -> Exception:
    return RpcError(
        error.code().name,
        error.details() or error.code().name,
        initial_metadata=tuple(error.initial_metadata() or ()),
        trailing_metadata=tuple(error.trailing_metadata() or ()),
        debug_error_string=error.debug_error_string(),
    )


def _release(version: str) -> tuple[str, str] | None:
    if re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", version) is None:
        return None
    major, minor, _ = version.split(".")
    return major, minor


def _target(endpoint: str) -> tuple[str, bool]:
    try:
        url = urlsplit(endpoint)
        host = url.hostname
        port = url.port
        if (
            url.scheme not in ("http", "https")
            or not host
            or url.username is not None
            or url.password is not None
            or url.path not in ("", "/")
            or url.query
            or url.fragment
        ):
            raise ValueError
        tls = url.scheme == "https"
        if not tls and host == "localhost":
            host = "127.0.0.1"
        authority = f"[{host}]" if ":" in host else host
        return (
            f"{authority}:{port if port is not None else (443 if tls else 80)}",
            tls,
        )
    except ValueError as error:
        raise ValueError("use an HTTP(S) origin without user information, path, query or fragment") from error


def _object(items: list[tuple[str, Any]]) -> dict[str, Any]:
    result = {}
    for key, value in items:
        if key in result:
            raise ValueError("duplicate catalog field")
        result[key] = value
    return result
