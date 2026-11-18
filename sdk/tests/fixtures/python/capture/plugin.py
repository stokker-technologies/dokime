# ***
#
# Starter Project
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""The Example Plugin: Its Suites and Operations."""

# === Imports === #

# Standard Library Imports
import asyncio

# Third Party Imports
# Local Imports
from dokime_sdk import Plugin, Suite
from dokime_sdk.templates import Context, TestTemplate, criterion, record, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ConfirmationTestTemplate(TestTemplate):
    confirmation = record(bool)

    @step("Confirm")
    async def confirm(self, ctx: Context) -> None:
        answer = await ctx.prompt("Continue?", choices=["Yes", "No"])
        await ctx.record(self.confirmation, answer == "Yes")

    @criterion("Confirmed")
    async def confirmed(self, ctx: Context) -> bool:
        return await ctx.records(self.confirmation, step=self.confirm).last()


class CaptureSuite(Suite):
    templates = (ConfirmationTestTemplate,)


class CapturePlugin(Plugin):
    version = "1.0.0"
    suites = (CaptureSuite,)


# === Public Functions === #


async def main() -> None:
    await CapturePlugin().serve()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
