# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Capture Measurements After Operator Confirmation"""

# === Imports === #

# Standard Library Imports
# Third Party Imports
import dokime

# Local Imports
from ..operations.read import Read

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dokime.parameters
class Parameters:
    samples: int = dokime.parameter(label="Number of Samples", default=4, min=1, max=1000)


@dokime.stream(id="bench.readings", label="Readings", window=1000)
class Row:
    index: int
    voltage_v: float = dokime.parameter(label="Voltage [V]", unit="V")


@dokime.prompt(id="bench.confirm", label="Confirm Connections")
class Confirm:
    connected: bool


DOCUMENT = dokime.Document("Capture Measurements").section(
    dokime.Section("Readings").plot(dokime.plot.time_series(Row.field("voltage_v")).label("Voltage").unit("V"))
)


@dokime.template(
    id="bench.capture", label="Capture", parameters=Parameters, requires=[Read], streams=[Row], prompts=[Confirm], document=DOCUMENT
)
class Template:
    @dokime.step(label="Confirm Connections")
    async def confirm(self, ctx: dokime.Context) -> None:
        # Ask the Operator
        prompt = Confirm(False)
        answer = await ctx.ask(prompt)
        if not answer.accepted or not answer.values.get("connected"):
            raise ValueError("connections were not confirmed")

    @dokime.step(label="Read Voltage")
    async def capture(self, ctx: dokime.Context) -> None:
        # Open the Stream
        writer = ctx.stream(Row).open()
        for index in range(ctx.parameters.samples):
            # Read Voltage
            operation = Read(index)
            voltage_v = await ctx.operation(operation)

            # Publish the Reading
            row = Row(index, voltage_v)
            await writer.push(row)

    @staticmethod
    @dokime.criterion(label="All Measurements Captured")
    def measurements_ok(ctx: dokime.Context) -> bool:
        # Read Captured Evidence
        rows = ctx.evidence()["samples"]

        # Check / Number of Samples
        number_ok = len(rows) == ctx.parameters.samples
        return number_ok


@dokime.suite(id="bench", label="Bench", templates=[Template])
class Suite:
    pass


# === Public Functions === #


# === Private Functions === #

# === Entry Point === #
