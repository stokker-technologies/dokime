# ***
#
# Thermal Example
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Serve the Plugin and Run Its Suite Through a Running Dokime."""

# === Imports === #

# Standard Library Imports
import asyncio
from datetime import datetime
from pathlib import Path

# Third Party Imports
from dokime_sdk import ResultStatus, connect
from rich.console import Console

# Local Imports
from thermal_example.plugin import ThermalCycleTestTemplate, ThermalPlugin

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Where This Run Writes Its Artifacts; Ignored by Git.
OUTPUT = Path("_output")

console = Console()

# === Types === #

# === Classes === #

# === Public Functions === #


async def main() -> None:
    # Initialize Client
    client = await connect()
    OUTPUT.mkdir(exist_ok=True)

    # Initialize & Register Plugin
    plugin = ThermalPlugin()
    await client.plugins.register(plugin)
    console.rule("Thermal Example")
    console.print(f"Plugin [bold]{plugin.identifier}[/bold] Is Ready")

    # Initialize Test Cycle
    started = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    cycle = await client.cycle(label=f"Thermal {started}")

    # Initialize Test Cases
    cases = await client.cases.materialize(ThermalCycleTestTemplate)

    for case in cases:
        # Initialize Test Execution
        execution = await cycle.execution(case)

        # Run Test Execution
        console.print(f"Running [bold]{case.label}[/bold]")
        async for progress in execution.run():
            console.print(f"  {progress.phase}")

        # Seal Test Execution & Report Outcome
        outcome = await execution.seal()

        if outcome.status == ResultStatus.PASSED:
            console.print("[bold green]Passed[/bold green]")
        else:
            console.print("[bold red]Not Passed[/bold red]")

    # Seal Test Cycle
    await cycle.seal()

    # Save Evidence
    evidence = await cycle.evidence()
    saved = OUTPUT / f"{cycle.identifier}.zip"
    saved.write_bytes(evidence)
    console.print(f"Saved [cyan]{saved}[/cyan]")

    # Close Client
    await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
