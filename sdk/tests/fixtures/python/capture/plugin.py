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

# isort: split

# Third Party Imports
from starter_project.plugin.suites.capture import CaptureSuite

# isort: split

# Local Imports
from dokime_sdk import Plugin

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


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
