# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Serve A Remote Plugin; Start Dokime Separately."""

# === Imports === #
# Standard Library Imports
import argparse
import asyncio
from typing import Any

# Local Imports
from suite import plugin

# Third Party Imports
import dokime

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def main() -> Any:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--address", default="127.0.0.1:7001")
    options = parser.parse_args()
    endpoint = await dokime.serve(plugin(), address=options.address)
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
