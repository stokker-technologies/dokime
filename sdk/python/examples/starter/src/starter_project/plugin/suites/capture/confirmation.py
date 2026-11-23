# ***
#
# Starter Project
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Ask the Operator to Confirm and Record the Answer."""

# === Imports === #

# Standard Library Imports
import logging

# Third Party Imports
from dokime_sdk import operations
from dokime_sdk.templates import Context, Observer, TestTemplate, criterion, record, step

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

log = logging.getLogger(__name__)
CONFIRMATION = record(bool, identifier="confirmation", label="Confirmation")

# === Types === #

# === Classes === #


class Temperature(Observer[float]):
    identifier = "temperature"
    label = "Temperature [°C]"
    row = float
    interval_ms = 100

    def sample(self, ctx: operations.Context) -> float:
        return 21.5


class ConfirmationTestTemplate(TestTemplate):
    uses_streams = (Temperature,)

    @step("Confirm")
    async def confirm(self, ctx: Context) -> None:
        answer = await ctx.prompt("Continue?", choices=["Yes", "No"])
        log.info("Operator Answered %s", answer)
        reply = await ctx.operation("example.echo", {"confirmed": answer == "Yes"})
        await ctx.record(CONFIRMATION, reply["confirmed"])

    @criterion("Confirmed")
    async def confirmed(self, ctx: Context) -> bool:
        return await ctx.records(CONFIRMATION, step=self.confirm).last()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
