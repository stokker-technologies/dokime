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
import logging
from datetime import timedelta

# isort: split

# Third Party Imports
from dokime_sdk import Plugin, Suite, operations
from dokime_sdk.operations import OperationPolicy, RetryPolicy, Setting, Settings, Visibility, operation
from dokime_sdk.templates import Context, Observer, Parameter, TestTemplate, Window, criterion, record, step
from google.protobuf.message import Message
from google.protobuf.wrappers_pb2 import BytesValue

# isort: split

# Local Imports
from thermal_example import sampling
from thermal_example.load import CHANNELS, Burst, ThermalLoadTestTemplate
from thermal_example.models import Counter, CounterColumns, Door, Temperature

# Type Checking Imports

# === Public Exports === #

# === Constants === #

log = logging.getLogger(__name__)

# === Types === #

# === Classes === #


class CounterObserver(Observer[Message]):
    identifier = "counter"
    label = "Exact Counter"
    row = Counter
    columns = CounterColumns
    interval_ms = 1

    def sample(self, ctx: operations.Context, window: Window) -> Message:
        return sampling.counter(window)


class TemperatureObserver(Observer[Message]):
    identifier = "temperature"
    label = "Temperature [°C]"
    row = Temperature
    interval_ms = 1000

    def __init__(self) -> None:
        self.temperature_celsius = 23.0

    def sample(self, ctx: operations.Context) -> Message:
        # Approach the Chamber Target on Each Granted Sample.
        self.temperature_celsius += (84.0 - self.temperature_celsius) * 0.25
        return Temperature(celsius=self.temperature_celsius)


class ThermalCycleTestTemplate(TestTemplate):
    uses_streams = (TemperatureObserver, CounterObserver)
    identifier = "cycle"
    label = "Thermal Cycle"
    limit = Parameter.number("Limit", default=85.0)

    door = record(Door, label="Door Check", identifier="door")

    async def initialize(self, ctx: Context) -> None:
        limit = ctx.parameter(ThermalCycleTestTemplate.limit)
        if not isinstance(limit, (float, int)):
            raise TypeError("Initialize Thermal Cycle: Limit Must Be Numeric")
        self.target_celsius = limit
        log.info("Initialized")

    @step("Ramp to Target", identifier="ramp", policy=OperationPolicy(retries=RetryPolicy(3, timedelta(milliseconds=100))))
    async def ramp(self, ctx: Context) -> None:
        await ctx.progress(1, 2, label="Ramp to Target")

    @step("Hold", identifier="hold", policy=OperationPolicy())
    async def hold(self, ctx: Context) -> None:
        answer = await ctx.prompt("Is the Door Closed?", choices=["Yes", "No"])
        await ctx.record(self.door, Door(closed=answer == "Yes"))
        await ctx.sleep(2.5)
        await ctx.progress(2, 2, label="Hold")

    # A 1 kHz Stream Answers Each Host Window at Once.

    @criterion("Temperature Within Limit", identifier="within_limit")
    async def within_limit(self, ctx: Context) -> bool:
        stats = await ctx.statistics(TemperatureObserver, "celsius", step=self.hold)
        door = await ctx.records(self.door, step=self.hold).last()
        maximum = stats.max
        return (
            stats.count > 0
            and isinstance(maximum, (int, float))
            and maximum <= self.target_celsius
            and door is not None
            and door.SerializeToString() == Door(closed=True).SerializeToString()
        )

    async def cleanup(self, ctx: Context) -> None:
        log.info("Cleaned Up")


class ThermalSuite(Suite):
    templates = [ThermalCycleTestTemplate, ThermalLoadTestTemplate]


@operation("thermal.echo", visibility=Visibility.PUBLIC)
async def echo(ctx: operations.Context, request: BytesValue) -> BytesValue:
    return request


class ThermalSettings(Settings):
    load_interval: timedelta = Setting.duration("Load Interval", default=timedelta(milliseconds=1))


class ThermalPlugin(Plugin):
    version = "1.0.0"
    suites = [ThermalSuite]
    records = (ThermalCycleTestTemplate.door,)
    operations = [echo]
    settings = (ThermalSettings,)
    observers = (TemperatureObserver, CounterObserver, *CHANNELS, Burst)


# === Public Functions === #


async def main() -> None:
    plugin = ThermalPlugin()
    await plugin.serve()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
