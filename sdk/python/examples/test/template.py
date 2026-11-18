# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Deterministic Suite Shared With The Cross-Language Host Story."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
# Local Imports
from suite import Capture

from dokime_sdk import Plugin, Template
from dokime_sdk.authoring import Suite

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def plugin() -> Plugin:
    template = Template(Capture, label="Capture")
    return Plugin("example", "0.1.0", suites=[Suite("example", [template], "Example")])


# === Private Functions === #

# === Entry Point === #
