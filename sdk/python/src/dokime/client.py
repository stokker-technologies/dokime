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
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from functools import partial
from importlib.resources import files
from pathlib import Path
from types import SimpleNamespace
from typing import Any
from urllib.parse import urlsplit

# Third Party Imports
import grpc
from google.protobuf import descriptor_pool, message_factory
from google.protobuf.descriptor import MethodDescriptor
from google.protobuf.message import Message
from harmos.client import Client as LibraryClient

# Local Imports
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


class Client:
    def __init__(self, channel: grpc.aio.Channel, *, metadata: tuple = ()):
        self.channel = channel
        self.metadata = metadata
        self.closed = False
        resource_channel = SimpleNamespace(
            unary_unary=lambda *args, **kwargs: partial(channel.unary_unary(*args, **kwargs), metadata=self.metadata),
            unary_stream=lambda *args, **kwargs: partial(channel.unary_stream(*args, **kwargs), metadata=self.metadata),
        )
        product_table = json.loads(files("dokime").joinpath("resources.json").read_text(encoding="utf-8"))
        self._library = LibraryClient(resource_channel, product_table)
        self.stubs = {name: stub(channel) for name, stub in services().items()}
        self._methods = []
        for definition in descriptors().file:
            if definition.package != "dokime.v1" and not definition.package.startswith("dokime.v1."):
                continue
            for service in definition.service:
                selected = descriptor_pool.Default().FindServiceByName(f"{definition.package}.{service.name}")
                self._methods.extend(selected.methods)
        self.plugins = Plugins(self)
        self.suites = Suites(self)
        self.templates = Templates(self)
        self.cycles = Cycles(self)
        self.executions = Executions(self)
        self.prompts = Prompts(self)
        self.records = Records(self)
        self.samples = Samples(self)
        self.reports = Reports(self)

    def __getattr__(self, name: str):
        return getattr(self._library, name)

    def call(self, request, *, method: MethodDescriptor | None = None, timeout: float | None = None):
        if self.closed:
            raise RuntimeError("Dokime client is closed")
        if method is None:
            if not isinstance(request, Message):
                raise TypeError("Streaming requests require an explicit method descriptor")
            candidates = [item for item in self._methods if item.input_type.full_name == request.DESCRIPTOR.full_name]
            if len(candidates) != 1:
                raise MissingServiceError(
                    f"Expected one service method for {request.DESCRIPTOR.full_name}; found {len(candidates)} in proto/dokime/v1"
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


class Plugins:
    def __init__(self, client: Client):
        self._client = client

    async def add(self, url: str):
        return await self._client.call(message("AddRemotePlugin", url=url))

    async def list(self):
        return await self._client.call(message("ListPlugins"))

    async def remove(self, entry: str) -> None:
        await self._client.call(message("RemovePlugin", entry=entry))


class Suites:
    def __init__(self, client: Client):
        self._client = client

    async def list(self):
        return (await self._client.call(message("CatalogListSuitesRequest"))).suites

    async def get(self, identifier: str):
        return await self._client.call(message("CatalogGetSuiteRequest", suite_id=identifier))


class Templates:
    def __init__(self, client: Client):
        self._client = client

    async def materialize(self, suite: str, template: str, parameters: Mapping[str, Sequence[Any]]):
        selection = message("RegisteredSelection", suite_id=suite, template_id=template)
        for key, values in parameters.items():
            entry = selection.parameters.add(key=key)
            entry.value.values.extend(encode(value) for value in values)
        result = await self._client.call(message("PlanningMaterializeTemplateRequest", selection=selection))
        return tuple(result.cases)


class Cycles:
    def __init__(self, client: Client):
        self._client = client

    async def start(self, cases):
        result = await self._client.call(
            message("ExecutionCreateCycleRequest", origins=[case.origin for case in cases], continue_automatically=True)
        )
        return Cycle(self._client, result.cycle_id, tuple(result.execution_ids))


@dataclass(frozen=True)
class Cycle:
    client: Client
    identifier: str
    executions: tuple[str, ...]

    async def watch(self):
        for identifier in self.executions:
            terminal = False
            async for frame in self.client._library.executions.progress.watch(
                message("dokime.v1.executions.ProgressWatchRequest", name=identifier)
            ):
                if terminal:
                    raise RuntimeError("Watch Cycle: Progress After Settlement")
                terminal = frame.finished
                yield frame
            if not terminal:
                raise RuntimeError("Watch Cycle: Missing Finished Progress")


class Executions:
    def __init__(self, client: Client):
        self._client = client

    def __getattr__(self, name):
        return getattr(self._client._library.executions, name)

    async def get(self, identifier: str):
        return await self._client.call(message("ExecutionGetExecutionRequest", execution_id=identifier))

    async def list(self):
        return (await self._client.call(message("QueryListExecutionsRequest"))).rows


class Prompts:
    def __init__(self, client: Client):
        self._client = client

    async def pending(self, execution: str):
        return (await self._client.call(message("PromptsListPendingRequest", execution_id=execution))).prompts

    async def get(self, identifier: str):
        return (await self._client.call(message("PromptsGetPromptRequest", prompt_id=identifier))).prompt

    async def respond(self, identifier: str, action: str, *, accepted: bool):
        await self._client.call(message("PromptsRespondRequest", prompt_id=identifier, action=action, accepted=accepted))


class Records:
    def __init__(self, client: Client):
        self._client = client

    async def list(self, execution: str):
        return await self._client.call(message("EvidenceListRecordsRequest", selection={"scope": {"execution_id": execution}}))


class Samples:
    def __init__(self, client: Client):
        self._client = client

    async def list(self, execution: str):
        return await self._client.call(message("TelemetryListSamplesRequest", scope={"execution_id": execution}))


class Reports:
    def __init__(self, client: Client):
        self._client = client

    async def download(self, execution: str) -> bytes:
        data = bytearray()
        completion = None
        terminal = False
        async for frame in self._client.call(message("ReportsDownloadExecutionReportRequest", execution_id=execution)):
            if terminal:
                raise RuntimeError("Download Report: Frame After Terminal Response")
            if frame.WhichOneof("value") == "response":
                terminal = True
                continue
            kind = frame.item.payload.WhichOneof("kind")
            if kind == "chunk":
                chunk = frame.item.payload.chunk
                if chunk.offset != len(data):
                    raise RuntimeError("Download Report: Noncontiguous Chunk")
                data.extend(chunk.data)
            elif kind == "complete":
                completion = frame.item.payload.complete
        if not terminal or completion is None:
            raise RuntimeError("Download Report: Missing Completion")
        if completion.size_bytes != len(data) or completion.sha256 != hashlib.sha256(data).hexdigest():
            raise RuntimeError("Download Report: Digest or Size Mismatch")
        return bytes(data)


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
