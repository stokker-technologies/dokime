# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Run the Nightly Report Workflow Operation."

# === Imports === #

# Standard Library Imports
import os
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def run() -> None:
    from ..release import nightly

    published = nightly.complete(policy.root())
    with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as output:
        output.write(f"published={str(published).lower()}\n")


# === Private Functions === #

# === Entry Point === #
