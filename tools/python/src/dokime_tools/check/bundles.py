# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Reject tracked plugin bundle build outputs"""

# === Imports === #

# Standard Library Imports
import subprocess

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> int:
    listing = subprocess.run(
        ["git", "ls-files", "--", "*.dokime"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    if listing.returncode != 0:
        listing = subprocess.run(
            ["jj", "file", "list", "glob:**/*.dokime"],
            capture_output=True,
            text=True,
            check=True,
            encoding="utf-8",
        )

    tracked = listing.stdout.splitlines()
    if tracked:
        raise SystemExit(
            "Tracked plugin bundles must be removed from history: " + ", ".join(tracked)
        )
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
