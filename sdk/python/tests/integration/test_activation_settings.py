# SPDX-License-Identifier: Apache-2.0

"""Typed Settings Are Loaded Through the Carrier Once per Activation."""

# === Imports === #

# Standard Library Imports
from unittest import IsolatedAsyncioTestCase

# Third Party Imports
from google.protobuf.wrappers_pb2 import BoolValue
from grpc.aio import insecure_channel
from harmos import OperationError
from harmos._wire.host.v1.plugins.settings_pb2 import SettingsListResponse
from harmos._wire.plugin.v1.instance_pb2 import InstanceInvokeRequest, InstanceInvokeResponse
from harmos._wire.plugin.v1.invocation_pb2 import ChildResponse, Request

# Local Imports
from dokime_sdk import Plugin
from dokime_sdk.operations import Context, Resource, Setting, Settings, operation

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CaptureSettings(Settings):
    enabled: bool = Setting.flag("Enabled", default=False)
    count: int = Setting.integer("Count", default=0, min=0, max=10)


class ActivationSettingsStory(IsolatedAsyncioTestCase):
    async def test_failed_settings_snapshot_stays_unpublished_on_the_library_carrier(self) -> None:
        initialized: list[bool] = []

        class ExamplePlugin(Plugin):
            version = "1.0.0"
            settings = (CaptureSettings,)

            async def initialize(self, ctx: Context) -> None:
                initialized.append(True)

        owner = ExamplePlugin()
        endpoint = await owner._activate("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.stop)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        invoke = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Invoke",
            request_serializer=InstanceInvokeRequest.SerializeToString,
            response_deserializer=InstanceInvokeResponse.FromString,
        )
        for values, message in (
            ({}, "Configure enabled"),
            ({"enabled": {"flag": False}, "count": {"text": "wrong"}}, "Save the Declared Type for count"),
        ):
            call = invoke()
            try:
                await call.write(InstanceInvokeRequest(request=Request(operation="example.initialize", invocation="activate", attempt=1)))
                frame = await call.read()
                assert isinstance(frame, InstanceInvokeResponse)
                self.assertTrue(frame.HasField("child"), frame)
                saved = SettingsListResponse(value={"values": values})
                await call.write(InstanceInvokeRequest(child=ChildResponse(call=frame.child.call, payload=saved.SerializeToString())))
                frame = await call.read()
                assert isinstance(frame, InstanceInvokeResponse)
                failure = OperationError(frame.response.failure)
                self.assertEqual(failure.kind, "permanent")
                self.assertEqual(failure.operation, "example.initialize")
                self.assertIn(message, failure.message)
                self.assertFalse(owner._settings_values)
                self.assertFalse(initialized)
            finally:
                call.cancel()

    async def test_library_lifecycle_failures_release_every_resource_in_reverse_order(self) -> None:
        for fail_initialize in (True, False):
            closed: list[str] = []

            class Connection(Resource):
                async def close(self) -> None:
                    closed.append("connection")

            class RemoteApi(Resource):
                async def open(self, ctx: Context) -> None:
                    await ctx.resource(Connection)

                async def close(self) -> None:
                    closed.append("api")
                    if not fail_initialize:
                        raise ValueError("Reconnect Before Closing")

            class ExamplePlugin(Plugin):
                version = "1.0.0"
                resources = (RemoteApi, Connection)

                async def initialize(self, ctx: Context) -> None:
                    await ctx.resource(RemoteApi)
                    if fail_initialize:
                        raise ValueError("Reconnect the Device")

                async def finalize(self, ctx: Context) -> None:
                    raise ValueError("Finalize Failed")

            owner = ExamplePlugin()
            endpoint = await owner._activate("127.0.0.1:0")
            self.addAsyncCleanup(endpoint.stop)
            channel = insecure_channel(endpoint.address)
            self.addAsyncCleanup(channel.close)
            invoke = channel.stream_stream(
                "/harmos.plugin.v1.Instance/Invoke",
                request_serializer=InstanceInvokeRequest.SerializeToString,
                response_deserializer=InstanceInvokeResponse.FromString,
            )
            for name in ("initialize",) if fail_initialize else ("initialize", "finalize"):
                call = invoke()
                try:
                    identity = f"example.{name}"
                    await call.write(InstanceInvokeRequest(request=Request(operation=identity, invocation=name, attempt=1)))
                    frame = await call.read()
                    assert isinstance(frame, InstanceInvokeResponse)
                    failed = fail_initialize or name == "finalize"
                    self.assertEqual(frame.response.HasField("failure"), failed, frame)
                    if failed:
                        failure = OperationError(frame.response.failure)
                        self.assertEqual(failure.kind, "permanent")
                        self.assertEqual(failure.operation, identity)
                        self.assertTrue(failure.message)
                finally:
                    call.cancel()
            self.assertEqual(closed, ["api", "connection"])
            self.assertFalse(owner._resource_values)

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
