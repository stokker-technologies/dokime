# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import Sequence
from dataclasses import dataclass
from typing import Any

# Third Party Imports
# Local Imports
from ..aggregate import Aggregate
from .models import Schema, Snapshot

# Type Checking Imports

# === Public Exports === #

# === Constants === #

contracts: Any

# === Types === #

# === Classes === #

@dataclass(frozen=True, slots=True)
class Statistic:
    name: str
    field: str
    aggregate: Aggregate
    value: int | float | None

@dataclass(frozen=True, slots=True)
class QualityCounts:
    field: str
    finite: int
    missing: int
    null: int
    non_finite: int

@dataclass(frozen=True, slots=True)
class Statistics:
    schema: Schema
    values: tuple[Statistic, ...]
    selected_rows: int
    quality_counts: tuple[QualityCounts, ...]
    snapshot: Snapshot
    complete: bool

# === Public Functions === #

def outputs(field: str, aggregates: Sequence[Aggregate]) -> list[dict[str, Any]]: ...
def decode(response: Any, requested: list[dict[str, Any]]) -> Statistics: ...
def statistic_value(value: Any) -> int | float: ...
