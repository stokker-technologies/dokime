# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Synchronize the repository Rust toolchain"""

# === Imports === #

# Standard Library Imports
import os
import subprocess
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    # Let rustup read rust-toolchain.toml instead of mise's override.
    environment = os.environ.copy()
    environment.pop("RUSTUP_TOOLCHAIN", None)

    subprocess.run(
        ["rustup", "toolchain", "install", "--no-self-update", "--no-update"],
        cwd=Path(__file__).resolve().parents[5],
        env=environment,
        check=True,
    )


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
