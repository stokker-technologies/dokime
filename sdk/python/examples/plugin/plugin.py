# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Serve an Independent Typed Plugin Operation."""

# === Imports === #

# Standard Library Imports
import asyncio
import sys

# Third Party Imports
from operations import plugin

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


# === Public Functions === #


async def main() -> None:
    endpoint = await plugin.start("127.0.0.1:0")
    print(endpoint.url, flush=True)
    try:
        await asyncio.to_thread(sys.stdin.buffer.read)
    finally:
        await endpoint.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
