# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Metadata wrappers backed by Rust contract builders."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from dataclasses import dataclass, field
from datetime import datetime

# Third Party Imports

# Local Imports
from dokime import _native

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class Identifier:
    value: str

    def to_contract(self) -> str:
        return _native.identifier_contract(self.value)

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True)
class Metadata:
    identifier: Identifier | str
    label: str
    description: str | None = None
    tags: tuple[str, ...] | list[str] = field(default_factory=tuple)

    attributes: dict[str, object] = field(default_factory=dict)
    created_at: datetime | None = None
    updated_at: datetime | None = None

    def to_contract(self) -> dict[str, object]:
        identifier = (
            self.identifier.to_contract()
            if isinstance(self.identifier, Identifier)
            else Identifier(str(self.identifier)).to_contract()
        )
        return _native.metadata_contract(
            identifier,
            self.label,
            self.description,
            list(self.tags),
            self.attributes,
            self.created_at,
            self.updated_at,
        )


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
