# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Owned Parameter Builder Values"""

# === Imports === #

# Standard Library Imports
from collections.abc import Iterable
from typing import Self

# Third Party Imports
# Local Imports
import dokime_sdk

from .values import Scalar, encode

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Parameter:
    """Compose an Owned Parameter and Pass build() to a Template."""

    def __init__(self, identifier: str, label: str, kind: str) -> None:
        self.message = dokime_sdk.Parameter(metadata=dokime_sdk.Metadata(identifier=identifier, label=label), type=kind)

    @classmethod
    def number(cls, identifier: str, label: str) -> "Parameter":
        return cls(identifier, label, "VALUE_TYPE_NUMBER")

    @classmethod
    def count(cls, identifier: str, label: str) -> "Parameter":
        return cls(identifier, label, "VALUE_TYPE_UNSIGNED_INTEGER")

    @classmethod
    def flag(cls, identifier: str, label: str) -> "Parameter":
        return cls(identifier, label, "VALUE_TYPE_BOOLEAN")

    @classmethod
    def sweep(cls, identifier: str, label: str, values: Iterable[float]) -> "Parameter":
        result = cls.number(identifier, label)
        result.message.expression.array.SetInParent()
        result.message.expression.array.values.extend(dokime_sdk.ParameterExpression(literal=encode(float(value))) for value in values)
        result.message.expansion.cartesian = True
        return result

    @classmethod
    def numbers(cls, identifier: str, label: str, values: Iterable[float]) -> "Parameter":
        result = cls(identifier, label, "VALUE_TYPE_LIST")
        result.message.element_type = dokime_sdk.VALUE_TYPE_NUMBER
        result.default([float(value) for value in values])
        return result

    def unit(self, unit: str) -> Self:
        self.message.unit = unit
        return self

    def default(self, value: Scalar) -> Self:
        if self.message.type == dokime_sdk.ValueType.Value("VALUE_TYPE_UNSIGNED_INTEGER"):
            if not isinstance(value, int) or isinstance(value, bool) or value < 0:
                raise ValueError("Set Count Default: Choose an Unsigned Integer")
            literal = dokime_sdk.Value(unsigned_integer=value)
        elif self.message.type == dokime_sdk.VALUE_TYPE_NUMBER:
            if not isinstance(value, (int, float)) or isinstance(value, bool):
                raise ValueError("Set Number Default: Choose a Number")
            literal = dokime_sdk.Value(number=float(value))
        else:
            literal = encode(value)
        self.message.expression.literal.CopyFrom(literal)
        return self

    def range(self, minimum: float | None = None, maximum: float | None = None) -> Self:
        self.message.ClearField("min")
        self.message.ClearField("max")
        if minimum is not None:
            self.message.min = minimum
        if maximum is not None:
            self.message.max = maximum
        return self

    def items(self, minimum: int | None = None, maximum: int | None = None) -> Self:
        self.message.ClearField("min_items")
        self.message.ClearField("max_items")
        if minimum is not None:
            self.message.min_items = minimum
        if maximum is not None:
            self.message.max_items = maximum
        return self

    def build(self) -> dokime_sdk.Parameter:
        result = dokime_sdk.Parameter()
        result.CopyFrom(self.message)
        return result


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
