# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Explicit-Close Async Clients Over Generated Service Descriptors."""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
import json
import os
from collections.abc import AsyncIterator, Mapping, Sequence
from dataclasses import dataclass
from io import FileIO
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit
from uuid import uuid4

# Third Party Imports
import grpc

# Local Imports
from _dokime_resources import (
    CyclesResource,
    ExecutionsResource,
    FilesResource,
    PluginsResource,
    PromptsResource,
    ReportsResource,
    Resource,
    SuitesResource,
)
from google.protobuf import descriptor_pool, message_factory
from google.protobuf.descriptor import MethodDescriptor
from google.protobuf.message import Message

from .v1 import descriptors, message, services
from .values import encode

# Type Checking Imports

# === Public Exports === #

__all__ = ["Client", "Cycle", "MissingServiceError", "connect"]

# === Constants === #

# === Types === #

# === Classes === #


class MissingServiceError(RuntimeError):
    """The Authoritative Protocol Sources Do Not Declare This Client Method."""


class Client(Resource):
    files: "Files"
    plugins: "Plugins"
    suites: "Suites"
    cycles: "Cycles"
    executions: "Executions"
    prompts: "Prompts"
    reports: "Reports"

    def __init__(self, channel: grpc.aio.Channel, *, metadata: tuple[tuple[str, str], ...] = ()) -> None:
        self.channel = channel
        self.metadata = metadata
        self.closed = False
        super().__init__(self)
        self.stubs = {name: stub(channel) for name, stub in services().items()}
        self._methods: list[MethodDescriptor] = []
        for definition in descriptors().file:
            if not definition.package.startswith("dokime.client.v1"):
                continue
            for service in definition.service:
                selected = descriptor_pool.Default().FindServiceByName(f"{definition.package}.{service.name}")
                self._methods.extend(selected.methods)
        self.files = Files(self)
        self.plugins = Plugins(self)
        self.suites = Suites(self)
        self.templates = Templates(self)
        self.cycles = Cycles(self)
        self.executions = Executions(self)
        self.prompts = Prompts(self)
        self.records = Records(self)
        self.samples = Samples(self)
        self.reports = Reports(self)

    def call(self, request: Any, *, method: MethodDescriptor | None = None, timeout: float | None = None) -> Any:
        if self.closed:
            raise RuntimeError("Dokime client is closed")
        if method is None:
            if not isinstance(request, Message):
                raise TypeError("Streaming requests require an explicit method descriptor")
            candidates = [item for item in self._methods if item.input_type.full_name == request.DESCRIPTOR.full_name]
            if len(candidates) != 1:
                raise MissingServiceError(
                    f"Expected one service method for {request.DESCRIPTOR.full_name}; found {len(candidates)} in proto/dokime/client/v1"
                )
            method = candidates[0]
        if not method.client_streaming and request.DESCRIPTOR.full_name != method.input_type.full_name:
            raise TypeError("Request does not match the service method")
        path = f"/{method.containing_service.full_name}/{method.name}"
        request_type = message_factory.GetMessageClass(method.input_type)
        response_type = message_factory.GetMessageClass(method.output_type)
        cardinality = ("stream" if method.client_streaming else "unary") + "_" + ("stream" if method.server_streaming else "unary")
        operation = getattr(self.channel, cardinality)(
            path, request_serializer=request_type.SerializeToString, response_deserializer=response_type.FromString
        )
        return operation(request, metadata=self.metadata, timeout=timeout)

    async def close(self) -> None:
        if self.closed:
            return
        self.closed = True
        await self.channel.close()


class Files(FilesResource):
    async def upload(self, path: str | Path) -> Message:
        request_id = str(uuid4())
        source = await asyncio.to_thread(FileIO, path, "rb")
        try:
            metadata = await asyncio.to_thread(os.fstat, source.fileno())
            created = await self._client.call(
                message(
                    "dokime.client.v1.files.UploadsCreateRequest",
                    metadata={"expected_length": metadata.st_size},
                    request_id=f"{request_id}/create",
                )
            )
            session = created.production.session
            try:
                if not 0 < created.chunk_bytes < created.message_bytes:
                    raise ValueError("Upload File: Invalid Transfer Limits")
                digest = hashlib.sha256()
                offset = 0
                while data := await asyncio.to_thread(source.read, created.chunk_bytes):
                    reply = await self._client.call(
                        message(
                            "dokime.client.v1.files.UploadsAppendRequest",
                            session=session,
                            offset=offset,
                            data=data,
                            request_id=f"{request_id}/append/{offset}",
                        )
                    )
                    offset += len(data)
                    if reply.production.length != offset:
                        raise ValueError("Upload File: Invalid Accepted Offset")
                    digest.update(data)
                published = await self._client.call(
                    message(
                        "dokime.client.v1.files.UploadsPublishRequest",
                        session=session,
                        length=offset,
                        sha256=digest.digest(),
                        request_id=f"{request_id}/publish",
                    )
                )
                if published.file.length != offset or published.file.sha256 != digest.digest():
                    raise ValueError("Upload File: Published Length or SHA-256 Changed")
                return published.file
            except BaseException:
                await self._client.call(
                    message("dokime.client.v1.files.UploadsDeleteRequest", session=session, request_id=f"{request_id}/abandon")
                )
                raise
        finally:
            await asyncio.to_thread(source.close)


class Plugins(PluginsResource):
    def __init__(self, client: Client) -> None:
        super().__init__(client)

    async def add(self, request: Any, *, timeout: float | None = None) -> Any:
        if isinstance(request, Message):
            return await self._client.call(request, timeout=timeout)
        url = request
        return await self._client.call(message("dokime.client.v1.AddRemotePlugin", url=url))

    async def list(self, request: Any = None, *, timeout: float | None = None) -> Any:
        if request is not None:
            return await super().list(request, timeout=timeout)
        return await self._client.call(message("dokime.client.v1.ListPlugins"), timeout=timeout)

    async def remove(self, request: Any, *, timeout: float | None = None) -> Any:
        if isinstance(request, Message):
            return await self._client.call(request, timeout=timeout)
        entry = request
        await self._client.call(message("dokime.client.v1.RemovePlugin", entry=entry))


class Suites(SuitesResource):
    def __init__(self, client: Client) -> None:
        super().__init__(client)

    async def list(self, request: Any = None, *, timeout: float | None = None) -> Any:
        if request is not None:
            return await super().list(request, timeout=timeout)
        return (await super().list(message("dokime.client.v1.SuitesListRequest"), timeout=timeout)).values

    async def get(self, request: Any, *, timeout: float | None = None) -> Any:
        if not isinstance(request, str):
            return await super().get(request, timeout=timeout)
        identifier = request
        return (await self._client.call(message("dokime.client.v1.SuitesGetRequest", name=identifier))).value


class Templates:
    def __init__(self, client: Client) -> None:
        self._client = client

    async def materialize(self, suite: str, template: str, parameters: Mapping[str, Sequence[Any]]) -> tuple[Any, ...]:
        selection = message("dokime.client.v1.RegisteredSelection", suite_id=suite, template_id=template)
        for key, values in parameters.items():
            entry = selection.parameters.add(key=key)
            entry.value.values.extend(encode(value) for value in values)
        result = await self._client.call(
            message("dokime.client.v1.CasesCreateRequest", plan={"selections": [selection]}, request_id=str(uuid4()))
        )
        return tuple(result.cases)


class Cycles(CyclesResource):
    def __init__(self, client: Client) -> None:
        super().__init__(client)

    async def start(self, cases: Sequence[Any]) -> "Cycle":
        result = await self._client.call(
            message(
                "dokime.client.v1.CyclesCreateRequest",
                origins=[case.origin for case in cases],
                continue_automatically=True,
                request_id=str(uuid4()),
            )
        )
        executions = await self._client.call(
            message("dokime.client.v1.ExecutionsListRequest", filter={"cycle_ids": [result.cycle.cycle_id]})
        )
        return Cycle(self._client, result.cycle.cycle_id, tuple(item.summary.execution_id for item in executions.executions))


@dataclass(frozen=True)
class Cycle:
    client: Client
    identifier: str
    executions: tuple[str, ...]

    async def watch(self) -> AsyncIterator[Any]:
        for identifier in self.executions:
            terminal = False
            async for frame in self.client.executions.progress.watch(
                message("dokime.client.v1.executions.ProgressWatchRequest", name=identifier)
            ):
                if terminal:
                    raise RuntimeError("Watch Cycle: Progress After Settlement")
                terminal = frame.finished
                yield frame
            if not terminal:
                raise RuntimeError("Watch Cycle: Missing Finished Progress")


class Executions(ExecutionsResource):
    def __init__(self, client: Client) -> None:
        super().__init__(client)

    async def get(self, request: Any, *, timeout: float | None = None) -> Any:
        if not isinstance(request, str):
            return await super().get(request, timeout=timeout)
        identifier = request
        return (await self._client.call(message("dokime.client.v1.ExecutionsGetRequest", name=identifier))).execution

    async def list(self, request: Any = None, *, timeout: float | None = None) -> Any:
        if request is not None:
            return await super().list(request, timeout=timeout)
        return (await super().list(message("dokime.client.v1.ExecutionsListRequest"), timeout=timeout)).executions


class Prompts(PromptsResource):
    def __init__(self, client: Client) -> None:
        super().__init__(client)

    async def pending(self, execution: str) -> Sequence[Any]:
        return (await self._client.call(message("dokime.client.v1.PromptsListRequest", execution_id=execution, pending_only=True))).prompts

    async def get(self, request: Any, *, timeout: float | None = None) -> Any:
        if not isinstance(request, str):
            return await super().get(request, timeout=timeout)
        identifier = request
        return (await self._client.call(message("dokime.client.v1.PromptsGetRequest", name=identifier))).prompt

    async def respond(self, identifier: str, action: str, *, accepted: bool) -> None:
        await self._client.call(
            message(
                "dokime.client.v1.prompts.AnswersCreateRequest",
                parent=identifier,
                action=action,
                accepted=accepted,
                request_id=str(uuid4()),
            )
        )


class Records:
    def __init__(self, client: Client) -> None:
        self._client = client

    async def list(self, execution: str) -> Any:
        return await self._client.call(message("dokime.client.v1.evidence.RecordsListRequest", scope={"execution": execution}))


class Samples:
    def __init__(self, client: Client) -> None:
        self._client = client

    async def list(self, execution: str) -> Any:
        return await self._client.call(message("dokime.client.v1.evidence.SamplesListRequest", scope={"execution_id": execution}))


class Reports(ReportsResource):
    def __init__(self, client: Client) -> None:
        super().__init__(client)

    async def download(self, execution: str, *, document: str = "") -> bytes:
        report = await self._client.call(
            message(
                "dokime.client.v1.ReportsCreateRequest", subject={"execution": execution}, document_id=document, request_id=str(uuid4())
            )
        )
        data = bytearray()
        while True:
            page = await self._client.call(message("dokime.client.v1.FilesReadRequest", file=report.file, offset=len(data)))
            if page.total_length != report.file.length or page.sha256 != report.file.sha256:
                raise RuntimeError("Download Report: File Identity Changed")
            if page.total_length < len(data) + len(page.data):
                raise RuntimeError("Download Report: Invalid File Length")
            data.extend(page.data)
            if len(data) == page.total_length:
                if hashlib.sha256(data).digest() != page.sha256:
                    raise RuntimeError("Download Report: Digest Mismatch")
                return bytes(data)
            if not page.data:
                raise RuntimeError("Download Report: Empty File Page")


# === Public Functions === #


async def connect(
    target: str | Path | None = None,
    *,
    certificate_pem: str | None = None,
    access_key: str | None = None,
    expected_backend_id: str | None = None,
) -> Client:
    selected = str(target or os.environ.get("DOKIME_REMOTE", ""))
    if not selected:
        raise ValueError("Name an endpoint or .pair file, or set DOKIME_REMOTE")
    if selected.endswith(".pair"):
        pairing = await asyncio.to_thread(_pairing, Path(selected))
        certificate_pem = pairing["certificate_pem"]
        access_key = pairing["access_key"]
        expected_backend_id = pairing["backend_identifier"]
        selected = pairing["endpoints"][0]
    endpoint = urlsplit(selected)
    if (
        endpoint.scheme not in ("http", "https")
        or not endpoint.hostname
        or not endpoint.port
        or endpoint.path
        or endpoint.query
        or endpoint.fragment
        or endpoint.username
        or endpoint.password
    ):
        raise ValueError("Expected an HTTP(S) origin with an explicit port")
    if endpoint.scheme == "https":
        if not certificate_pem:
            raise ValueError("HTTPS requires a pinned server certificate")
        credentials = grpc.ssl_channel_credentials(certificate_pem.encode())
        channel = grpc.aio.secure_channel(endpoint.netloc, credentials)
    else:
        if certificate_pem:
            raise ValueError("HTTP must not carry a certificate")
        channel = grpc.aio.insecure_channel(endpoint.netloc)
    metadata = (("authorization", f"Bearer {access_key}"),) if access_key else ()
    client = Client(channel, metadata=metadata)
    try:
        if expected_backend_id:
            from harmos.v1.status_pb2 import StatusGetRequest

            response = await client.status.get(StatusGetRequest())
            if response.status.identity.host_id != expected_backend_id:
                raise ValueError("Backend identity differs from the paired backend")
        else:
            await channel.channel_ready()
        return client
    except BaseException:
        await client.close()
        raise


# === Private Functions === #


def _pairing(path: Path) -> dict:
    with path.open("rb") as source:
        content = source.read(1_048_577)
    if len(content) > 1_048_576:
        raise ValueError("Pairing file exceeds 1 MiB")
    pairing = json.loads(content)
    required = {"version", "endpoints", "backend_identifier", "label", "certificate_pem", "access_key"}
    if not isinstance(pairing, dict) or set(pairing) != required or type(pairing["version"]) is not int or pairing["version"] != 1:
        raise ValueError("Invalid version 1 pairing file")
    endpoints = pairing["endpoints"]
    if not isinstance(endpoints, list) or not 1 <= len(endpoints) <= 32 or any(not isinstance(endpoint, str) for endpoint in endpoints):
        raise ValueError("Pairing requires between 1 and 32 endpoint strings")
    if len(set(endpoints)) != len(endpoints):
        raise ValueError("Pairing endpoints must be distinct")
    identity = pairing["backend_identifier"]
    label = pairing["label"]
    if (
        not isinstance(identity, str)
        or not identity
        or len(identity.encode()) > 128
        or any(character.isspace() or ord(character) < 32 for character in identity)
    ):
        raise ValueError("Invalid pairing backend identifier")
    if not isinstance(label, str) or not label.strip() or len(label.encode()) > 256 or any(ord(character) < 32 for character in label):
        raise ValueError("Invalid pairing label")
    key = pairing["access_key"]
    if key is not None and (
        not isinstance(key, str) or not 32 <= len(key) <= 4096 or not all(33 <= ord(character) <= 126 for character in key)
    ):
        raise ValueError("Invalid pairing access key")
    return pairing


# === Entry Point === #
