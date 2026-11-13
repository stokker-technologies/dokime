# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prove the SDK Product Dependency Boundary"""

# === Imports === #

# Standard Library Imports
import subprocess
import sys

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    command = [
        "cargo",
        "tree",
        "--locked",
        "--manifest-path",
        "tests/consumer/Cargo.toml",
        "--package",
        "dokime",
        "--edges",
        "normal,build",
        "--prefix",
        "none",
    ]
    result = subprocess.run(command, check=True, capture_output=True, text=True)
    print(result.stdout, end="")
    products = {
        line.split()[0]
        for line in result.stdout.splitlines()
        if line.startswith(("dokime", "harmos"))
    }
    expected = {
        "dokime",
        "dokime-derive",
        "dokime-protocol",
        "harmos",
        "harmos-protocol",
        "harmos-sdk-derive",
    }
    unexpected = products - expected
    if unexpected:
        print(
            "SDK Compiles Host Product Packages: " + ", ".join(sorted(unexpected)),
            file=sys.stderr,
        )
        raise SystemExit(1)
    print("SDK Product Boundary: " + ", ".join(sorted(products)))


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
