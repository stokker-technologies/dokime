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
import argparse
import asyncio

# Third Party Imports
# Local Imports
from operations import plugin

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


# === Public Functions === #


async def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--address", default="127.0.0.1:0")
    options = parser.parse_args()
    endpoint = await plugin.start(options.address)
    print(endpoint.url, flush=True)
    try:
        await asyncio.Event().wait()
    finally:
        await endpoint.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
