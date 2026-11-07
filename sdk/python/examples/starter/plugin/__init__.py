# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compose the Embedded Bench Plugin"""

# === Imports === #

# Standard Library Imports
# Third Party Imports
import dokime

# Local Imports
from .operations.read import Read
from .suites.capture import Confirm, Row, Suite

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


# === Public Functions === #


def create() -> dokime.Plugin:
    return dokime.Plugin("bench", "0.1.0", catalog=[Read, Row, Confirm, Suite])


# === Private Functions === #

# === Entry Point === #
