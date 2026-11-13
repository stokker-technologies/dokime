# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Serve the Shared Thermal Suite Until Explicit Stdin Close."""

# === Imports === #

# Standard Library Imports
# SPDX-License-Identifier: Apache-2.0
import asyncio
import sys

# Third Party Imports
from suite import plugin

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def main():
    endpoint = await plugin().start("127.0.0.1:0")
    print(endpoint.url, flush=True)
    try:
        await asyncio.to_thread(sys.stdin.buffer.read)
    finally:
        await endpoint.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
