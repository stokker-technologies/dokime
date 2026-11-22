# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Plain Values Across Product Declarations and the Carrier."""

# === Imports === #

# Standard Library Imports
import unittest
from collections.abc import Callable
from datetime import timedelta
from typing import Any, ClassVar, cast

# Third Party Imports
from google.protobuf import descriptor_pool, message_factory
from google.protobuf.descriptor_pb2 import FieldDescriptorProto, FileDescriptorProto
from google.protobuf.json_format import MessageToDict
from google.protobuf.message import Message
from google.protobuf.timestamp_pb2 import Timestamp
from google.protobuf.wrappers_pb2 import BoolValue, DoubleValue, Int64Value, StringValue
from grpc.aio import insecure_channel
from harmos import VISIBILITY_PROTECTED
from harmos._columns import columns_type
from harmos._wire.host.v1.plugins.settings_pb2 import SettingsListResponse
from harmos._wire.host.v1.time_pb2 import TimeGetResponse
from harmos._wire.plugin.v1.instance_pb2 import (
    InstanceInvokeRequest,
    InstanceInvokeResponse,
    InstanceSamplesRequest,
    InstanceSamplesResponse,
)
from harmos._wire.plugin.v1.invocation_pb2 import ChildResponse, Request

# Local Imports
import dokime_sdk.operations
from dokime_sdk import Failure, Plugin, Suite
from dokime_sdk.operations import FailureError, Operation, SettingValue, Visibility, operation
from dokime_sdk.templates import Context, TestTemplate, Window, criterion, observer, prompt, record, step, stream, window_observer

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class PlainValueStory(unittest.IsolatedAsyncioTestCase):
    async def test_settings_values(self) -> None:
        expected = {
            "enabled": SettingValue(flag=False),
            "count": SettingValue(integer=0),
            "threshold": SettingValue(number=0.0),
            "label": SettingValue(text=""),
            "interval": SettingValue(duration=timedelta()),
        }

        @operation("example.settings", visibility=Visibility.PUBLIC)
        async def read(ctx: dokime_sdk.operations.Context, request: bool) -> bool:
            return await ctx.settings() == expected

        class SettingsPlugin(Plugin):
            identifier = "example"
            version = "1.0.0"
            operations: ClassVar[list[Callable[..., object]]] = [read]

        endpoint = await SettingsPlugin()._activate("127.0.0.1:0")
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
                        operation="example.settings",
                        payload=BoolValue(value=True).SerializeToString(),
                        invocation="settings",
                        attempt=1,
                    )
                )
            )
            frame = await call.read()
            assert isinstance(frame, InstanceInvokeResponse)
            self.assertTrue(frame.HasField("child"), frame)
            response = SettingsListResponse(
                value={
                    "values": {
                        "enabled": {"flag": False},
                        "count": {"integer": 0},
                        "threshold": {"number": 0.0},
                        "label": {"text": ""},
                        "interval": {"duration": {}},
                    }
                }
            )
            await call.write(InstanceInvokeRequest(child=ChildResponse(call=frame.child.call, payload=response.SerializeToString())))
            frame = await call.read()
            assert isinstance(frame, InstanceInvokeResponse)
            self.assertFalse(frame.response.HasField("failure"), frame.response.failure)
            self.assertTrue(BoolValue.FromString(frame.response.payload).value)
        finally:
            call.cancel()

    async def test_observer_presence(self) -> None:
        definition = FileDescriptorProto(name="optional_reading.proto", package="reading", syntax="proto3")
        message = definition.message_type.add(name="Reading")
        message.oneof_decl.add(name="_value")
        message.field.add(
            name="value",
            number=1,
            type=FieldDescriptorProto.TYPE_DOUBLE,
            label=FieldDescriptorProto.LABEL_OPTIONAL,
            proto3_optional=True,
            oneof_index=0,
        )
        pool = descriptor_pool.DescriptorPool()
        pool.Add(definition)
        row_type = message_factory.GetMessageClass(pool.FindMessageTypeByName("reading.Reading"))
        columns = columns_type(row_type, "readings.reading")
        rows = (row_type(), row_type(value=0.0), row_type(value=2.5))
        pending = iter(rows)

        class ReadingsPlugin(Plugin):
            version = "1.0.0"

            @stream("Reading", row=row_type)
            @observer()
            def reading(self, ctx: dokime_sdk.operations.Context) -> Message:
                return next(pending)

            @stream("Window", row=row_type)
            @window_observer(columns=columns)
            def window(self, ctx: dokime_sdk.operations.Context, window: Window) -> Message:
                return columns(value=[{}, {"value": 0.0}, {"value": 2.5}])

        endpoint = await ReadingsPlugin()._activate("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.stop)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        samples = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Samples",
            request_serializer=InstanceSamplesRequest.SerializeToString,
            response_deserializer=InstanceSamplesResponse.FromString,
        )
        for name in ("reading", "window"):
            call = samples()
            try:
                await call.write(InstanceSamplesRequest(open={"stream": f"readings.{name}"}))
                await call.write(InstanceSamplesRequest(window={"start": {}, "count": 3, "period": {"seconds": 1}}))
                response = await call.read()
                assert isinstance(response, InstanceSamplesResponse)
                self.assertTrue(response.HasField("batch"), response)
                decoded = columns.FromString(response.batch.columns)
                values = list(getattr(decoded, "value"))
                self.assertEqual([value.HasField("value") for value in values], [False, True, True])
                self.assertEqual([getattr(value, "value") if value.HasField("value") else None for value in values], [None, 0.0, 2.5])
            finally:
                call.cancel()

    async def test_plugin_hooks_are_protected_operations_with_the_author_context(self) -> None:
        events: list[str] = []

        class LifecyclePlugin(Plugin):
            version = "1.0.0"

            async def initialize(self, ctx: dokime_sdk.operations.Context) -> None:
                self.assert_context(ctx)
                events.append("initialize")

            async def finalize(self, ctx: dokime_sdk.operations.Context) -> None:
                self.assert_context(ctx)
                events.append("finalize")

            def assert_context(self, ctx: dokime_sdk.operations.Context) -> None:
                if ctx.plugin is not self:
                    raise ValueError("Invoke Lifecycle: Preserve the Author Plugin")

        plugin = LifecyclePlugin()
        declarations = plugin.declarations()
        endpoint = await plugin._activate("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.stop)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        invoke = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Invoke",
            request_serializer=InstanceInvokeRequest.SerializeToString,
            response_deserializer=InstanceInvokeResponse.FromString,
        )
        for name in ("initialize", "finalize"):
            identifier = f"{plugin.identifier}.{name}"
            declaration = next(value for value in declarations.operations if value.id == identifier)
            self.assertEqual(declaration.request, "google.protobuf.Empty")
            self.assertEqual(declaration.response, "google.protobuf.Empty")
            self.assertEqual(declaration.visibility, VISIBILITY_PROTECTED)
            call = invoke()
            try:
                await call.write(InstanceInvokeRequest(request=Request(operation=identifier, invocation=name, attempt=1)))
                response = await call.read()
                assert isinstance(response, InstanceInvokeResponse)
                self.assertFalse(response.response.HasField("failure"), response.response.failure)
            finally:
                call.cancel()
        self.assertEqual(events, ["initialize", "finalize"])

    async def test_operations_use_plain_values_over_the_real_carrier(self) -> None:
        increment_operation = Operation("example.increment", int, int, Visibility.PRIVATE)

        @operation("example.flag", visibility=Visibility.PUBLIC)
        async def flag(ctx: dokime_sdk.operations.Context, request: bool) -> bool:
            self.assertIs(ctx.plugin, plugin)
            self.assertFalse(hasattr(ctx, "execution"))
            self.assertFalse(hasattr(ctx, "parameters"))
            return not request

        @operation("example.text", visibility=Visibility.PUBLIC)
        async def text(ctx: dokime_sdk.operations.Context, request: str) -> str:
            return request.upper()

        @operation("example.number", visibility=Visibility.PUBLIC)
        async def number(ctx: dokime_sdk.operations.Context, request: float) -> float:
            return request * 2

        @operation("example.integer", visibility=Visibility.PUBLIC)
        async def integer(ctx: dokime_sdk.operations.Context, request: int) -> int:
            return await ctx.operation(increment_operation)(request)

        @operation("example.increment", visibility=Visibility.PRIVATE)
        async def increment(ctx: dokime_sdk.operations.Context, request: int) -> int:
            return request + 1

        @operation("example.structured", visibility=Visibility.PUBLIC)
        async def structured(ctx: dokime_sdk.operations.Context, request: BoolValue) -> BoolValue:
            return BoolValue(value=not request.value)

        class PlainPlugin(Plugin):
            identifier = "example"
            version = "1.0.0"
            operations: ClassVar[list[Callable[..., object]]] = [flag, text, number, integer, increment, structured]

        plugin = PlainPlugin()
        endpoint = await plugin._activate("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.stop)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        invoke = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Invoke",
            request_serializer=InstanceInvokeRequest.SerializeToString,
            response_deserializer=InstanceInvokeResponse.FromString,
        )
        cases = [
            ("flag", BoolValue(value=True), BoolValue(value=False)),
            ("text", StringValue(value="hello"), StringValue(value="HELLO")),
            ("number", DoubleValue(value=2.125), DoubleValue(value=4.25)),
            ("integer", Int64Value(value=2**60), Int64Value(value=2**60 + 1)),
            ("structured", BoolValue(value=False), BoolValue(value=True)),
        ]
        for name, request, expected in cases:
            call = invoke()
            try:
                await call.write(
                    InstanceInvokeRequest(
                        request=Request(operation=f"example.{name}", payload=request.SerializeToString(), invocation=name, attempt=1)
                    )
                )
                response = await call.read()
                assert isinstance(response, InstanceInvokeResponse), "Invoke Operation: Expected a Response Before End of Stream"
                self.assertFalse(response.response.HasField("failure"), response.response.failure)
                actual = type(expected).FromString(response.response.payload)
                self.assertEqual(actual, expected)
            finally:
                call.cancel()

    async def test_author_failures_keep_classification_and_causes_on_the_carrier(self) -> None:
        @operation("example.failure", visibility=Visibility.PUBLIC)
        async def fail(ctx: dokime_sdk.operations.Context, request: bool) -> bool:
            raise FailureError(
                Failure(
                    "Transient",
                    "outer_failure",
                    "Read Failed",
                    "Retry Read",
                    cause=Failure("Refused", "inner_failure", "Device Refused", "Check Device"),
                    details={"device": "42"},
                    diagnostics="trace_42",
                )
            )

        class FailurePlugin(Plugin):
            identifier = "example"
            version = "1.0.0"
            operations: ClassVar[list[Callable[..., object]]] = [fail]

        author = FailurePlugin()
        endpoint = await author._activate("127.0.0.1:0")
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
                        operation="example.failure", payload=BoolValue(value=True).SerializeToString(), invocation="failure", attempt=1
                    )
                )
            )
            response = await call.read()
            assert isinstance(response, InstanceInvokeResponse)
            failure = response.response.failure
            self.assertEqual(failure.code, "outer_failure")
            self.assertEqual(failure.cause.code, "inner_failure")
            self.assertEqual(dict(failure.details), {"device": "42"})
            self.assertEqual(failure.diagnostics, "trace_42")
            self.assertNotEqual(failure.classification, failure.cause.classification)
        finally:
            call.cancel()

    async def test_scalar_streams_use_declared_payload_types(self) -> None:
        class ReadingsTestTemplate(TestTemplate):
            @step("Start")
            def start(self, ctx: Context) -> None:
                pass

            @stream("Voltage")
            @observer()
            def voltage(self, ctx: Context) -> float:
                return 2.5

            @stream("Accepted", row=bool)
            @observer()
            def accepted(self, ctx: Context) -> bool:
                return True

        class ReadingsSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [ReadingsTestTemplate]

        class StreamsPlugin(Plugin):
            version = "1.0.0"
            suites: ClassVar[list[type[Suite]]] = [ReadingsSuite]

        plugin = StreamsPlugin()
        endpoint = await plugin._activate("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.stop)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        template = plugin._suites[0].templates[0]
        case = template.build({}).message
        origin = {"plugin_run": {"plugin": plugin.identifier, "template": template.identifier, "test_case": case}}
        invoke = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Invoke",
            request_serializer=InstanceInvokeRequest.SerializeToString,
            response_deserializer=InstanceInvokeResponse.FromString,
        )
        call = invoke()
        try:
            definition = next(item for item, _ in plugin._plugin._registrations()[0] if item.id == "streams.readings.readings.start")
            request = cast(Any, definition.request)(
                execution="execution", step="streams.readings.readings.start", test_case=case, origin=origin
            )
            await call.write(
                InstanceInvokeRequest(
                    request=Request(
                        operation="streams.readings.readings.start",
                        payload=request.SerializeToString(),
                        invocation="start",
                        scope="execution",
                        attempt=1,
                    )
                )
            )
            response = await call.read()
            assert isinstance(response, InstanceInvokeResponse)
            self.assertFalse(response.response.HasField("failure"), response.response.failure)
        finally:
            call.cancel()
        samples = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Samples",
            request_serializer=InstanceSamplesRequest.SerializeToString,
            response_deserializer=InstanceSamplesResponse.FromString,
        )
        for name, row, expected in (("voltage", DoubleValue, [2.5, 2.5]), ("accepted", BoolValue, [True, True])):
            call = samples()
            try:
                await call.write(InstanceSamplesRequest(open={"stream": f"streams.readings.readings.{name}", "scope": "execution"}))
                await call.write(InstanceSamplesRequest(window={"start": {}, "count": 2, "period": {"seconds": 1}}))
                response = await call.read()
                assert isinstance(response, InstanceSamplesResponse)
                self.assertEqual(response.batch.count, 2)
                values = columns_type(row, f"streams.readings.readings.{name}").FromString(response.batch.columns)
                self.assertEqual(MessageToDict(values)["value"], expected)
            finally:
                call.cancel()

    async def test_records_roundtrip_plain_values_and_bound_step_filters(self) -> None:
        story = self

        class CaptureTestTemplate(TestTemplate):
            question = prompt("Choose a Reading", choices=["Yes", "No"])
            flag = record(bool)
            text = record(str)
            integer = record(int)
            number = record(float)
            structured = record(BoolValue)

            @step("Save")
            async def save(self, ctx: Context) -> None:
                answer = await ctx.prompt("Continue?", choices=["Yes", "No"])
                story.assertIs(ctx.plugin, plugin)
                try:
                    await ctx.record(self.integer, cast(Any, True))
                except TypeError:
                    pass
                else:
                    story.fail("A Boolean Must Not Encode as an Integer Record")
                await ctx.record(self.flag, answer == "Yes")
                await ctx.record(self.text, await ctx.prompt(self.question))
                await ctx.record(self.integer, 2**60 + 3)
                await ctx.record(self.number, 2.125)
                await ctx.record(self.structured, BoolValue(value=True))

            @criterion("Matches")
            async def matches(self, ctx: Context) -> bool:
                return (
                    await ctx.records(self.flag, step=self.save).last() is False
                    and await ctx.records(self.text, step=self.save).last() == "No"
                    and await ctx.records(self.integer, step=self.save).last() == 2**60 + 3
                    and await ctx.records(self.number, step=self.save).last() == 2.125
                    and await ctx.records(self.structured, step=self.save).last() == BoolValue(value=True)
                )

        class CaptureSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [CaptureTestTemplate]

        class RecordsPlugin(Plugin):
            version = "1.0.0"
            suites: ClassVar[list[type[Suite]]] = [CaptureSuite]

        plugin = RecordsPlugin()
        endpoint = await plugin._activate("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.stop)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        invoke = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Invoke",
            request_serializer=InstanceInvokeRequest.SerializeToString,
            response_deserializer=InstanceInvokeResponse.FromString,
        )
        template = plugin._suites[0].templates[0]
        case = template.build({}).message
        origin = {"plugin_run": {"plugin": plugin.identifier, "template": template.identifier, "test_case": case}}
        definitions = {item.id: item for item, _ in plugin._plugin._registrations()[0]}
        requests: list[tuple[str, Message]] = [
            (
                "records.capture.capture.save",
                cast(Any, definitions["records.capture.capture.save"].request)(
                    execution="execution", step="records.capture.capture.save", test_case=case, origin=origin
                ),
            ),
            (
                "records.capture.capture.matches",
                cast(Any, definitions["records.capture.capture.matches"].request)(
                    execution="execution", criterion="records.capture.capture.matches", test_case=case, origin=origin
                ),
            ),
        ]
        requirements = {item.id: item for item in plugin._plugin.requires}
        records: dict[str, bytes] = {}
        for operation_id, request in requests:
            call = invoke()
            try:
                await call.write(
                    InstanceInvokeRequest(
                        request=Request(
                            operation=operation_id,
                            payload=request.SerializeToString(),
                            invocation=operation_id,
                            scope="execution",
                            attempt=1,
                        )
                    )
                )
                while True:
                    frame = await call.read()
                    assert isinstance(frame, InstanceInvokeResponse), "Invoke Template: Expected a Response Before End of Stream"
                    if frame.HasField("response"):
                        self.assertFalse(frame.response.HasField("failure"), frame.response.failure)
                        if operation_id == "records.capture.capture.matches":
                            self.assertTrue(cast(Any, definitions[operation_id].response.FromString(frame.response.payload)).passed)
                        break
                    requirement = requirements[frame.child.operation]
                    kind = frame.child.operation
                    child = cast(Any, requirement.request.FromString(frame.child.payload))
                    response = cast(Any, requirement.response())
                    if kind == "harmos.plugin.v1.Time/Get":
                        response = TimeGetResponse(now=Timestamp())
                    elif kind == "dokime.host.v1.Prompts/Create":
                        asked = child
                        if asked.prompt_type:
                            self.assertEqual(asked.prompt_type, "records.capture.capture.question")
                        action = next(action for action in asked.definition.actions if action.label == "No")
                        response.action = action.identifier
                        response.accepted = True
                    elif kind == "dokime.host.v1.evidence.Records/Append":
                        records[child.kind] = child.payload
                    elif kind == "dokime.host.v1.evidence.Records/List":
                        self.assertEqual(child.step, "records.capture.capture.save")
                        last = records.get(child.record_type)
                        response.count = int(last is not None)
                        if last is not None:
                            response.last.payload = last
                            response.last.record_type = child.record_type
                            response.last.scope.execution_id = "execution"
                    else:
                        self.fail(f"Unexpected Host Call: {kind}")
                    await call.write(
                        InstanceInvokeRequest(child=ChildResponse(call=frame.child.call, payload=response.SerializeToString()))
                    )
            finally:
                call.cancel()
        self.assertEqual(
            set(records),
            {
                "records.capture.capture.flag",
                "records.capture.capture.text",
                "records.capture.capture.integer",
                "records.capture.capture.number",
                "records.capture.capture.structured",
            },
        )


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
