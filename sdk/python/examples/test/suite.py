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
from dokime_sdk import Context, Plugin, criterion, record, step, suite

# Local Imports

# Type Checking Imports

# === Public Exports === #

__all__ = ["Capture", "plugin"]

# === Constants === #

# === Types === #

# === Classes === #


@suite(label="Capture")
class Capture:
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

plugin = Plugin("example", "0.1.0", suites=[Capture])
