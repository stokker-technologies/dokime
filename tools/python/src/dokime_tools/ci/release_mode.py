# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Run the Release Mode Workflow Operation."

# === Imports === #

# Standard Library Imports
import os
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


def run() -> None:
    from .review import github_release_mode

    result = github_release_mode()
    with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as output:
        output.write("".join(f"{key}={value}\n" for key, value in result.items()))


# === Private Functions === #

# === Entry Point === #
