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
from dokime_sdk.templates import Context, TestTemplate, criterion, observer, record, step, stream

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

log = logging.getLogger(__name__)
CONFIRMATION = record(bool, identifier="confirmation", label="Confirmation")

# === Types === #

# === Classes === #


class ConfirmationTestTemplate(TestTemplate):
    @step("Confirm")
    async def confirm(self, ctx: Context) -> None:
        answer = await ctx.prompt("Continue?", choices=["Yes", "No"])
        log.info("Operator Answered %s", answer)
        reply = await ctx.operation("example.echo", {"confirmed": answer == "Yes"})
        await ctx.record(CONFIRMATION, reply["confirmed"])

    @stream("Temperature [°C]", row=float)
    @observer(interval_ms=100)
    def temperature(self, ctx: Context) -> float:
        return 21.5

    @criterion("Confirmed")
    async def confirmed(self, ctx: Context) -> bool:
        return await ctx.records(CONFIRMATION, step=self.confirm).last()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
