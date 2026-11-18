# ***
#
# Starter Project
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""The Example Plugin: Its Suites and Operations."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from dokime_sdk import Plugin

# Local Imports
from starter_project.plugin.operations.echo import echo
from starter_project.plugin.suites.capture import CaptureSuite

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ExamplePlugin(Plugin):
    suites = [CaptureSuite]
    operations = [echo]


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
