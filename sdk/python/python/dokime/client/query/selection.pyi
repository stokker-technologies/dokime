# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from datetime import datetime
from typing import Any, Literal, overload

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from ..history import Checkpoint
from .models import Snapshot

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

type Time = str | datetime
type Window = tuple[Time | None, Time | None]
type Scalar = bool | int | float | str | datetime
type Where = Mapping[str, Scalar] | Sequence[Predicate]
type Operator = Literal["eq", "lt", "lte", "gt", "gte", "in", "exists"]
type At = Message | Snapshot | Checkpoint | None

# === Classes === #

@dataclass(frozen=True, slots=True)
class Predicate:
    field: str
    operator: Operator
    value: Scalar | Sequence[Scalar]

    def __post_init__(self) -> None: ...
    def to_wire(self) -> dict[str, Any]: ...

def predicates(where: Where | None) -> tuple[Predicate, ...]: ...
def timestamp(value: Time) -> dict[str, int]: ...
def window(between: Window | None) -> dict[str, Any] | None: ...
def snapshot_ref(at: At = None) -> dict[str, Any] | None: ...
@overload
def positive(value: int, name: str, maximum: int = ...) -> int: ...
@overload
def positive(value: None, name: str, maximum: int = ...) -> None: ...
def _scalar(value: object, field: str) -> dict[str, Any]: ...
