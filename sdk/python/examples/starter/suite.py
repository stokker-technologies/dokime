# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Deterministic Suite Shared With The Cross-Language Host Story."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from google.protobuf.wrappers_pb2 import BoolValue

import dokime

# Local Imports
from dokime.v1 import message

# Type Checking Imports

# === Public Exports === #

__all__ = ["Capture", "plugin"]

# === Constants === #

READING = dokime.Stream("example.reading")
DOCUMENT = dokime.Document("Capture").section(
    dokime.Section("Evidence")
    .plot(dokime.plot.time_series(READING.field()))
    .summary(dokime.Summary().tile("Maximum", READING.field().max()))
)

# === Types === #

# === Classes === #


@dokime.suite(
    "example.capture",
    label="Capture",
    parameters=[message("Parameter", metadata={"identifier": "limit", "label": "Limit"}, type=4, expression={"literal": {"number": 10.0}})],
    documents=[DOCUMENT],
)
class Capture:
    async def initialize(self, ctx):
        self.reading = 0.0
        self.limit = ctx.parameters["limit"]
        self.attempts = 0

    @dokime.stream("Reading", identifier="example.reading")
    @dokime.observer(interval_ms=200)
    def reading_sample(self, ctx):
        return self.reading

    @dokime.step("Measure", retries=1)
    async def measure(self, ctx):
        self.attempts += 1
        self.reading = self.limit - 1.0
        await ctx.record("example.confirmation", value=True)
        await ctx.progress(1, 2, label="Measured")

    @dokime.step("Confirm")
    async def confirm(self, ctx):
        answer = await ctx.prompt("Continue?", choices=["Yes", "No"])
        await ctx.record("example.confirmation", value=answer == "Yes")
        await ctx.sleep(1.0)

    @dokime.criterion("Within Limit")
    async def within_limit(self, ctx):
        statistics = await ctx.statistics("example.reading", step="measure")
        confirmation = await ctx.records("example.confirmation").last()
        return statistics.count > 0 and statistics.max < self.limit and confirmation is not None and confirmation.value

    async def cleanup(self, ctx):
        await ctx.log("Capture resources released")


# === Public Functions === #


def plugin():
    return dokime.Plugin("example", "0.1.0", suites=[Capture], records=[dokime.Record("example.confirmation", BoolValue, "Confirmation")])


# === Private Functions === #

# === Entry Point === #
