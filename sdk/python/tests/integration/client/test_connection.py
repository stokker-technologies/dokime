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
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

# Third Party Imports
import pytest

# Local Imports
from dokime.client import (
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
            # Apply the Declared Backend Shape
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

            # Connect and Classify
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
                with pytest.raises(BackendChanged):
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
        "ftp://localhost:8765",
        "https://user:pass@host:8765",
        "https://host:8765/path",
        "localhost:8765",
    ],
)
def test_endpoint_refusals(endpoint: str) -> None:
    with pytest.raises(ValueError):
        DokimeClient(endpoint)


@pytest.mark.parametrize("endpoint", ["http://lab.example:8765", "https://lab.example:8765", "http://192.0.2.1:8765"])
def test_transport_secrets(endpoint: str, monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.delenv("DOKIME_KEY", raising=False)
    client = DokimeClient(endpoint)
    assert client._transport._channel is None
    assert client._transport._access_key is None
    if endpoint.startswith("http:"):
        with pytest.raises(ValueError):
            DokimeClient(endpoint, ca_certificate_pem="certificate")
    else:
        with pytest.raises(ValueError):
            DokimeClient(endpoint, ca_certificate_pem="certificate")


def test_pairing_checks_identity_before_calls_and_after_reconnect(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (server, _):
            path = tmp_path / "lab.dokime-remote"
            document = {
                "version": 1,
                "endpoints": [server.endpoint],
                "backend_identifier": server.backend_id,
                "label": "Lab",
                "ca_certificate_pem": None,
                "access_key": None,
            }
            path.write_text(json.dumps(document), encoding="utf-8")
            selected = DokimeClient.from_pairing(path)
            try:
                assert (await selected.connect()).id == server.backend_id
                await selected._transport._channel.close()
                selected._transport._channel = None
                selected._transport.backend = None
                server.backend_id = "replacement-backend"
                with pytest.raises(BackendChanged):
                    await selected.create_cycle(label="Refused After Reconnect")
                assert server.created == 0
                assert selected._transport._channel is None
            finally:
                await selected.close()

            path.write_text(json.dumps(document), encoding="utf-8")
            mismatched = DokimeClient.from_pairing(path)
            try:
                with pytest.raises(BackendChanged):
                    await mismatched.create_cycle(label="Refused First Connect")
                assert server.created == 0
            finally:
                await mismatched.close()

    asyncio.run(scenario())


def test_remote_candidates_choose_verified_identity_and_remember(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (stalled, _), serve() as (wrong, _), serve() as (good, _):
            stalled.identity_gate.clear()
            wrong.backend_id = "wrong-backend"
            good.backend_id = "expected-backend"
            document = {
                "version": 1,
                "endpoints": [stalled.endpoint, wrong.endpoint, good.endpoint],
                "backend_identifier": good.backend_id,
                "label": "Remote Test",
                "ca_certificate_pem": None,
                "access_key": None,
            }
            path = tmp_path / "remote.dokime-remote"
            path.write_text(json.dumps(document), encoding="utf-8")
            client = DokimeClient.from_pairing(path)
            try:
                assert (await client.connect()).id == good.backend_id
                assert client._transport._endpoint == good.endpoint
                assert wrong.created == good.created == 0
                await client._transport._channel.close()
                client._transport._channel = None
                client._transport.backend = None
                # A remembered success stays first even if another endpoint becomes valid.
                wrong.backend_id = good.backend_id
                assert (await client.connect()).id == good.backend_id
                assert client._transport._endpoint == good.endpoint
            finally:
                await client.close()
            stalled.identity_gate.set()
            document["endpoints"] = [wrong.endpoint, good.endpoint]
            document["backend_identifier"] = "missing-backend"
            path.write_text(json.dumps(document), encoding="utf-8")
            refused = DokimeClient.from_pairing(path)
            try:
                with pytest.raises(BackendChanged):
                    await refused.connect()
                assert refused._transport._channel is None
            finally:
                await refused.close()

    asyncio.run(scenario())


def test_query_tabular_core_imports() -> None:
    script = """
import importlib.abc
import sys
class MissingExtras(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname.split('.')[0] in ('polars', 'pandas'):
            raise ModuleNotFoundError(fullname, name=fullname)
sys.meta_path.insert(0, MissingExtras())
from dokime import DokimeClient
from dokime.client.query import tabular
assert not {'polars', 'pandas'}.intersection(sys.modules)
reader = DokimeClient().executions(status='failed')
assert not reader.closed
import asyncio
from unittest.mock import AsyncMock
from dokime import QueryDependencyError
from dokime.client import _wire
from dokime.client.execution import Execution
from dokime.client._contracts import operation
from dokime.client.models import Backend
async def scenario():
    client = DokimeClient()
    execution = Execution._bind('00000000-0000-0000-0000-000000000001', client._transport,
        Backend('example', 'epoch', '1', 'Example', frozenset()))
    reply = operation('Query', 'ListRecords').response()
    _wire.copy(reply, {'rows': [{'record_id': 'row-1', 'severity': {'Info': {}}, 'details': _wire.encode({'count': 7}, 64)}],
        'schema': {'identity': 'example.event'}, 'snapshot': {'token': 'cut', 'identity': 'cut-1'},
        'limits': {'rows': 10, 'bytes': 65536}, 'complete': True})
    client._transport.call = AsyncMock(return_value=reply)
    for extra in ('polars', 'pandas'):
        query = execution.query_records('example.event')
        try:
            await getattr(query, 'to_' + extra)()
        except QueryDependencyError as error:
            assert error.extra == extra
        else:
            raise AssertionError('Missing extra was not reported')
        assert client._transport.call.await_count == 0
        rows = [row async for row in query]
        assert rows[0].details['count'] == 7 and query.complete
        client._transport.call.reset_mock()
    await client.close()
asyncio.run(scenario())
"""
    environment = dict(os.environ, PYTHONPATH=os.pathsep.join(sys.path))
    result = subprocess.run([sys.executable, "-c", script], check=False, env=environment, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr


# === Private Functions === #

# === Entry Point === #
