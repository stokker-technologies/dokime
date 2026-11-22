# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Regenerate the Embedded Report Before Tauri Rebuilds the Development Runtime."""

# === Imports === #

# Standard Library Imports
import subprocess
import sys

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import frontend
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main(arguments: list[str]) -> int:
    # Generate the Report Before Cargo Checks the Runtime's Embedded Input
    try:
        frontend.build_report([])
    except subprocess.CalledProcessError as error:
        # Tauri Keeps Watching After a Compilation Failure, Including Report Failures.
        print(
            f"error: could not compile the development report; fix the renderer build errors and save its sources: {error}", file=sys.stderr
        )
        return 101

    # Rebuild and Run With Tauri's Arguments and the Inherited Compiler Environment
    return subprocess.run(
        ["cargo", "run", *arguments],
        check=False,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    ).returncode


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
