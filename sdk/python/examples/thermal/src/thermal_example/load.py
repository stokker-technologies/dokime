# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Declare the Shared Thermal Load Streams"""

# === Imports === #

# Standard Library Imports
from collections.abc import Mapping

# isort: split

# Third Party Imports
from dokime_sdk import Scalar
from dokime_sdk.operations import OperationPolicy
from dokime_sdk.templates import Context, Materialization, Parameter, TestTemplate, Observer, Window, materialize, step
from google.protobuf.message import Message

# isort: split

# Local Imports
from thermal_example import sampling
from thermal_example.models import Counter, CounterColumns

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


CHANNELS = tuple(sampling.channel(f"channel_{index:03}") for index in range(128))


class Burst(Observer[Counter]):
    row = Counter
    columns = CounterColumns
    interval_ms = 1
    interval_setting = "load_interval"
    label = "Single-Stream Ceiling"

    def sample(self, ctx: Context, window: Window) -> Message:
        return sampling.counter(window)


class ThermalLoadTestTemplate(TestTemplate):
    identifier = "load"
    label = "Thermal Load"
    burst_enabled = Parameter.flag("Burst", default=False)
    uses_streams = (*CHANNELS, Burst)

    async def initialize(self, ctx: Context) -> None:
        await ctx.progress(0, 1, label="Hold")

    @materialize
    def select(self, case: Materialization, parameters: Mapping[str, Scalar]) -> None:
        burst = parameters["burst_enabled"]
        retained = [stream for stream in case.message.streams if stream.endswith(".burst") == burst]
        del case.message.streams[:]
        case.message.streams.extend(retained)
        case.step("Hold", self.hold)

    @step("Hold", identifier="hold", policy=OperationPolicy())
    async def hold(self, ctx: Context) -> None:
        await ctx.cancelled()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
