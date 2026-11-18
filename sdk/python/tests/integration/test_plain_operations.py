# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""TODO: Describe this module's responsibility."""

# === Imports === #

# Standard Library Imports
import unittest

# Third Party Imports
from google.protobuf.message import Message
from google.protobuf.timestamp_pb2 import Timestamp
from google.protobuf.wrappers_pb2 import BoolValue, DoubleValue, Int64Value, StringValue
from grpc.aio import insecure_channel
from harmos._wire.host.v1.time_pb2 import TimeGetResponse
from harmos._wire.plugin.v1.instance_pb2 import InstanceInvokeRequest, InstanceInvokeResponse
from harmos._wire.plugin.v1.invocation_pb2 import ChildResponse, Request

from dokime import Context, Operation, Plugin, Visibility, criterion, prompt, record, step, suite
from dokime._generated.host import HostOperation, host_requirement
from dokime._wire.host.v1.evidence.records_pb2 import CapturedRecord, RecordsAppendRequest, RecordsListRequest, RecordsListResponse
from dokime._wire.host.v1.prompts_pb2 import PromptsCreateRequest, PromptsCreateResponse
from dokime._wire.plugin.v1.criteria_pb2 import CriteriaEvaluateRequest, CriteriaEvaluateResponse
from dokime._wire.plugin.v1.steps_pb2 import StepsExecuteRequest
from dokime.operations import operation

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class PlainValueStory(unittest.IsolatedAsyncioTestCase):
    async def test_operations_use_plain_values_over_the_real_carrier(self) -> None:
        increment = Operation("example.increment", int, int, Visibility.PRIVATE)

        class Operations:
            @operation("example.flag", request=bool, response=bool, visibility=Visibility.PUBLIC)
            def flag(self, ctx: Context, request: bool) -> bool:
                return not request

            @operation("example.text", request=str, response=str, visibility=Visibility.PUBLIC)
            async def text(self, ctx: Context, request: str) -> str:
                return request.upper()

            @operation("example.number", request=float, response=float, visibility=Visibility.PUBLIC)
            def number(self, ctx: Context, request: float) -> float:
                return request * 2

            @operation("example.integer", request=int, response=int, visibility=Visibility.PUBLIC)
            async def integer(self, ctx: Context, request: int) -> int:
                return await ctx.operation(increment)(request)

            @operation("example.increment", request=int, response=int, visibility=Visibility.PRIVATE)
            def increment(self, ctx: Context, request: int) -> int:
                return request + 1

            @operation("example.structured", request=BoolValue, response=BoolValue, visibility=Visibility.PUBLIC)
            def structured(self, ctx: Context, request: BoolValue) -> BoolValue:
                return BoolValue(value=not request.value)

        plugin = Plugin("plain", "1.0.0", operations=Operations())
        endpoint = await plugin.start("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)
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

    async def test_records_roundtrip_plain_values_and_bound_step_filters(self) -> None:
        @suite()
        class CaptureSuite:
            question = prompt("Choose a Reading", choices=["Yes", "No"])
            flag = record(bool)
            text = record(str)
            integer = record(int)
            number = record(float)
            structured = record(BoolValue)

            @step("Save")
            async def save(self, ctx: Context) -> None:
                answer = await ctx.prompt("Continue?", choices=["Yes", "No"])
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

        plugin = Plugin("records", "1.0.0", suites=[CaptureSuite])
        endpoint = await plugin.start("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)
        channel = insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        invoke = channel.stream_stream(
            "/harmos.plugin.v1.Instance/Invoke",
            request_serializer=InstanceInvokeRequest.SerializeToString,
            response_deserializer=InstanceInvokeResponse.FromString,
        )
        template = plugin.suites[0].templates[0]
        case = template.build({}).message
        origin = {"plugin_run": {"plugin": plugin.identifier, "template": template.identifier, "test_case": case}}
        requests: list[tuple[str, Message]] = [
            ("capture.save", StepsExecuteRequest(execution="execution", step="capture.save", test_case=case, origin=origin)),
            ("capture.matches", CriteriaEvaluateRequest(execution="execution", criterion="capture.matches", test_case=case, origin=origin)),
        ]
        requirements = {host_requirement(item).id: (item, host_requirement(item)) for item in HostOperation}
        records: dict[str, CapturedRecord] = {}
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
                        if operation_id == "capture.matches":
                            self.assertTrue(CriteriaEvaluateResponse.FromString(frame.response.payload).passed)
                        break
                    kind, requirement = requirements[frame.child.operation]
                    child = requirement.request.FromString(frame.child.payload)
                    response = requirement.response()
                    if kind == HostOperation.TIME_GET:
                        response = TimeGetResponse(now=Timestamp())
                    elif kind == HostOperation.PROMPTS_CREATE:
                        asked = PromptsCreateRequest.FromString(frame.child.payload)
                        if asked.prompt_type:
                            self.assertEqual(asked.prompt_type, "capture.question")
                        action = next(action for action in asked.definition.actions if action.label == "No")
                        response = PromptsCreateResponse(action=action.identifier, accepted=True)
                    elif kind == HostOperation.EVIDENCE_RECORDS_APPEND:
                        child = RecordsAppendRequest.FromString(frame.child.payload)
                        records[child.kind] = CapturedRecord(payload=child.payload, record_type=child.kind)
                    elif kind == HostOperation.EVIDENCE_RECORDS_LIST:
                        child = RecordsListRequest.FromString(frame.child.payload)
                        self.assertEqual(child.step, "capture.save")
                        last = records.get(child.record_type)
                        response = RecordsListResponse(count=int(last is not None), last=last)
                    else:
                        self.fail(f"Unexpected Host Call: {kind}")
                    await call.write(
                        InstanceInvokeRequest(child=ChildResponse(call=frame.child.call, payload=response.SerializeToString()))
                    )
            finally:
                call.cancel()
        self.assertEqual(set(records), {"capture.flag", "capture.text", "capture.integer", "capture.number", "capture.structured"})


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
