# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Run the Bindings Workflow Operation."

# === Imports === #

# Standard Library Imports
import argparse
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def run(options: argparse.Namespace) -> int:
    from ..test.archive import install_wheels
    from . import tasks

    install_wheels(Path.cwd(), options.profile)
    return tasks.main(["check", "--bindings", "--profile", options.profile])


# === Private Functions === #

# === Entry Point === #
