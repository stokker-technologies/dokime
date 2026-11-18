# ***
#
# Thermal Example
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""The Shared Thermal Procedure With Local Declarations."""

# === Imports === #

# Standard Library Imports
import asyncio

# isort: split

# Third Party Imports
from dokime_sdk import Plugin, Suite
from dokime_sdk.templates import Context, Parameter, TestTemplate, Window, criterion, observer, record, step, stream, window_observer
from google.protobuf.message import Message

# isort: split

# Local Imports
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

    async def setup(self, ctx: Context) -> None:
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
    templates = [ThermalCycleTestTemplate]


class ThermalPlugin(Plugin):
    version = "1.0.0"
    suites = [ThermalSuite]


# === Public Functions === #


async def main() -> None:
    plugin = ThermalPlugin()
    await plugin.serve()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
