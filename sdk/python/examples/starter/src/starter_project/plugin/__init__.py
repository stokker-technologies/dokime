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
from datetime import timedelta

# Third Party Imports
from dokime_sdk import Plugin
from dokime_sdk.operations import Setting, Settings

# Local Imports
from starter_project.plugin.operations.echo import echo
from starter_project.plugin.suites.capture import CaptureSuite
from starter_project.plugin.suites.capture.confirmation import CONFIRMATION, Temperature

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ConnectionSettings(Settings):
    api_address: str = Setting.text("Remote API Address", default="127.0.0.1:30059")
    dll_timeout: timedelta = Setting.duration("DLL Host Timeout", default=timedelta(seconds=5), max=timedelta(seconds=30))


class ExamplePlugin(Plugin):
    suites = [CaptureSuite]
    operations = [echo]
    settings = (ConnectionSettings,)
    records = (CONFIRMATION,)
    observers = (Temperature,)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
