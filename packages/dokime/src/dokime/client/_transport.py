# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Authenticated asyncio gRPC transport, negotiation and verified downloads."""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
import ipaddress
import json
import math
import os
import re
from collections.abc import AsyncIterator, Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit

# Third Party Imports
import grpc
from google.protobuf.json_format import ParseDict
from google.protobuf.message import DecodeError, Message

from ..dynamic import from_contract

# Local Imports
from ..operation import Operation
from . import _contracts, _wire
from .errors import BackendChanged, ClientClosed, ProtocolError, RpcError
from .models import Backend

# Type Checking Imports

# === Public Exports === #

# === Constants === #
_MAX_MESSAGE_BYTES = 16 * 1024 * 1024
_DISCOVERY_BYTES = 1024 * 1024

# === Types === #

# === Classes === #


class Transport:
    def __init__(
        self,
        endpoint: str,
        token: str | None,
        ca_certificate: str | Path | None,
        timeout: float,
        *,
        expected_backend_id: str | None = None,
        ca_certificate_pem: str | None = None,
    ) -> None:
        self._target, self._remote = _target(endpoint)
        if token is not None and (
            not 32 <= len(token) <= 4096 or not all(33 <= ord(c) <= 126 for c in token)
        ):
            raise ValueError(
                "bearer token must contain 32–4096 visible ASCII characters"
            )
        if self._remote and not token:
            raise ValueError("HTTPS endpoints require a bearer token")
        if expected_backend_id is not None and (
            not expected_backend_id
            or len(expected_backend_id.encode("utf-8")) > 256
            or any(
                ord(char) < 32 or 127 <= ord(char) <= 159
                for char in expected_backend_id
            )
        ):
            raise ValueError("invalid expected backend identity")
        self._expected_backend_id = expected_backend_id
        if ca_certificate is not None and ca_certificate_pem is not None:
            raise ValueError("CA certificate file and PEM text are mutually exclusive")
        if (
            ca_certificate is not None or ca_certificate_pem is not None
        ) and not self._remote:
            raise ValueError("CA certificates require HTTPS")
        if not math.isfinite(timeout) or timeout <= 0:
            raise ValueError("RPC timeout must be positive and finite")
        self._ca = (
            Path(ca_certificate).read_bytes()
            if ca_certificate is not None
            else ca_certificate_pem.encode("utf-8")
            if ca_certificate_pem is not None
            else None
        )
        self._metadata = (("authorization", f"Bearer {token}"),) if token else ()
        self.timeout = timeout
        self._channel: grpc.aio.Channel | None = None
        self._lock = asyncio.Lock()
        self._calls: set[Any] = set()
        self.closed = False
        self.backend: Backend | None = None
        self._request_limit = _MAX_MESSAGE_BYTES
        self._response_limit = _DISCOVERY_BYTES

    async def connect(self) -> Backend:
        async with self._lock:
            if self.closed:
                raise ClientClosed("DokimeClient is closed")
            if self.backend is not None:
                return self.backend
            options = (
                ("grpc.max_send_message_length", _MAX_MESSAGE_BYTES),
                ("grpc.max_receive_message_length", _MAX_MESSAGE_BYTES),
                ("grpc.keepalive_time_ms", 10000),
                ("grpc.keepalive_timeout_ms", 5000),
                ("grpc.keepalive_permit_without_calls", 1),
                ("grpc.enable_retries", 0),
            )
            self._channel = (
                grpc.aio.secure_channel(
                    self._target,
                    grpc.ssl_channel_credentials(self._ca),
                    options=options,
                )
                if self._remote
                else grpc.aio.insecure_channel(self._target, options=options)
            )
            try:
                identity = await self._invoke_operation(
                    _contracts.operation("Backend", "GetIdentity"),
                    _contracts.request("Backend", "GetIdentity"),
                )
                health = await self._invoke_operation(
                    _contracts.operation("Backend", "GetHealth"),
                    _contracts.request("Backend", "GetHealth"),
                )
                capabilities = await self._invoke_operation(
                    _contracts.operation("Backend", "GetCapabilities"),
                    _contracts.request("Backend", "GetCapabilities"),
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
                if (
                    self._expected_backend_id is not None
                    and identity.backend_id != self._expected_backend_id
                ):
                    raise BackendChanged(
                        "backend does not match the selected profile identity"
                    )
                if any(
                    item.backend_id != identity.backend_id
                    or item.epoch != identity.epoch
                    for item in (health, capabilities)
                ):
                    raise ProtocolError("backend identity changed during negotiation")
                if _wire.enum_name(health.readiness) not in ("ready", "degraded"):
                    raise RpcError("UNAVAILABLE", "backend is not ready")
                required = {
                    f"/dokime.v1.Backend/{method}"
                    for method in ("GetIdentity", "GetHealth", "GetCapabilities")
                }
                if not required.issubset(
                    capabilities.methods
                ) or not capabilities.HasField("limits"):
                    raise ProtocolError("backend discovery capabilities are incomplete")
                if (
                    not capabilities.limits.max_request_bytes
                    or not capabilities.limits.max_response_bytes
                ):
                    raise ProtocolError("backend message limits are invalid")
                self._request_limit = min(
                    capabilities.limits.max_request_bytes, _MAX_MESSAGE_BYTES
                )
                self._response_limit = min(
                    capabilities.limits.max_response_bytes, _MAX_MESSAGE_BYTES
                )
                self.backend = Backend(
                    identity.backend_id,
                    identity.epoch,
                    identity.api.version,
                    identity.display_name,
                    frozenset(capabilities.methods),
                    product_version=identity.product_version,
                    source_revision=identity.source_revision
                    if identity.HasField("source_revision")
                    else None,
                    sdk_revision=identity.sdk_revision
                    if identity.HasField("sdk_revision")
                    else None,
                    descriptor_sha256=bytes(identity.api.descriptor_sha256),
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
        if (
            self.backend is not None
            and operation.id.startswith("dokime.")
            and operation.grpc_path not in self.backend.methods
        ):
            raise RpcError("UNIMPLEMENTED", "method is not available to this caller")

    def _request(self, request: Message, expected: type[Message]) -> bytes:
        if not isinstance(request, expected):
            raise TypeError("request type differs from the selected operation contract")
        payload = request.SerializeToString()
        if len(payload) > self._request_limit:
            raise RpcError(
                "RESOURCE_EXHAUSTED", "request exceeds negotiated message limit"
            )
        return payload

    def _response(self, payload: bytes, expected: type[Message]) -> Message:
        if len(payload) > self._response_limit:
            raise RpcError(
                "RESOURCE_EXHAUSTED", "response exceeds negotiated message limit"
            )
        try:
            return expected.FromString(payload)
        except DecodeError as error:
            raise ProtocolError("invalid canonical protobuf response") from error

    async def _invoke_operation(
        self, operation: Operation, request: Message
    ) -> Message:
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
        reader = _Stream(
            self, _contracts.operation(service, method), request, discover=True
        )
        await reader._open()
        return reader

    async def operations(self) -> tuple["DiscoveredOperation", ...]:
        await self.connect()
        import dokime

        discovery = dokime.HOST_OPERATIONS
        entries = []
        keys = set()
        cursors = set()
        after = None
        used = 0
        async with asyncio.timeout(self.timeout):
            while True:
                page = await self._invoke_operation(
                    discovery,
                    discovery.request(
                        after=after,
                        rows=64,
                        bytes=min(self._response_limit, 4 * 1024 * 1024),
                    ),
                )
                used += len(page.catalog)
                if used > 16 * 1024 * 1024:
                    raise ProtocolError("operation catalog exceeds byte limit")
                try:
                    batch = json.loads(page.catalog, object_pairs_hook=_object)
                except (ValueError, UnicodeError) as error:
                    raise ProtocolError("invalid caller operation catalog") from error
                if (
                    not isinstance(batch, list)
                    or len(batch) > 64
                    or len(entries) + len(batch) > 4096
                ):
                    raise ProtocolError("operation catalog exceeds row limit")
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
                    try:
                        descriptor = from_contract(row["contract"])
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
                if (
                    not batch
                    or not after
                    or after in cursors
                    or len(after.encode("utf-8")) > 4 * 1024 * 1024
                ):
                    raise ProtocolError("operation catalog does not progress")
                cursors.add(after)
                used += len(after.encode("utf-8"))

    async def show(
        self, identifier: str, *, owner: str | None = None
    ) -> "DiscoveredOperation":
        matches = [
            entry
            for entry in await self.operations()
            if entry.operation.id == identifier
            and (owner is None or entry.owner == owner)
        ]
        if len(matches) != 1:
            raise ValueError("operation must identify exactly one authorized provider")
        return matches[0]

    async def _select(
        self, operation: Operation | str, *, owner: str | None = None
    ) -> Operation:
        selected = await self.show(
            operation if isinstance(operation, str) else operation.id, owner=owner
        )
        if not selected.grpc_available:
            raise RpcError("UNIMPLEMENTED", "operation has no available gRPC binding")
        if isinstance(operation, Operation):
            if json.loads(operation.contract) != json.loads(
                selected.operation.contract
            ):
                raise ProtocolError(
                    "installed operation contract differs from the live provider"
                )
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
        prepared = (
            ParseDict(dict(request), selected.request(), ignore_unknown_fields=False)
            if isinstance(request, Mapping)
            else request
        )
        return await self._invoke_operation(selected, prepared)

    async def upload(
        self, operation: Operation, request: Message, items: AsyncIterator[Message]
    ) -> Message:
        selected = await self._select(operation)
        if selected.mode != "client_stream":
            raise ValueError("upload requires a client-streaming operation")
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

    def exchange(
        self, operation: Operation, request: Message, items: AsyncIterator[Message]
    ) -> "_Stream":
        if operation.mode != "bidi":
            raise ValueError("exchange requires a bidirectional operation")
        return _Stream(self, operation, request, items=items, discover=True)

    async def _inputs(
        self, operation: Operation, request: Message, items: AsyncIterator[Message]
    ) -> AsyncIterator[bytes]:
        self._request(request, operation.request)
        yield self._request(
            operation.rpc_request(request=request), operation.rpc_request
        )
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
    ) -> tuple[int, str, Message]:
        service, method = (
            ("History", "DownloadCheckpointArtifact")
            if expected is not None
            else ("Reports", "DownloadCycleReport")
        )
        reader = await self.stream(service, method, request)
        partial = destination.with_name(destination.name + ".part")
        owned = False
        try:
            with partial.open("xb") as output:
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
                        if expected is not None:
                            if (
                                metadata.artifact_id != request.artifact_id
                                or (metadata.size_bytes, metadata.sha256) != expected
                            ):
                                raise ProtocolError(
                                    "artifact identity or inventory changed"
                                )
                        elif (
                            metadata.test_cycle_id != request.cycle_id
                            or metadata.identifier != request.identifier
                        ):
                            raise ProtocolError("report identity differs from request")
                    elif kind == "chunk" and metadata is not None:
                        chunk = payload
                        if chunk.offset != offset or not chunk.data:
                            raise ProtocolError("chunks are not contiguous")
                        output.write(chunk.data)
                        offset += len(chunk.data)
                        digest.update(chunk.data)
                    elif kind == "complete" and metadata is not None:
                        complete = payload
                        if (complete.size_bytes, complete.sha256) != (
                            offset,
                            digest.hexdigest(),
                        ) or (
                            expected is not None
                            and (offset, digest.hexdigest()) != expected
                        ):
                            raise ProtocolError("download size or SHA-256 differs")
                    else:
                        raise ProtocolError("expected one header before file data")
                if complete is None or metadata is None:
                    raise ProtocolError("download ended without metadata or completion")
                output.flush()
                os.fsync(output.fileno())
            os.replace(partial, destination)
            owned = False
            return offset, digest.hexdigest(), metadata
        finally:
            await reader.close()
            if owned:
                partial.unlink(missing_ok=True)

    async def verify(self, expected: Backend, *, live: bool = False) -> None:
        identity = await self.unary(
            "Backend", "GetIdentity", _contracts.request("Backend", "GetIdentity")
        )
        if identity.backend_id != expected.id or (
            live and identity.epoch != expected.epoch
        ):
            raise BackendChanged(
                "backend identity or live epoch changed; recover published evidence through History"
            )

    async def close(self) -> None:
        async with self._lock:
            self.closed = True
            for call in tuple(self._calls):
                call.cancel()
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
                    timeout=_stream_timeout(operation),
                )
            else:
                if self._items is None:
                    raise ValueError("bidirectional operation requires input items")
                self._transport._request(self._request, operation.request)
                self._call = channel.stream_stream(operation.grpc_path)(
                    self._transport._inputs(operation, self._request, self._items),
                    metadata=self._transport._metadata,
                    timeout=_stream_timeout(operation),
                )
            self._transport._calls.add(self._call)

    def __aiter__(self):
        return self

    async def __anext__(self) -> Message:
        await self._open()
        call = self._call
        if call is None:
            raise ProtocolError("operation stream did not open")
        try:
            payload = await call.read()
            if payload is grpc.aio.EOF:
                self._transport._calls.discard(self._call)
                if self._completion is None:
                    raise ProtocolError(
                        "operation stream ended without its final response"
                    )
                raise StopAsyncIteration
            if self._completion is not None:
                raise ProtocolError("operation output followed its final response")
            if not isinstance(payload, bytes):
                raise ProtocolError("operation stream returned invalid frame bytes")
            envelope = self._transport._response(payload, self._operation.rpc_response)
            kind = envelope.WhichOneof(envelope.DESCRIPTOR.oneofs[0].name)
            if kind == "response":
                self._completion = envelope.response
                return await self.__anext__()
            if kind != "item":
                raise ProtocolError("operation stream frame has no payload")
            return envelope.item
        except asyncio.CancelledError:
            await self.close()
            if self._transport.closed:
                raise ClientClosed("DokimeClient is closed") from None
            raise
        except grpc.aio.AioRpcError as error:
            await self.close()
            if self._transport.closed:
                raise ClientClosed("DokimeClient is closed") from None
            raise _error(error) from error
        except ProtocolError:
            await self.close()
            raise

    async def response(self) -> Message:
        async for _ in self:
            pass
        if self._completion is None:
            raise ProtocolError("operation has no final response")
        return self._completion

    async def close(self) -> None:
        self._closed = True
        if self._call is not None:
            self._call.cancel()
            self._transport._calls.discard(self._call)


# === Public Functions === #

# === Private Functions === #


def _stream_timeout(operation: Operation) -> float:
    try:
        timeout = json.loads(operation.contract)["operation"]["policy"]["timeout"]
        seconds, nanos = timeout["secs"], timeout["nanos"]
    except (ValueError, KeyError, TypeError) as error:
        raise ProtocolError("operation has no declared stream deadline") from error
    if (
        type(seconds) is not int
        or type(nanos) is not int
        or seconds < 0
        or not 0 <= nanos < 1_000_000_000
    ):
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
    if (
        re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", version)
        is None
    ):
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
        remote = url.scheme == "https"
        if not remote:
            host = "127.0.0.1" if host == "localhost" else host
            if not ipaddress.ip_address(host).is_loopback:
                raise ValueError
        authority = f"[{host}]" if ":" in host else host
        return (
            f"{authority}:{port if port is not None else (443 if remote else 80)}",
            remote,
        )
    except ValueError as error:
        raise ValueError(
            "use an HTTPS origin or an HTTP literal loopback origin without user information, path, query or fragment"
        ) from error


# === Entry Point === #


def _object(items: list[tuple[str, Any]]) -> dict[str, Any]:
    result = {}
    for key, value in items:
        if key in result:
            raise ValueError("duplicate catalog field")
        result[key] = value
    return result
