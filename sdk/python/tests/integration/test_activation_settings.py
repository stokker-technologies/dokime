# SPDX-License-Identifier: Apache-2.0

"""Typed Settings Are Loaded Through the Carrier Once per Activation."""

# === Imports === #

# Standard Library Imports
from unittest import IsolatedAsyncioTestCase

# Third Party Imports
from google.protobuf.wrappers_pb2 import BoolValue
from grpc.aio import insecure_channel
from harmos._wire.host.v1.plugins.settings_pb2 import SettingsListResponse
from harmos._wire.plugin.v1.instance_pb2 import InstanceInvokeRequest, InstanceInvokeResponse
from harmos._wire.plugin.v1.invocation_pb2 import ChildResponse, Request

# Local Imports
from dokime_sdk import Plugin
from dokime_sdk.operations import Context, Setting, Settings, operation

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CaptureSettings(Settings):
    enabled: bool = Setting.flag("Enabled", default=False)
    count: int = Setting.integer("Count", default=0, min=0, max=10)


class ActivationSettingsStory(IsolatedAsyncioTestCase):
    async def test_initialize_loads_one_snapshot_and_handlers_read_it_without_host_calls(self) -> None:
        @operation("example.read_settings")
        async def read(ctx: Context, request: bool) -> bool:
            snapshot = ctx.settings(CaptureSettings)
            return snapshot.enabled == request and snapshot.count == 0

        class ExamplePlugin(Plugin):
            version = "1.0.0"
            settings = (CaptureSettings,)
            operations = (read,)

        endpoint = await ExamplePlugin()._activate("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.stop)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        invoke = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Invoke",
            request_serializer=InstanceInvokeRequest.SerializeToString,
            response_deserializer=InstanceInvokeResponse.FromString,
        )

        # Activate Through the Carrier and Supply the Host's Saved Values.
        call = invoke()
        try:
            await call.write(InstanceInvokeRequest(request=Request(operation="example.initialize", invocation="activate", attempt=1)))
            frame = await call.read()
            assert isinstance(frame, InstanceInvokeResponse)
            self.assertTrue(frame.HasField("child"), frame)
            saved = SettingsListResponse(value={"values": {"enabled": {"flag": False}, "count": {"integer": 0}}})
            await call.write(InstanceInvokeRequest(child=ChildResponse(call=frame.child.call, payload=saved.SerializeToString())))
            frame = await call.read()
            assert isinstance(frame, InstanceInvokeResponse)
            self.assertTrue(frame.HasField("response"), frame)
            self.assertFalse(frame.response.HasField("failure"), frame.response.failure)
        finally:
            call.cancel()

        # Read Synchronously: the Next Frame Is the Handler Result, Without a Child Call.
        call = invoke()
        try:
            await call.write(
                InstanceInvokeRequest(
                    request=Request(
                        operation="example.read_settings",
                        payload=BoolValue(value=False).SerializeToString(),
                        invocation="read",
                        attempt=1,
                    )
                )
            )
            frame = await call.read()
            assert isinstance(frame, InstanceInvokeResponse)
            self.assertTrue(frame.HasField("response"), frame)
            self.assertFalse(frame.response.HasField("failure"), frame.response.failure)
            self.assertTrue(BoolValue.FromString(frame.response.payload).value)
        finally:
            call.cancel()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
