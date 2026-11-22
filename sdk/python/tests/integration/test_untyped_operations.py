# SPDX-License-Identifier: Apache-2.0

"""Untyped Operation Payloads Traverse the Library's Real Invocation Carrier."""

# === Imports === #

# Standard Library Imports
import json
from unittest import IsolatedAsyncioTestCase

# Third Party Imports
from grpc.aio import insecure_channel
from harmos._wire.plugin.v1.instance_pb2 import InstanceInvokeRequest, InstanceInvokeResponse
from harmos._wire.plugin.v1.invocation_pb2 import ChildResponse, Request

# Local Imports
from dokime_sdk import Plugin
from dokime_sdk._wire.model.v1.metadata_pb2 import Metadata
from dokime_sdk.operations import Context, operation

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class UntypedCarrierTests(IsolatedAsyncioTestCase):
    async def test_operation_id_and_dict_traverse_the_child_carrier_without_a_peer_contract(self) -> None:
        @operation("caller.measure")
        async def measure(ctx: Context, request: Metadata) -> Metadata:
            reply = await ctx.operation("device.measure", {"identifier": request.identifier})
            return Metadata(identifier=reply["identifier"], label=reply["label"])

        class CallerPlugin(Plugin):
            identifier = "caller"
            version = "1.0.0"
            operations = (measure,)

        owner = CallerPlugin()
        self.assertFalse(any(item.id == "device.measure" for item in owner.declarations().requires))
        endpoint = await owner._activate("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.stop)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        invoke = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Invoke",
            request_serializer=InstanceInvokeRequest.SerializeToString,
            response_deserializer=InstanceInvokeResponse.FromString,
        )
        call = invoke()
        try:
            await call.write(
                InstanceInvokeRequest(
                    request=Request(
                        operation="caller.measure",
                        payload=Metadata(identifier="reading").SerializeToString(),
                        invocation="measure",
                        attempt=1,
                    )
                )
            )
            frame = await call.read()
            assert isinstance(frame, InstanceInvokeResponse)
            self.assertEqual(frame.WhichOneof("frame"), "child")
            self.assertEqual(frame.child.operation, "device.measure")
            self.assertEqual(json.loads(frame.child.json), {"identifier": "reading"})
            self.assertFalse(frame.child.fingerprint)
            self.assertFalse(frame.child.payload)
            await call.write(
                InstanceInvokeRequest(child=ChildResponse(call=frame.child.call, payload=b'{"identifier":"reading","label":"Reading"}'))
            )
            frame = await call.read()
            assert isinstance(frame, InstanceInvokeResponse)
            self.assertFalse(frame.response.HasField("failure"), frame.response.failure)
            self.assertEqual(Metadata.FromString(frame.response.payload), Metadata(identifier="reading", label="Reading"))
        finally:
            call.cancel()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
