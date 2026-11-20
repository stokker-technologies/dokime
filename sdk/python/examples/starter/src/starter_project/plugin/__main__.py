# ***
#
# Starter Project
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Serve the Plugin; Dokime Starts This Module or Connects to It."""

# === Imports === #

# Standard Library Imports
import asyncio

# Third Party Imports
# Local Imports
from starter_project.plugin import ExamplePlugin

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def main() -> None:
    plugin = ExamplePlugin()
    endpoint = await plugin.start()
    if not endpoint.managed:
        print(f"Serving on {endpoint.address}", flush=True)
    await endpoint.wait()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
