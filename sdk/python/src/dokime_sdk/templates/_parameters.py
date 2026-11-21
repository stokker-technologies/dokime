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
from .._generated import shared
from .._values import Scalar, encode

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Parameter:
    """Compose an Owned Parameter and Pass build() to a Template."""

    def __init__(self, identifier: str, label: str, kind: str) -> None:
        self.message = shared.Parameter(metadata=shared.Metadata(identifier=identifier, label=label), type=kind)

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
    def text(cls, identifier: str, label: str) -> "Parameter":
        return cls(identifier, label, "VALUE_TYPE_TEXT")

    @classmethod
    def list(cls, identifier: str, label: str, element: type[bool] | type[int] | type[float] | type[str]) -> "Parameter":
        kinds: dict[type, shared.ValueType] = {
            bool: shared.VALUE_TYPE_BOOLEAN,
            int: shared.VALUE_TYPE_INTEGER,
            float: shared.VALUE_TYPE_NUMBER,
            str: shared.VALUE_TYPE_TEXT,
        }
        if element not in kinds:
            raise TypeError("Declare List Parameter: Supply bool, int, float or str as the Element Type")
        result = cls(identifier, label, "VALUE_TYPE_LIST")
        result.message.element_type = kinds[element]
        return result

    @classmethod
    def choice(cls, identifier: str, label: str, values: Iterable[bool | int | float | str]) -> "Parameter":
        choices = tuple(values)
        if not choices or any(type(value) is not type(choices[0]) for value in choices):
            raise ValueError("Declare Choice Parameter: Supply Nonempty Choices of One Scalar Type")
        kinds = {bool: "VALUE_TYPE_BOOLEAN", int: "VALUE_TYPE_INTEGER", float: "VALUE_TYPE_NUMBER", str: "VALUE_TYPE_TEXT"}
        kind = kinds.get(type(choices[0]))
        if kind is None:
            raise TypeError("Declare Choice Parameter: Supply bool, int, float or str Choices")
        result = cls(identifier, label, kind)
        result.message.choices.extend(encode(value) for value in choices)
        return result

    @classmethod
    def sweep(cls, identifier: str, label: str, values: Iterable[float]) -> "Parameter":
        result = cls.number(identifier, label)
        result.message.expression.array.SetInParent()
        result.message.expression.array.values.extend(shared.ParameterExpression(literal=encode(float(value))) for value in values)
        result.message.expansion.cartesian = True
        return result

    @classmethod
    def numbers(cls, identifier: str, label: str, values: Iterable[float]) -> "Parameter":
        result = cls(identifier, label, "VALUE_TYPE_LIST")
        result.message.element_type = shared.VALUE_TYPE_NUMBER
        result.default([float(value) for value in values])
        return result

    def unit(self, unit: str) -> Self:
        self.message.unit = unit
        return self

    def required(self, required: bool = True) -> Self:
        self.message.required = required
        return self

    def default(self, value: Scalar) -> Self:
        if self.message.type == shared.ValueType.Value("VALUE_TYPE_UNSIGNED_INTEGER"):
            if not isinstance(value, int) or isinstance(value, bool) or value < 0:
                raise ValueError("Set Count Default: Choose an Unsigned Integer")
            literal = shared.Value(unsigned_integer=value)
        elif self.message.type == shared.VALUE_TYPE_NUMBER:
            if not isinstance(value, (int, float)) or isinstance(value, bool):
                raise ValueError("Set Number Default: Choose a Number")
            literal = shared.Value(number=float(value))
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

    def build(self) -> shared.Parameter:
        value = self.message
        if value.HasField("min") and value.HasField("max") and value.min > value.max:
            raise ValueError("Build Parameter: Set min No Greater Than max")
        if value.HasField("min_items") and value.HasField("max_items") and value.min_items > value.max_items:
            raise ValueError("Build Parameter: Set min_items No Greater Than max_items")
        if value.type != shared.VALUE_TYPE_LIST and (value.element_type or value.HasField("min_items") or value.HasField("max_items")):
            raise ValueError("Build Parameter: Use List Kind Before Setting Element Type or Item Bounds")
        if value.type == shared.VALUE_TYPE_LIST and not value.element_type:
            raise ValueError("Build Parameter: Declare the List Element Type")
        if value.expression.WhichOneof("kind") == "literal":
            expected = {
                shared.VALUE_TYPE_BOOLEAN: "boolean",
                shared.VALUE_TYPE_INTEGER: "integer",
                shared.VALUE_TYPE_UNSIGNED_INTEGER: "unsigned_integer",
                shared.VALUE_TYPE_NUMBER: "number",
                shared.VALUE_TYPE_TEXT: "text",
                shared.VALUE_TYPE_BYTES: "data",
                shared.VALUE_TYPE_DATETIME: "datetime",
                shared.VALUE_TYPE_LIST: "list",
            }
            if value.expression.literal.WhichOneof("kind") != expected.get(value.type):
                raise ValueError("Build Parameter: Supply a Default Matching the Parameter Kind")
            if value.choices and value.expression.literal not in value.choices:
                raise ValueError("Build Parameter: Choose a Default From the Declared Choices")
            if value.type == shared.VALUE_TYPE_LIST:
                for item in value.expression.literal.list.values:
                    if item.WhichOneof("kind") != expected.get(value.element_type):
                        raise ValueError("Build Parameter: Supply List Values Matching the Element Type")
        result = shared.Parameter()
        result.CopyFrom(self.message)
        return result


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
