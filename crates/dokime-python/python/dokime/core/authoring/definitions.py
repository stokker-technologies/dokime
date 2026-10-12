# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Decorator-produced definition types for authored templates."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from dataclasses import dataclass

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class StepDefinition:
    step_id: str
    method_name: str
    label: str | None = None
    description: str | None = None
    order: int = 0
    repeatable: bool = False


@dataclass(frozen=True)
class SamplerDefinition:
    method_name: str
    row_type: type
    interval_ms: int

    @property
    def stream(self) -> str:
        return self.row_type.__dokime_stream__.stream


@dataclass(frozen=True)
class ValidationCriterionDefinition:
    criterion_id: str
    method_name: str
    label: str | None = None
    order: int = 0


@dataclass(frozen=True)
class InitializeDefinition:
    initialize_id: str
    method_name: str
    label: str | None = None
    order: int = 0


@dataclass(frozen=True)
class CleanupDefinition:
    cleanup_id: str
    method_name: str
    label: str | None = None
    order: int = 0


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
