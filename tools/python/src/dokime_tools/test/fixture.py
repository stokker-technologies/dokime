# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Hold A Hub Temporary Directory For Native Fixture Consumers"""

# === Imports === #

# Standard Library Imports
import sys

# Third Party Imports
from harmos_tools import temporary

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

def main() -> None:
    """Keep The Canonical Owner Alive Until Its Native Consumer Closes Stdin."""
    with temporary.directory() as directory:
        print(directory, flush=True)
        sys.stdin.buffer.read()

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
