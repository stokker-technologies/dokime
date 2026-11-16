# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Hold a Hub Temporary Directory for Native Fixture Consumers"

# === Imports === #

# Standard Library Imports
import sys

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import temporary

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    """Keep the Canonical Owner Alive Until Its Native Consumer Closes Stdin."""
    directory_directory = temporary.Directory()
    directory = directory_directory.path
    try:
        print(directory, flush=True)
        sys.stdin.buffer.read()
    finally:
        directory_directory.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
