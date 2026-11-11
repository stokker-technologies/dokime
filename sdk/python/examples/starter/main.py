# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Start a Backend and Pair the Desktop"""

# === Imports === #

# Standard Library Imports
import asyncio
from pathlib import Path

# Third Party Imports
import dokime

# Local Imports
from plugin import create

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


# === Public Functions === #


async def main() -> None:
    # Compose the Plugin
    plugin = create()

    # Start the Backend
    async with dokime.Backend(port=0, application_root=Path(".dokime"), plugins=[plugin]) as backend:
        # Pair the Desktop
        backend.export_pairing("bench.dokime-remote", label="Bench")
        print(f"Backend: {backend.endpoint}; open bench.dokime-remote in the desktop", flush=True)

        # Serve Until Cancelled
        await asyncio.Event().wait()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
