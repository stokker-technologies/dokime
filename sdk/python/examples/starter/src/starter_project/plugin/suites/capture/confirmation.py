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

# Third Party Imports
from dokime_sdk.templates import Context, TestTemplate, criterion, record, step

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ConfirmationTestTemplate(TestTemplate):
    confirmation = record(bool, label="Confirmation")

    @step("Confirm")
    async def confirm(self, ctx: Context) -> None:
        answer = await ctx.prompt("Continue?", choices=["Yes", "No"])
        await ctx.record(self.confirmation, answer == "Yes")

    @criterion("Confirmed")
    async def confirmed(self, ctx: Context) -> bool:
        return await ctx.records(self.confirmation, step=self.confirm).last()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
