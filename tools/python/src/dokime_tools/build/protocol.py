# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate or Check Protocol Bindings and Composed SDK Roots"""

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import cargo, resource_roots
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    options = parser.parse_args()
    if options.check:
        subprocess.run(
            ["mise", "run", "test", "--", "--package", "dokime-protocol", "--target", "integration"],
            check=True,
            creationflags=spawn_flags(),
        )
    else:
        status = cargo.focused(Path.cwd(), "run", ["--locked", "--package", "dokime-protocol-generator"])
        if status:
            raise SystemExit(status)
    # C# Wire Generation Runs the Grpc.Tools Compiler That Preparing the Staged Harmos Project Restores.
    subprocess.run(["mise", "-C", "sdk/csharp", "run", "prepare:harmos"], check=True, creationflags=spawn_flags())
    resource_roots.generate(options.check)
    if not options.check:
        subprocess.run(["mise", "-C", "sdk/python", "run", "generate"], check=True, creationflags=spawn_flags())
