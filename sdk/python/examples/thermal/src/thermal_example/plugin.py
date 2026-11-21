# ***
#
# Thermal Example
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""The Shared Thermal Procedure with Local Declarations."""

# === Imports === #

# Standard Library Imports
import asyncio
from datetime import timedelta

# isort: split

# Third Party Imports
from dokime_sdk import Plugin, Suite, operations
from dokime_sdk.operations import OperationPolicy, RetryPolicy, SettingDeclaration, SettingValue, Visibility, operation
from dokime_sdk.templates import Context, Parameter, TestTemplate, Window, criterion, observer, record, step, stream, window_observer
from google.protobuf.message import Message
from google.protobuf.wrappers_pb2 import BytesValue

# isort: split

# Local Imports
from thermal_example import sampling
from thermal_example.load import ThermalLoadTestTemplate
from thermal_example.models import Counter, CounterColumns, Door, Temperature

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ThermalCycleTestTemplate(TestTemplate):
    identifier = "cycle"
    label = "Thermal Cycle"
    parameters = [Parameter.number("limit", "Limit").default(85.0).build()]

    door = record(Door, label="Door Check")

    async def initialize(self, ctx: Context) -> None:
        limit = ctx.parameters["limit"]
        if not isinstance(limit, (float, int)):
            raise TypeError("Initialize Thermal Cycle: Limit Must Be Numeric")
        self.limit = limit
        await ctx.log("Initialized")

    @step("Ramp to Target", identifier="ramp", policy=OperationPolicy(retries=RetryPolicy(3, timedelta(milliseconds=100))))
    async def ramp(self, ctx: Context) -> None:
        await ctx.progress(1, 2, label="Ramp to Target")

    @step("Hold", identifier="hold", policy=OperationPolicy())
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
        return sampling.counter(window)

    @criterion("Temperature Within Limit", identifier="within_limit")
    async def within_limit(self, ctx: Context) -> bool:
        stats = await ctx.statistics(self.temperature, "celsius", step=self.hold)
        door = await ctx.records(self.door, step=self.hold).last()
        maximum = stats.max
        return (
            stats.count > 0
            and isinstance(maximum, (int, float))
            and maximum <= self.limit
            and door is not None
            and door.SerializeToString() == Door(closed=True).SerializeToString()
        )

    async def cleanup(self, ctx: Context) -> None:
        await ctx.log("Cleaned Up")


class ThermalSuite(Suite):
    templates = [ThermalCycleTestTemplate, ThermalLoadTestTemplate]


@operation("thermal.echo", visibility=Visibility.PUBLIC)
async def echo(ctx: operations.Context, request: BytesValue) -> BytesValue:
    return request


class ThermalPlugin(Plugin):
    version = "1.0.0"
    suites = [ThermalSuite]
    operations = [echo]
    settings = [SettingDeclaration("thermal.load.interval", "Load Interval", "", SettingValue(duration=timedelta(milliseconds=1)))]


# === Public Functions === #


async def main() -> None:
    plugin = ThermalPlugin()
    await plugin.serve()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
