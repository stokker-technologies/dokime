# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Isolate Environment Inputs For Mocked Tooling Stories."""

# === Imports === #

# Standard Library Imports
import os
import unittest
from unittest.mock import patch

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def isolate_environment() -> None:
    """Keep Native Temporary Storage And Restore The Caller At Module Cleanup."""
    environment = {key: value for key, value in os.environ.items() if key in {"TEMP", "TMP", "TMPDIR"}}
    unittest.enterModuleContext(patch.dict(os.environ, environment, clear=True))


# === Private Functions === #

# === Entry Point === #
