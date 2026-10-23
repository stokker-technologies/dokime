# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Negotiate, Bound And Close Backend Connections"""

# === Imports === #

# Standard Library Imports
import asyncio
import json
from pathlib import Path
from typing import Any

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import (
    BackendChanged,
    ClientClosed,
    DokimeClient,
    ProtocolError,
    RpcError,
)

from .backend import SUITE_ID, collect, serve

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Rust Connection Stories Read The Same Negotiation Fixture.
NEGOTIATION = json.loads((Path(__file__).resolve().parent / "fixtures/negotiation.json").read_text())
READINESS = ("Unspecified", "Starting", "Ready", "Degraded", "Stopping")

# === Types === #

# === Classes === #

# === Public Functions === #


def test_connect_deadline_expires() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            # TODO(source-hook): injectable clock for Transport.timeout (dokime/client/_transport.py asyncio.timeout and gRPC deadline)
            client._transport.timeout = 0.05
            server.identity_gate.clear()
            with pytest.raises(RpcError) as error:
                await client.connect()
            assert error.value.code == "DEADLINE_EXCEEDED"
            assert client._transport.backend is None

            server.identity_gate.set()
            assert (await client.connect()).id == server.backend_id

    asyncio.run(scenario())


def test_call_deadline_propagates() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            await client.connect()
            client._transport.timeout = 5
            assert (await collect(client.suites()))[0].id == SUITE_ID

            # gRPC Rounds The Propagated Deadline, So Bound It Loosely
            assert server.catalog_deadline is not None
            assert 0 < server.catalog_deadline <= 2 * client._transport.timeout

    asyncio.run(scenario())


def test_close_cancels_calls() -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            await client.connect()
            server.catalog_gate.clear()
            pending = asyncio.create_task(collect(client.suites()))
            await server.catalog_started.wait()
            await client.close()
            with pytest.raises(ClientClosed):
                await pending
            await server.catalog_finished.wait()
            assert server.stop_calls == 0

    asyncio.run(scenario())


@pytest.mark.parametrize("case", NEGOTIATION, ids=lambda case: case["name"])
def test_backend_negotiation(case: dict[str, Any]) -> None:
    async def scenario() -> None:
        async with serve() as (server, client):
            # Apply The Declared Backend Shape
            for field in (
                "health_epoch",
                "availability_epoch",
                "max_request_bytes",
                "max_response_bytes",
            ):
                if field in case:
                    setattr(server, field, case[field])
            if "readiness" in case:
                server.readiness = {READINESS[case["readiness"]]: {}}
            server.version = case["version"]
            server.package = case.get("package", "dokime.v1")
            server.descriptor = bytes([case.get("descriptor", 0)]) * 32
            if missing := case.get("missing"):
                server.methods["Backend"].remove(missing.rsplit("/", 1)[1])

            # Connect And Classify
            if case["accepted"]:
                connected = await client.connect()
                assert connected.api_version == server.version
            elif case.get("error") == "unavailable":
                with pytest.raises(RpcError) as error:
                    await client.connect()
                assert error.value.code == "UNAVAILABLE"
            else:
                with pytest.raises(ProtocolError):
                    await client.connect()
            assert server.created == 0

    asyncio.run(scenario())


def test_expected_backend() -> None:
    async def scenario() -> None:
        async with serve() as (server, _):
            async with DokimeClient(server.endpoint, expected_backend_id=server.backend_id) as selected:
                assert (await selected.connect()).id == server.backend_id

            mismatched = DokimeClient(server.endpoint, expected_backend_id="another-backend")
            try:
                with pytest.raises(BackendChanged, match="selected profile identity"):
                    await mismatched.create_cycle(label="Mismatched")
                assert server.created == 0
                assert mismatched._transport.backend is None
                assert mismatched._transport._channel is None
            finally:
                await mismatched.close()

    asyncio.run(scenario())


@pytest.mark.parametrize(
    "endpoint",
    [
        "http://remote.example:8765",
        "ftp://localhost:8765",
        "https://user:pass@host:8765",
        "https://host:8765/path",
        "localhost:8765",
    ],
)
def test_endpoint_refusals(endpoint: str) -> None:
    with pytest.raises(ValueError):
        DokimeClient(endpoint)


# === Private Functions === #

# === Entry Point === #
