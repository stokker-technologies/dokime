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
    return Metadata(identifier=request.identifier, label=f"Probe {request.identifier}", created_at=request.created_at)


@operation("probe.double")
async def double(ctx: Context, value: float) -> float:
    return value * 2


class ProbePlugin(Plugin):
    version = "1.0.0"
    operations = (describe, double)


class ClientOperationStory(IsolatedAsyncioTestCase):
    async def test_client_calls_operations_by_identifier(self) -> None:
        specification = spec_from_file_location("operation_host", Path(__file__).resolve().parents[1] / "support/host.py")
        assert specification is not None and specification.loader is not None
        module = module_from_spec(specification)
        specification.loader.exec_module(module)
        host = await module.Host.start(self)
        await host.client.plugins.register(ProbePlugin())

        # Plugin Operations Take and Return Protobuf JSON With Proto Field Names; Scalars Travel as {"value": ...}.
        reply = await host.client.operation("probe.describe", {"identifier": "bench", "created_at": "2026-11-23T08:00:00Z"})
        self.assertEqual(
            reply,
            {
                "identifier": "bench",
                "label": "Probe bench",
                "description": "",
                "tags": [],
                "attributes": {},
                "created_at": "2026-11-23T08:00:00Z",
            },
        )
        self.assertEqual(await host.client.operation("probe.double", {"value": 1.5}), {"value": 3.0})

        # Host Operations Answer Through Their gRPC Projection.
        discovered = await host.client.operation("dokime.host.v1.Operations/Get", {"id": "probe.double"})
        self.assertEqual(discovered["operation"]["owner"], "probe")

        # Unknown Identifiers, Mistyped Requests and Execution Scopes Are Refused With an Action.
        for operation_id, payload, execution, code in (
            ("probe.missing", None, None, "operation_not_found"),
            ("probe.describe", {"identifier": 1}, None, "invalid_request"),
            ("probe.describe", None, "execution", "host_capability_required"),
        ):
            with self.subTest(code=code), self.assertRaises(OperationError) as refusal:
                await host.client.operation(operation_id, payload, execution=execution)
            self.assertEqual((refusal.exception.operation, refusal.exception.value.code), (operation_id, code))
            self.assertTrue(refusal.exception.action)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
