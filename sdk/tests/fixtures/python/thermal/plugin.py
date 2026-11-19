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
import sys

# Third Party Imports
# Local Imports
from thermal_example.plugin import ThermalPlugin

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def main() -> None:
    plugin = ThermalPlugin()
    serving = asyncio.create_task(plugin.serve())
    try:
        await asyncio.to_thread(sys.stdin.buffer.read)
    finally:
        serving.cancel()
        await asyncio.gather(serving, return_exceptions=True)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
