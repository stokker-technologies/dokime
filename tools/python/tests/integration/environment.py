# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Isolate Environment Inputs for Mocked Tooling Stories."

# === Imports === #

# Standard Library Imports
import os
import unittest
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Resource:
    """A Native Resource Double with No External Handles."""

    def __init__(self, environment: dict[str, str] | None = None) -> None:
        self.environment = dict(os.environ if environment is None else environment)

    def close(self) -> None:
        return


# === Public Functions === #


def isolate_environment() -> None:
    """Keep Native Temporary Storage and Restore the Caller at Module Cleanup."""
    environment = {}
    for environment_key, environment_value in os.environ.items():
        if environment_key not in {"TEMP", "TMP", "TMPDIR"}:
            continue
        environment[environment_key] = environment_value

    unittest.enterModuleContext(patch.dict(os.environ, environment, clear=True))


# === Private Functions === #

# === Entry Point === #
