# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Name Harmos Aggregates Through Their Generated Enum Message"""

# === Imports === #

# Standard Library Imports
from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Literal

# Third Party Imports
from google.protobuf.message import Message

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

_VARIANT = "_harmos_variant"
_MAX_RANK = 100

# === Types === #

type Member = Literal[
    "Count", "Sum", "Mean", "Median", "Min", "Max", "Rms", "Std", "Percentile"
]

# === Classes === #


@dataclass(frozen=True, slots=True)
class Aggregate:
    """One member of the Harmos aggregate vocabulary.

    Every operation's generated code declares its own aggregate message, so the
    helper converts through the protobuf mapping form instead of one class:
    `to_wire()` initializes any generated aggregate field and `from_wire()`
    reads one back through its `_harmos_variant` oneof.
    """

    member: Member
    rank: int | None = None

    def __post_init__(self) -> None:
        if (self.member == "Percentile") != (self.rank is not None):
            raise ValueError("only a percentile carries a rank")
        if self.rank is not None and not 0 <= self.rank <= _MAX_RANK:
            raise ValueError(f"percentile rank {self.rank} is outside 0 to {_MAX_RANK}")

    @classmethod
    def count(cls) -> Aggregate:
        return cls("Count")

    @classmethod
    def sum(cls) -> Aggregate:
        return cls("Sum")

    @classmethod
    def mean(cls) -> Aggregate:
        return cls("Mean")

    @classmethod
    def median(cls) -> Aggregate:
        return cls("Median")

    @classmethod
    def min(cls) -> Aggregate:
        return cls("Min")

    @classmethod
    def max(cls) -> Aggregate:
        return cls("Max")

    @classmethod
    def rms(cls) -> Aggregate:
        return cls("Rms")

    @classmethod
    def std(cls) -> Aggregate:
        """The population standard deviation."""
        return cls("Std")

    @classmethod
    def percentile(cls, rank: int) -> Aggregate:
        """A whole percentage from 0 to 100, interpolated between ranks."""
        return cls("Percentile", rank)

    @classmethod
    def from_wire(cls, message: Message) -> Aggregate:
        """Read a generated aggregate message; an absent member is `Count`."""
        member = message.WhichOneof(_VARIANT)
        if member is None:
            return cls.count()
        if member == "Percentile":
            return cls.percentile(getattr(message, member).rank)
        return cls(member)

    def to_wire(self) -> dict[str, dict[str, Any]]:
        """The protobuf mapping form a generated aggregate field accepts."""
        if self.rank is None:
            return {self.member: {}}
        return {self.member: {"rank": self.rank}}

    def __str__(self) -> str:
        if self.rank is None:
            return self.member.lower()
        return f"p{self.rank}"


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
