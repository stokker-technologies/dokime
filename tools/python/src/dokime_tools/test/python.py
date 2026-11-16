# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run Python Client Lanes With The Independent Python Product Wheel"""

# === Imports === #

# Standard Library Imports
import argparse
from pathlib import Path
from tempfile import TemporaryDirectory

# Third Party Imports
# Local Imports
from dokime_tools.build import wheel
from dokime_tools.test import policy, run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    options, arguments = parser.parse_known_args()
    # A Focused Lane Tests the Current Source: Build the Wheel the Lane Installs.
    wheel.build(policy.root(), options.profile)
    # Each Invocation Owns Its Fixtures; Pytest Must Not Clean Another Run's Links.
    with TemporaryDirectory(prefix="dokime-pytest-") as scratch:
        if "--" not in arguments:
            arguments.append("--")
        arguments.append(f"--basetemp={Path(scratch) / 'fixtures'}")
        return run.main(["test", "--profile", options.profile, "--language", "python", "--project", "sdk/python", *arguments])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
