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
import argparse
import asyncio

# Third Party Imports
# Local Imports
from dokime.connection import connect

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("endpoint")
    options = parser.parse_args()
    client = await connect(options.endpoint)
    try:
        status = await client.status.get()
        print(status.status.identity.host_id)
        entries = await client.plugins.list()
        for entry in entries.entries:
            print(entry.name)
        suites = await client.suites.list()
        for suite in suites.suites:
            print(suite.metadata.label)
    finally:
        await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
