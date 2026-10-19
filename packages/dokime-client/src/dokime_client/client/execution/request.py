# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Private execution identity and timeout validation."""

# === Imports === #

# Standard Library Imports
import math
from uuid import UUID

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


# === Public Functions === #

# === Private Functions === #


def _interval(value: float) -> None:
    if not math.isfinite(value) or value <= 0:
        raise ValueError("interval/timeout must be positive and finite")


def _identity(value: str) -> None:
    try:
        identity = UUID(value)
    except (ValueError, AttributeError) as error:
        raise ValueError("a registered UUID is required") from error
    if str(identity) != value:
        raise ValueError("a canonical registered UUID is required")


# === Entry Point === #
