# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Structural Views Of Native Query Metadata."""

# === Imports === #

# Standard Library Imports
from collections.abc import Sequence
from typing import Protocol

# Third Party Imports
from google.protobuf.message import Message

# Local Imports

# Type Checking Imports

# === Public Exports === #

__all__ = ["Field", "PageLimits", "Schema", "Snapshot", "Timestamp"]

# === Constants === #

# === Types === #

# === Classes === #


class Timestamp(Protocol):
    seconds: int
    nanos: int


class Snapshot(Protocol):
    token: str
    identity: str
    expires_at: Timestamp

    def HasField(self, field_name: str) -> bool: ...


class Field(Protocol):
    id: str
    value_type: Message
    nullable: bool
    unit: str

    def HasField(self, field_name: str) -> bool: ...


class Schema(Protocol):
    identity: str
    versions: Sequence[int]
    fields: Sequence[Field]


class PageLimits(Protocol):
    rows: int
    bytes: int


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
