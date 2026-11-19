# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Run the Stream-Free Confirmation Fixture for the Prompt Deadline Story."""

# === Imports === #

# Standard Library Imports
import asyncio
import runpy
import sys
from pathlib import Path
from typing import cast

# Third Party Imports
# Local Imports
from dokime_sdk import Plugin, connect
from dokime_sdk.templates import TestTemplate

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def main() -> None:
    # Load the Existing Stream-Free Authoring Fixture.
    fixture = runpy.run_path(sys.argv[1])
    plugin = cast(type[Plugin], fixture["ExamplePlugin"])
    template = cast(type[TestTemplate], fixture["ConfirmationTestTemplate"])

    # Register the Plugin and Materialize Its Confirmation Case.
    client = await connect()
    try:
        await client.plugins.register(plugin())
        cycle = await client.cycle(label="Prompt Deadlines")
        cases = await client.cases.materialize(template)
        for case in cases:
            # Expose Progress So the Story Can Hold and Answer the Prompt.
            execution = await cycle.execution(case)
            async for progress in execution.run():
                print(progress.phase, flush=True)
            await execution.seal()

        # Seal the Cycle and Save Its Retained Evidence.
        await cycle.seal()
        evidence = await cycle.evidence()
        output = Path("_output")
        output.mkdir()
        output.joinpath("evidence.zip").write_bytes(evidence)
    finally:
        await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
