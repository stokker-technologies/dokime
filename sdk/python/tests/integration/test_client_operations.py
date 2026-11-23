# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Client Calls Served Operations by Identifier With Dict Payloads."""

# === Imports === #

# Standard Library Imports
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path
from unittest import IsolatedAsyncioTestCase

# Third Party Imports
from google.protobuf.json_format import MessageToDict
from google.protobuf.struct_pb2 import Struct

# Local Imports
from dokime_sdk import Plugin
from dokime_sdk._wire.model.v1.metadata_pb2 import Metadata
from dokime_sdk.operations import Context, OperationError, operation

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@operation("probe.describe")
async def describe(ctx: Context, request: Metadata) -> Metadata:
    return Metadata(
        identifier=request.identifier, label=f"Probe {request.identifier}", attributes=request.attributes, created_at=request.created_at
    )


@operation("probe.relay")
async def relay(ctx: Context, request: Struct) -> Struct:
    reply = Struct()
    reply.update(await ctx.operation("probe.describe", MessageToDict(request)))
    return reply


@operation("probe.double")
async def double(ctx: Context, value: float) -> float:
    return value * 2


class ProbePlugin(Plugin):
    version = "1.0.0"
    operations = (describe, relay, double)


class ClientOperationStory(IsolatedAsyncioTestCase):
    async def test_client_calls_operations_by_identifier(self) -> None:
        specification = spec_from_file_location("operation_host", Path(__file__).resolve().parents[1] / "support/host.py")
        assert specification is not None and specification.loader is not None
        module = module_from_spec(specification)
        specification.loader.exec_module(module)
        host = await module.Host.start(self)
        await host.client.plugins.register(ProbePlugin())

        # Replies Use Proto Field Names With Every Field Present; Unset Messages Read as null.
        request = {"identifier": "bench", "attributes": {"site": {"text": "lab"}}, "created_at": "2026-11-23T08:00:00Z"}
        reply = await host.client.operation("probe.describe", request)
        self.assertEqual((reply["label"], reply["description"], reply["tags"]), ("Probe bench", "", []))
        self.assertEqual((reply["created_at"], reply["updated_at"]), ("2026-11-23T08:00:00Z", None))
        self.assertEqual((reply["attributes"]["site"]["text"], reply["attributes"]["site"]["number"]), ("lab", None))

        # The Same Call Made by a Plugin Returns the Same Dict; Scalars Travel as {"value": ...}.
        self.assertEqual(await host.client.operation("probe.relay", request), reply)
        self.assertEqual(await host.client.operation("probe.double", {"value": 1.5}), {"value": 3.0})

        # Host Operations Answer Through Their gRPC Projection.
        discovered = await host.client.operation("dokime.host.v1.Operations/Get", {"id": "probe.double"})
        self.assertEqual(discovered["operation"]["owner"], "probe")

        # Unknown Identifiers and Mistyped Requests Are Refused With an Action.
        for operation_id, payload, code in (
            ("probe.missing", None, "operation_not_found"),
            ("probe.describe", {"identifier": 1}, "invalid_request"),
        ):
            with self.subTest(code=code), self.assertRaises(OperationError) as refusal:
                await host.client.operation(operation_id, payload)
            self.assertEqual((refusal.exception.operation, refusal.exception.value.code), (operation_id, code))
            self.assertTrue(refusal.exception.action)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
