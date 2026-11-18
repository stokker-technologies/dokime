# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""The Shared Thermal Procedure With Local Declarations."""

# === Imports === #

# Standard Library Imports
from typing import cast

# Third Party Imports
from google.protobuf import descriptor_pb2, descriptor_pool, message_factory
from google.protobuf.message import Message

from dokime import Context, Parameter, Plugin, Window, criterion, observer, record, step, stream, window_observer
from dokime.authoring import Suite, Template

# Local Imports

# Type Checking Imports


# Protobuf Types Belong to the Fixture, Not to the SDK.
def row_type(name: str, field: str, kind: int, *, repeated: bool = False) -> type[Message]:
    definition = descriptor_pb2.FileDescriptorProto(name=f"thermal/{name}.proto", package="thermal", syntax="proto3")
    row = definition.message_type.add(name=name)
    row.field.add(
        name=field,
        number=1,
        type=cast(descriptor_pb2.FieldDescriptorProto.Type, kind),
        label=descriptor_pb2.FieldDescriptorProto.LABEL_REPEATED if repeated else descriptor_pb2.FieldDescriptorProto.LABEL_OPTIONAL,
    )
    pool = descriptor_pool.Default()
    pool.Add(definition)
    return message_factory.GetMessageClass(pool.FindMessageTypeByName(f"thermal.{name}"))


Temperature = row_type("Temperature", "celsius", descriptor_pb2.FieldDescriptorProto.TYPE_DOUBLE)
Counter = row_type("Counter", "value", descriptor_pb2.FieldDescriptorProto.TYPE_UINT64)
CounterColumns = row_type("CounterColumns", "value", descriptor_pb2.FieldDescriptorProto.TYPE_UINT64, repeated=True)
Door = row_type("Door", "closed", descriptor_pb2.FieldDescriptorProto.TYPE_BOOL)


class Thermal:
    door = record(Door, label="Door Check")

    async def initialize(self, ctx: Context) -> None:
        limit = ctx.parameters["limit"]
        if not isinstance(limit, (float, int)):
            raise TypeError("Initialize Thermal Cycle: Limit Must Be Numeric")
        self.limit = limit
        await ctx.log("Initialized")

    @step("Ramp to Target", identifier="ramp", retries=2)
    async def ramp(self, ctx: Context) -> None:
        await ctx.progress(1, 2, label="Ramp to Target")

    @step("Hold", identifier="hold")
    async def hold(self, ctx: Context) -> None:
        answer = await ctx.prompt("Is the Door Closed?", choices=["Yes", "No"])
        await ctx.record(self.door, Door(closed=answer == "Yes"))
        await ctx.sleep(2.5)
        await ctx.progress(2, 2, label="Hold")

    @stream("Temperature [°C]", row=Temperature)
    @observer(interval_ms=1000)
    def temperature(self, ctx: Context) -> Message:
        return Temperature(celsius=self.limit - 1.0)

    # A 1 kHz Stream Answers Each Host Window at Once.
    @stream("Exact Counter", row=Counter)
    @window_observer(columns=CounterColumns, interval_ms=1)
    def counter(self, ctx: Context, window: Window) -> Message:
        return CounterColumns(value=[1_152_921_504_606_847_975] * window.count)

    @criterion("Temperature Within Limit", identifier="within_limit")
    async def within_limit(self, ctx: Context) -> bool:
        stats = await ctx.statistics(self.temperature, "celsius", step=self.hold)
        door = await ctx.records(self.door, step=self.hold).last()
        return (
            stats.statistics.count > 0
            and stats.statistics.max.number <= self.limit
            and door is not None
            and door.SerializeToString() == Door(closed=True).SerializeToString()
        )

    async def cleanup(self, ctx: Context) -> None:
        await ctx.log("Cleaned Up")


def plugin() -> Plugin:
    template = Template(
        Thermal,
        label="Thermal Cycle",
        identifier="cycle",
        parameters=[
            Parameter(
                metadata={"identifier": "limit", "label": "Limit"}, type="VALUE_TYPE_NUMBER", expression={"literal": {"number": 85.0}}
            )
        ],
    )
    return Plugin("thermal", "1.0.0", suites=[Suite("thermal", [template], "Thermal")])
