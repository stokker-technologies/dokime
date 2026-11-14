from typing import Any

# SPDX-License-Identifier: Apache-2.0
"""The Same Thermal Procedure as Every SDK."""

from google.protobuf import descriptor_pb2, descriptor_pool, message_factory

import dokime
from dokime import HOST_OPERATIONS
from dokime.v1 import message


# Protobuf Types Belong to the Fixture, Not to the SDK.
def row_type(name: Any, field: Any, kind: Any) -> Any:
    definition = descriptor_pb2.FileDescriptorProto(name=f"thermal/{name}.proto", package="thermal.v1", syntax="proto3")
    row = definition.message_type.add(name=name)
    row.field.add(name=field, number=1, type=kind, label=1)
    pool = descriptor_pool.Default()
    pool.Add(definition)
    return message_factory.GetMessageClass(pool.FindMessageTypeByName(f"thermal.v1.{name}"))


Temperature = row_type("Temperature", "celsius", descriptor_pb2.FieldDescriptorProto.TYPE_DOUBLE)
Door = row_type("Door", "closed", descriptor_pb2.FieldDescriptorProto.TYPE_BOOL)


class Thermal:
    async def initialize(self, ctx: Any) -> Any:
        self.limit = ctx.parameters["limit"]
        await ctx.log("Initialized")

    @dokime.step("Ramp to Target", identifier="ramp", retries=2)
    async def ramp(self, ctx: Any) -> Any:
        await ctx.progress(1, 2, label="Ramp to Target")

    @dokime.step("Hold", identifier="hold")
    async def hold(self, ctx: Any) -> Any:
        question = next(operation for operation in HOST_OPERATIONS if operation.id == "dokime.plugin.host.v1.Prompts/Create")
        answer = await ctx.operation(question)(
            message("dokime.plugin.host.v1.PromptsCreateRequest", prompt="door", prompt_type="thermal.door")
        )
        await ctx.record("thermal.door", Door(closed=answer.action == "yes" and answer.accepted))
        await ctx.sleep(2.5)
        await ctx.progress(2, 2, label="Hold")

    @dokime.stream("Temperature [°C]", identifier="thermal.temperature", row=Temperature)
    @dokime.observer(interval_ms=1000)
    def temperature(self, ctx: Any) -> Any:
        return Temperature(celsius=self.limit - 1.0)

    @dokime.criterion("Temperature Within Limit", identifier="within_limit")
    async def within_limit(self, ctx: Any) -> Any:
        stats = await ctx.statistics("thermal.temperature", "celsius", step="hold")
        door = await ctx.records("thermal.door", step="hold").last()
        return stats.count > 0 and stats.max <= self.limit and door is not None and door.closed

    async def cleanup(self, ctx: Any) -> Any:
        await ctx.log("Cleaned Up")


def plugin() -> Any:
    template = dokime.Template(
        "thermal.cycle",
        Thermal,
        "Thermal Cycle",
        parameters=[
            message("Parameter", metadata={"identifier": "limit", "label": "Limit"}, type=4, expression={"literal": {"number": 85.0}})
        ],
    )
    prompt = message(
        "PromptDefinitions",
        prompts=[
            message(
                "Prompt",
                metadata={"identifier": "thermal.door", "label": "Door Check"},
                message="Is the Door Closed?",
                archetype=1,
                actions=[{"identifier": "yes", "label": "Yes"}, {"identifier": "no", "label": "No"}],
            )
        ],
    )
    needed = {
        "dokime.plugin.host.v1.Prompts/Create",
        "dokime.plugin.host.v1.evidence.Records/Append",
        "dokime.plugin.host.v1.evidence.Records/List",
        "dokime.plugin.host.v1.Evidence/Statistics",
        "dokime.plugin.host.v1.executions.Progress/Get",
        "dokime.plugin.host.v1.executions.Progress/Set",
        "dokime.plugin.host.v1.evidence.Logs/Append",
        "harmos.context.v1.Time/Sleep",
    }
    return dokime.Plugin(
        "thermal",
        "1.0.0",
        suites=[dokime.Suite("thermal", [template], "Thermal")],
        records=[dokime.Record("thermal.door", Door, "Door Check")],
        sections=[prompt],
        requires=[operation for operation in HOST_OPERATIONS if operation.id in needed],
    )
