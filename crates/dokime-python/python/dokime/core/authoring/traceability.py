# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Canonical traceability facts supplied before an execution's first step."""

# === Imports === #

# Standard Library Imports
from dataclasses import dataclass, field

# Third Party Imports

# Local Imports
from dokime.client.models import Value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class TraceabilityRow:
    """One fact; the backend assigns its execution scope and provenance."""

    identifier: str
    label: str
    value: Value
    unit: str | None = None
    source: str | None = None


@dataclass(frozen=True)
class TraceabilitySection:
    """An ordered group of facts, matching Rust's TraceabilitySection."""

    identifier: str
    label: str
    rows: list[TraceabilityRow] = field(default_factory=list)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
