# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Own Async Connections to the Generated Client Tree."""

# === Imports === #

# Standard Library Imports
import json
from asyncio import wait_for
from collections.abc import AsyncIterator, Awaitable, Callable, Sequence
from dataclasses import replace
from math import isfinite
from pathlib import Path
from typing import TypedDict
from urllib.parse import SplitResult, urlsplit

# Third Party Imports
from grpc import ssl_channel_credentials
from grpc.aio import (
    AioRpcError,
    Channel,
    ClientCallDetails,
    Metadata,
    UnaryStreamCall,
    UnaryStreamClientInterceptor,
    UnaryUnaryCall,
    UnaryUnaryClientInterceptor,
    insecure_channel,
    secure_channel,
)

import dokime_sdk._generated.client

# Local Imports
from ._client import Client
from ._configuration import load, resolve
from ._errors import Failure, SdkError, chain, transport_error

# Type Checking Imports

# === Public Exports === #

# === Constants === #

_CONNECT_TIMEOUT_S = 30.0

# === Types === #

# === Classes === #


class Pairing(TypedDict):
    certificate_pem: str | None
    access_key: str | None
    backend_identifier: str
    endpoints: list[str]
    label: str
    version: int


class _UnaryMetadata(UnaryUnaryClientInterceptor):
    def __init__(self, metadata: Sequence[tuple[str, str]]) -> None:
        self._metadata = tuple(metadata)

    async def intercept_unary_unary[Q, R](
        self,
        continuation: Callable[[ClientCallDetails, Q], Awaitable[UnaryUnaryCall[Q, R]]],
        client_call_details: ClientCallDetails,
        request: Q,
    ) -> UnaryUnaryCall[Q, R]:
        details = ClientCallDetails(
            client_call_details.method,
            client_call_details.timeout,
            Metadata(*self._metadata, *(client_call_details.metadata or ())),
            client_call_details.credentials,
            client_call_details.wait_for_ready,
        )
        return await continuation(details, request)


class _StreamMetadata(UnaryStreamClientInterceptor):
    def __init__(self, metadata: Sequence[tuple[str, str]]) -> None:
        self._metadata = tuple(metadata)

    async def intercept_unary_stream[Q, R](
        self,
        continuation: Callable[[ClientCallDetails, Q], Awaitable[UnaryStreamCall[Q, R]]],
        client_call_details: ClientCallDetails,
        request: Q,
    ) -> AsyncIterator[R] | UnaryStreamCall[Q, R]:
        details = ClientCallDetails(
            client_call_details.method,
            client_call_details.timeout,
            Metadata(*self._metadata, *(client_call_details.metadata or ())),
            client_call_details.credentials,
            client_call_details.wait_for_ready,
        )
        return await continuation(details, request)


# === Public Functions === #


async def connect(
    target: str | Path | None = None,
    *,
    connect_timeout: float = _CONNECT_TIMEOUT_S,
    certificate_pem: str | None = None,
    access_key: str | None = None,
    expected_backend_id: str | None = None,
) -> Client:
    if not isfinite(connect_timeout) or connect_timeout <= 0:
        raise ValueError("Connect: Supply a Positive Finite connect_timeout in Seconds")
    selected = resolve("DOKIME_REMOTE", str(target) if target is not None else None, load())
    if not selected:
        raise ValueError("Connect: Name an Endpoint or .pair File, or Set DOKIME_REMOTE")
    endpoints = [selected]
    if selected.endswith(".pair"):
        pairing = _pairing(Path(selected))
        certificate_pem = pairing["certificate_pem"]
        access_key = pairing["access_key"]
        expected_backend_id = pairing["backend_identifier"]
        endpoints = pairing["endpoints"]
    else:
        _endpoint(selected)
        if not selected.startswith("http://"):
            raise ValueError("Connect: Use a Direct HTTP Origin or a Pinned HTTPS Pairing File")
        if certificate_pem:
            raise ValueError("Connect: HTTP Cannot Carry a Certificate; Select a Pinned HTTPS Pairing File")
    failures: list[Failure] = []
    for selected in endpoints:
        try:
            return await _connect_endpoint(selected, connect_timeout, certificate_pem, access_key, expected_backend_id)
        except AioRpcError as error:
            failure = transport_error("connect", error).failure
            failures.append(replace(failure, details={**failure.details, "connection_endpoint": selected}))
        except (TimeoutError, ValueError) as error:
            reason = str(error) or type(error).__name__
            failures.append(
                Failure(
                    "Timed Out" if isinstance(error, TimeoutError) else "Refused",
                    type(error).__name__,
                    reason,
                    "Check the Host Listener or Export a New Pairing",
                    "connect",
                    details={"connection_endpoint": selected},
                )
            )
    message = "; ".join(f"{failure.details['connection_endpoint']}: {failure.message}" for failure in failures)
    classifications = {failure.classification for failure in failures}
    classification = failures[0].classification if len(classifications) == 1 else "Transient"
    raise SdkError(
        Failure(
            classification,
            "connection_failed",
            message,
            "Check the Host Listener or Export a New Pairing",
            "connect",
            cause=chain(failures),
        )
    )


# === Private Functions === #


async def _connect_endpoint(
    selected: str,
    connect_timeout: float,
    certificate_pem: str | None,
    access_key: str | None,
    expected_backend_id: str | None,
) -> Client:
    endpoint = _endpoint(selected)
    metadata = (("authorization", f"Bearer {access_key}"),) if access_key else ()
    interceptors = [_UnaryMetadata(metadata), _StreamMetadata(metadata)]
    if endpoint.scheme == "https":
        if not certificate_pem:
            raise ValueError("Connect: HTTPS Requires a Pinned Server Certificate")
        channel = secure_channel(endpoint.netloc, ssl_channel_credentials(certificate_pem.encode()), interceptors=interceptors)
    else:
        if certificate_pem:
            raise ValueError("Connect: HTTP Cannot Carry a Certificate")
        channel = insecure_channel(endpoint.netloc, interceptors=interceptors)
    wire = dokime_sdk._generated.client.Client(channel)
    wire._owned_channel = channel
    client = Client._bind(wire)
    try:
        await wait_for(_ready(channel, wire, expected_backend_id), connect_timeout)
        return client
    except BaseException:
        await client.close()
        raise


def _endpoint(selected: str) -> SplitResult:
    if any(character.isspace() or ord(character) < 32 for character in selected):
        raise ValueError("Connect: Remove Whitespace and Control Characters from the Endpoint")
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
        raise ValueError("Connect: Supply an HTTP(S) Origin with an Explicit Port or Select a .pair File")
    return endpoint


async def _ready(channel: Channel, client: dokime_sdk._generated.client.Client, expected_backend_id: str | None) -> None:
    if expected_backend_id:
        # Prove Identity With a Fail-Fast Call Before Waiting for Readiness.
        status = await client.status.get()
        if status.status.identity.host_id != expected_backend_id:
            raise ValueError("Connect: Host Identity Differs from the Paired Host; Export a Pairing From the Intended Host")
    await channel.channel_ready()


def _pairing(path: Path) -> Pairing:
    source = path.open("rb")
    try:
        content = source.read(1_048_577)
    finally:
        source.close()
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
    certificate = pairing["certificate_pem"]
    if certificate is not None and not isinstance(certificate, str):
        raise ValueError("Read Pairing: Certificate Must Be PEM Text")
    return Pairing(
        version=1, endpoints=list(endpoints), backend_identifier=identity, label=label, certificate_pem=certificate, access_key=key
    )


# === Entry Point === #
