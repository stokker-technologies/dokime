# ***
#
# Starter Project
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Type-Check the Three SDK Public Homes and Bound Client Flow."""

# === Imports === #

# Standard Library Imports
from typing import assert_type

# Third Party Imports
# Local Imports
import dokime_sdk.operations
import dokime_sdk.templates
from dokime_sdk import Case, Client, Cycle, Execution, Outcome, Plugin, Progress, Suite
from dokime_sdk.operations import Operation, Visibility, declaration, operation
from dokime_sdk.templates import TestTemplate, criterion, record, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ConfirmationTestTemplate(TestTemplate):
    confirmation = record(bool)

    @step("Confirm")
    async def confirm(self, ctx: dokime_sdk.templates.Context) -> None:
        assert_type(await ctx.prompt("Continue?", choices=["Yes", "No"]), str)
        await ctx.record(self.confirmation, True)
        assert_type(await ctx.records(self.confirmation, step=self.confirm).last(), bool)

    @criterion("Confirmed")
    async def confirmed(self, ctx: dokime_sdk.templates.Context) -> bool:
        return await ctx.records(self.confirmation, step=self.confirm).last()


class CaptureSuite(Suite):
    templates = (ConfirmationTestTemplate,)


class ExamplePlugin(Plugin):
    version = "1.0.0"
    suites = (CaptureSuite,)


# === Public Functions === #


@operation("example.echo", visibility=Visibility.PUBLIC)
async def echo(ctx: dokime_sdk.operations.Context, request: str) -> str:
    assert_type(ctx.plugin, Plugin)
    return request


async def client_surface(client: Client) -> None:
    assert_type(declaration(echo), Operation[str, str])
    await client.plugins.register(ExamplePlugin())
    cycle = await client.cycle(label="Capture")
    assert_type(cycle, Cycle)
    cases = await client.cases.materialize(ConfirmationTestTemplate)
    assert_type(cases, list[Case])
    for case in cases:
        execution = await cycle.execution(case)
        assert_type(execution, Execution)
        async for progress in execution.run():
            assert_type(progress, Progress)
        assert_type(await execution.seal(), Outcome)
        assert_type(await execution.evidence(), bytes)
    await cycle.seal()
    assert_type(await cycle.evidence(), bytes)
    assert_type(
        await client.operation(dokime_sdk.operations.Operation("example.echo", request=str, response=str, visibility=Visibility.PUBLIC))(
            "value"
        ),
        str,
    )
    await client.close()


# === Private Functions === #

# === Entry Point === #
