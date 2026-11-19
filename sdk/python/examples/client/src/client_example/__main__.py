# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Connect to a Running Host and Browse Its Composed Resources."""

# === Imports === #

# Standard Library Imports
import asyncio

# Third Party Imports
# Local Imports
from dokime_sdk import connect

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def main() -> None:
    client = await connect()
    try:
        async for cycle in client.cycles.list():
            print(f"{cycle.identifier}: {cycle.label}")
    finally:
        await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
