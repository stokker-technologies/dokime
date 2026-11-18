# ***
#
# Starter Project
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""The Capture Suite, One Module per Template."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from dokime_sdk import Suite

# Local Imports
from starter_project.plugin.suites.capture.confirmation import ConfirmationTestTemplate

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CaptureSuite(Suite):
    templates = [ConfirmationTestTemplate]


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
