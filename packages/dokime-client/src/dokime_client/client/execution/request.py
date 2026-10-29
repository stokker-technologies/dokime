# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Private execution identity and timeout validation."""

import math
from uuid import UUID


def _interval(value: float) -> None:
    try:
        valid = type(value) in (int, float) and math.isfinite(value) and value > 0
    except OverflowError:
        valid = False
    if not valid:
        raise ValueError("interval/timeout must be positive and finite")


def _identity(value: str) -> None:
    try:
        identity = UUID(value)
    except (ValueError, AttributeError) as error:
        raise ValueError("a registered UUID is required") from error
    if str(identity) != value:
        raise ValueError("a canonical registered UUID is required")
