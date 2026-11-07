# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports

from dataclasses import dataclass
from typing import Any, Literal

# Third Party Imports
from google.protobuf.message import Message

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

type Member = Literal["Count", "Sum", "Mean", "Median", "Min", "Max", "Rms", "Std", "Percentile"]

# === Classes === #

@dataclass(frozen=True, slots=True)
class Aggregate:
    member: Member
    rank: int | None = None

    def __post_init__(self) -> None: ...
    @classmethod
    def count(cls) -> Aggregate: ...
    @classmethod
    def sum(cls) -> Aggregate: ...
    @classmethod
    def mean(cls) -> Aggregate: ...
    @classmethod
    def median(cls) -> Aggregate: ...
    @classmethod
    def min(cls) -> Aggregate: ...
    @classmethod
    def max(cls) -> Aggregate: ...
    @classmethod
    def rms(cls) -> Aggregate: ...
    @classmethod
    def std(cls) -> Aggregate: ...
    @classmethod
    def percentile(cls, rank: int) -> Aggregate: ...
    @classmethod
    def from_wire(cls, message: Message) -> Aggregate: ...
    def to_wire(self) -> dict[str, dict[str, Any]]: ...
