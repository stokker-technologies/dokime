# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Exercise Live External Contracts Through Actual Canonical gRPC Calls"""

# === Imports === #

# Standard Library Imports
import asyncio
import copy
import json
from pathlib import Path

# Third Party Imports
import grpc
import pytest

# Local Imports
from dokime.client import DokimeClient, ProtocolError, _contracts
from dokime.dynamic import from_contract

from .backend import BackendFixture
from .canonical import register

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_invalid_external_contract_is_refused() -> None:
    contract = _contract("unary")
    contract["fingerprint"] = "02" * 32
    contract["operation"]["fingerprint"] = contract["fingerprint"]
    with pytest.raises(ValueError):
        from_contract(contract)


@pytest.mark.parametrize("mode", ["unary", "client_stream", "server_stream", "bidi"])
def test_external_operation(mode: str) -> None:
    async def scenario():
        contract = _contract(mode)
        operation = from_contract(contract)
        backend = BackendFixture()
        server = grpc.aio.server()
        register(server, backend)
        transmitted = []
        native_available = False
        invalid_catalog = False

        async def discovery(request, context):
            if invalid_catalog:
                return _contracts.operation("Host", "Catalog").response(entries=b"invalid")
            native = copy.deepcopy(contract)
            native.pop("grpc")

            return _contracts.operation("Host", "Catalog").response(
                entries=json.dumps(
                    [
                        {
                            "owner": "native_fixture",
                            "generation": 7,
                            "contract": native,
                            "grants": [],
                            "conditional_access": False,
                            "grpc_available": native_available,
                        },
                        {
                            "owner": "external_fixture",
                            "generation": 7,
                            "contract": contract,
                            "grants": [],
                            "conditional_access": True,
                            "grpc_available": True,
                        },
                    ]
                ).encode()
            )

        async def unary(request, context):
            transmitted.append(request.wide)
            await context.send_initial_metadata((("fixture-owner", "external_fixture"),))
            return operation.response(wide=request.wide)

        async def upload(requests, context):
            values = []
            async for envelope in requests:
                branch = envelope.WhichOneof("value")
                if not values:
                    assert branch == "request"
                else:
                    assert branch == "item"
                values.append(getattr(envelope, branch).wide)
            transmitted.extend(values)
            return operation.response(wide=sum(values))

        async def download(request, context):
            assert operation.output is not None
            transmitted.append(request.wide)
            yield operation.rpc_response(item=operation.output(wide=request.wide))
            yield operation.rpc_response(response=operation.response(wide=request.wide))

        async def exchange(requests, context):
            assert operation.output is not None
            values = []
            async for envelope in requests:
                branch = envelope.WhichOneof("value")
                assert branch == ("request" if not values else "item")
                values.append(getattr(envelope, branch).wide)
                yield operation.rpc_response(item=operation.output(wide=values[-1]))
            transmitted.extend(values)
            yield operation.rpc_response(response=operation.response(wide=sum(values)))

        factories = {
            "unary": grpc.unary_unary_rpc_method_handler,
            "client_stream": grpc.stream_unary_rpc_method_handler,
            "server_stream": grpc.unary_stream_rpc_method_handler,
            "bidi": grpc.stream_stream_rpc_method_handler,
        }
        callbacks = {
            "unary": unary,
            "client_stream": upload,
            "server_stream": download,
            "bidi": exchange,
        }
        handler = factories[mode](
            callbacks[mode],
            request_deserializer=operation.rpc_request.FromString,
            response_serializer=lambda result: result.SerializeToString(),
        )
        catalog_handler = grpc.unary_unary_rpc_method_handler(
            discovery,
            request_deserializer=_contracts.operation("Host", "Catalog").request.FromString,
            response_serializer=lambda result: result.SerializeToString(),
        )
        # Register The External Discovery Service Ahead Of The Standard Fixture Handler.
        server.add_generic_rpc_handlers([grpc.method_handlers_generic_handler("external.v1.Plugin", {"Invoke": handler})])
        server.add_registered_method_handlers("harmos.host.v1.Host", {"Catalog": catalog_handler})
        port = server.add_insecure_port("127.0.0.1:0")
        await server.start()
        client = DokimeClient(f"http://127.0.0.1:{port}")
        try:
            providers = await client.operations()
            assert [item.operation.id for item in providers] == ["external.fixture.invoke"]
            native_available = True
            with pytest.raises(ProtocolError):
                await client.operations()
            native_available = False
            invalid_catalog = True
            with pytest.raises(ProtocolError):
                await client.operations()
            invalid_catalog = False
            provider = await client.show("external.fixture.invoke")
            assert provider.owner == "external_fixture" and provider.generation == 7
            assert provider.conditional_access
            selected = provider.operation
            # A Request Constructed Before Rediscovery Retains Its Message Class.
            request = providers[0].operation.request(wide=2**63)

            async def items():
                assert selected.input is not None
                yield selected.input(wide=9)

            if mode == "unary":
                reply = await client.call(selected, request)
                assert reply.wide == 2**63
                assert (await client.call(selected.id, {"wide": str(2**63)})).wide == 2**63
                stale = _contract("server_stream")
                before = list(transmitted)
                with pytest.raises(ProtocolError):
                    await client.call(from_contract(stale), from_contract(stale).request(wide=1))
                assert transmitted == before
            elif mode == "client_stream":
                reply = await client.upload(selected, request, items())
                assert reply.wide == 2**63 + 9
            else:
                reader = client.download(selected, request) if mode == "server_stream" else client.exchange(selected, request, items())
                assert [item.wide async for item in reader] == ([2**63] if mode == "server_stream" else [2**63, 9])
                assert (await reader.response()).wide == 2**63 + (9 if mode == "bidi" else 0)
                await reader.close()
        finally:
            await client.close()
            await server.stop(0)

    asyncio.run(scenario())


# === Private Functions === #


def _contract(mode: str) -> dict:
    fixture = Path(__file__).with_name("fixtures") / "operations.json"
    return json.loads(fixture.read_text(encoding="utf-8"))[mode]


# === Entry Point === #
