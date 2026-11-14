from typing import cast

from google.protobuf.message import Message

from dokime import Parameter, Prompt, PromptDefinitions
from dokime.authoring import Record, Suite, Template, criterion, observer, step, stream
from dokime.plugin import Context, Plugin

# SPDX-License-Identifier: Apache-2.0
"""The Same Thermal Procedure as Every SDK."""

from google.protobuf import descriptor_pb2, descriptor_pool, message_factory

import dokime
from dokime.plugin import HostOperation


# Protobuf Types Belong to the Fixture, Not to the SDK.
def row_type(name: str, field: str, kind: int) -> type[Message]:
    definition = descriptor_pb2.FileDescriptorProto(name=f"thermal/{name}.proto", package="thermal", syntax="proto3")
    row = definition.message_type.add(name=name)
    row.field.add(
        name=field,
        number=1,
        type=cast(descriptor_pb2.FieldDescriptorProto.Type, kind),
        label=descriptor_pb2.FieldDescriptorProto.LABEL_OPTIONAL,
    )
    pool = descriptor_pool.Default()
    pool.Add(definition)
    return message_factory.GetMessageClass(pool.FindMessageTypeByName(f"thermal.{name}"))


Temperature = row_type("Temperature", "celsius", descriptor_pb2.FieldDescriptorProto.TYPE_DOUBLE)
Door = row_type("Door", "closed", descriptor_pb2.FieldDescriptorProto.TYPE_BOOL)


class Thermal:
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
        answer = await ctx.host.prompts.create("door", "thermal.door", {})
        await ctx.record("thermal.door", Door(closed=answer.action == "yes" and answer.accepted))
        await ctx.sleep(2.5)
        await ctx.progress(2, 2, label="Hold")

    @stream("Temperature [°C]", identifier="thermal.temperature", row=Temperature)
    @observer(interval_ms=1000)
    def temperature(self, ctx: Context) -> Message:
        return Temperature(celsius=self.limit - 1.0)

    @criterion("Temperature Within Limit", identifier="within_limit")
    async def within_limit(self, ctx: Context) -> bool:
        stats = await ctx.statistics("thermal.temperature", "celsius", step="hold")
        door = await ctx.records("thermal.door", step="hold").last()
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
        "thermal.cycle",
        Thermal,
        "Thermal Cycle",
        parameters=[
            Parameter(
                metadata={"identifier": "limit", "label": "Limit"}, type="VALUE_TYPE_NUMBER", expression={"literal": {"number": 85.0}}
            )
        ],
    )
    prompt = PromptDefinitions(
        prompts=[
            Prompt(
                metadata={"identifier": "thermal.door", "label": "Door Check"},
                message="Is the Door Closed?",
                archetype="PROMPT_ARCHETYPE_CONFIRMATION",
                actions=[{"identifier": "yes", "label": "Yes"}, {"identifier": "no", "label": "No"}],
            )
        ],
    )
    needed = [
        HostOperation.PROMPTS_CREATE,
        HostOperation.EVIDENCE_RECORDS_APPEND,
        HostOperation.EVIDENCE_RECORDS_LIST,
        HostOperation.EVIDENCE_STATISTICS,
        HostOperation.EXECUTIONS_PROGRESS_GET,
        HostOperation.EXECUTIONS_PROGRESS_SET,
        HostOperation.EVIDENCE_LOGS_APPEND,
        HostOperation.TIME_GET,
        HostOperation.TIME_SLEEP,
    ]
    return dokime.Plugin(
        "thermal",
        "1.0.0",
        suites=[Suite("thermal", [template], "Thermal")],
        records=[Record("thermal.door", Door, "Door Check")],
        sections=[prompt],
        requires=needed,
    )
