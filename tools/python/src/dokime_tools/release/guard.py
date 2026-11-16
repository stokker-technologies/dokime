# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Require Exact-Commit Release Proof Before Publication."""

# === Imports === #

# Standard Library Imports
import os

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ..ci import green

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def check() -> None:
    """Require Exact-Commit CI Before Tagging Or Publishing A Stable Release."""
    if not os.environ.get("RELEASE_TARGET_SHA"):
        return
    token = os.environ.get("GH_TOKEN")
    if os.environ.get("CI_READ_TOKEN"):
        os.environ["GH_TOKEN"] = os.environ["CI_READ_TOKEN"]
    try:
        green.main()
    finally:
        if token is None:
            os.environ.pop("GH_TOKEN", None)
        else:
            os.environ["GH_TOKEN"] = token
    from ..release import run

    if run.main(["identity"]):
        raise RuntimeError("Verify Publishing Identity: trusted publishing identity failed")


# === Private Functions === #

# === Entry Point === #
