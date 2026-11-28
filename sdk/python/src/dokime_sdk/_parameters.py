# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Typed Template Parameter Declarations"""

# === Imports === #

# Standard Library Imports
import builtins
import math
from collections.abc import Sequence
from dataclasses import dataclass
from typing import ClassVar, TypeVar, cast

# Third Party Imports
# Local Imports
from ._generated.proto.model.v1.parameters_pb2 import Parameter as ParameterMessage
from ._generated.proto.model.v1.parameters_pb2 import ParameterExpression
from ._generated.proto.model.v1.scalar_pb2 import (
    VALUE_TYPE_BOOLEAN,
    VALUE_TYPE_INTEGER,
    VALUE_TYPE_LIST,
    VALUE_TYPE_NUMBER,
    VALUE_TYPE_TEXT,
    VALUE_TYPE_UNSIGNED_INTEGER,
    Value,
    ValueType,
)
from ._values import Scalar, encode

# Type Checking Imports

# === Public Exports === #

# === Constants === #

_SCALAR_KINDS: dict[type, ValueType] = {
    bool: VALUE_TYPE_BOOLEAN,
    int: VALUE_TYPE_INTEGER,
    float: VALUE_TYPE_NUMBER,
    str: VALUE_TYPE_TEXT,
}

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class Parameter[T]:
    """A Typed Template Attribute; the Host Validates Its Values Before Creation."""

    __type_params__: ClassVar[tuple[TypeVar, ...]]

    _message: ParameterMessage
    _type: type[T]

    def __set_name__(self, owner: type, name: str) -> None:
        if self._message.metadata.identifier and self._message.metadata.identifier != name:
            raise ValueError("Declare Parameter: Use a Separate Declaration for Each Attribute")
        self._message.metadata.identifier = name

    @classmethod
    def integer(
        cls,
        label: str,
        *,
        default: int | None = None,
        min: int | None = None,
        max: int | None = None,
        choices: Sequence[int] = (),
        required: bool = True,
        unit: str = "",
    ) -> "Parameter[int]":
        return _declare(label, int, default, min, max, choices, required, unit)

    @classmethod
    def unsigned_integer(
        cls,
        label: str,
        *,
        default: int | None = None,
        min: int | None = None,
        max: int | None = None,
        choices: Sequence[int] = (),
        required: bool = True,
        unit: str = "",
    ) -> "Parameter[int]":
        value = _declare(label, int, default, min, max, choices, required, unit)
        value._message.type = VALUE_TYPE_UNSIGNED_INTEGER
        value._message.ClearField("choices")
        for choice in choices:
            value._message.choices.add(unsigned_integer=choice)
        if default is not None:
            value._message.expression.literal.CopyFrom(Value(unsigned_integer=default))
        return value

    @classmethod
    def number(
        cls,
        label: str,
        *,
        default: float | None = None,
        min: float | None = None,
        max: float | None = None,
        choices: Sequence[float] = (),
        required: bool = True,
        unit: str = "",
    ) -> "Parameter[float]":
        return _declare(label, float, default, min, max, choices, required, unit)

    @classmethod
    def text(
        cls,
        label: str,
        *,
        default: str | None = None,
        choices: Sequence[str] = (),
        required: bool = True,
    ) -> "Parameter[str]":
        return _declare(label, str, default, None, None, choices, required, "")

    @classmethod
    def flag(cls, label: str, *, default: bool | None = None, required: bool = True) -> "Parameter[bool]":
        return _declare(label, bool, default, None, None, (), required, "")

    @classmethod
    def list[E: bool | int | float | str](
        cls,
        label: str,
        element: type[E],
        *,
        default: builtins.list[E] | None = None,
        min_items: int | None = None,
        max_items: int | None = None,
        required: bool = True,
    ) -> "Parameter[builtins.list[E]]":
        if element not in _SCALAR_KINDS:
            raise TypeError("Declare List Parameter: Supply bool, int, float or str as the Element Type")
        if min_items is not None and min_items < 0 or max_items is not None and max_items < 0:
            raise ValueError("Declare List Parameter: Supply Nonnegative Item Bounds")
        if min_items is not None and max_items is not None and min_items > max_items:
            raise ValueError("Declare List Parameter: Set min_items No Greater Than max_items")
        message = ParameterMessage(
            metadata={"label": label},
            type=VALUE_TYPE_LIST,
            element_type=_SCALAR_KINDS[element],
            required=required,
            min_items=min_items,
            max_items=max_items,
        )
        if default is not None:
            if any(type(value) is not element for value in default):
                raise TypeError("Declare List Parameter: Supply Default Values Matching the Element Type")
            message.expression.literal.CopyFrom(encode(tuple(default)))
        return Parameter(message, cast(type[builtins.list[E]], builtins.list))

    @classmethod
    def sweep(cls, label: str, values: Sequence[float], *, unit: str = "") -> "Parameter[float]":
        result = cls.number(label, unit=unit)
        result._message.expression.array.values.extend(ParameterExpression(literal=encode(float(value))) for value in values)
        result._message.expansion.cartesian = True
        return result

    def _definition(self) -> ParameterMessage:
        if not self._message.metadata.identifier:
            raise ValueError("Declare Parameter: Bind the Declaration to a Template Attribute")
        message = ParameterMessage()
        message.CopyFrom(self._message)
        return message

    def _read(self, value: Scalar) -> T:
        if not isinstance(value, self._type) or isinstance(value, bool) and self._type is int:
            raise TypeError(f"Read Parameter: Supply a Host-Resolved {self._type.__name__} for {self._message.metadata.label}")
        return cast(T, value)


# === Public Functions === #

# === Private Functions === #


def _declare[T: bool | int | float | str](
    label: str,
    kind: type[T],
    default: T | None,
    minimum: float | None,
    maximum: float | None,
    choices: Sequence[T],
    required: bool,
    unit: str,
) -> Parameter[T]:
    for value in (default, *choices):
        if value is not None and type(value) is not kind:
            raise TypeError(f"Declare Parameter: Supply {kind.__name__} Values for {label}")
    for bound in (minimum, maximum):
        if bound is not None and not math.isfinite(bound):
            raise ValueError("Declare Parameter: Supply Finite Bounds")
    if minimum is not None and maximum is not None and minimum > maximum:
        raise ValueError("Declare Parameter: Set min No Greater Than max")
    if default is not None and choices and default not in choices:
        raise ValueError("Declare Parameter: Choose a Default From the Declared Choices")
    message = ParameterMessage(
        metadata={"label": label},
        type=_SCALAR_KINDS[kind],
        required=required,
        unit=unit,
        min=minimum,
        max=maximum,
        choices=[encode(value) for value in choices],
    )
    if default is not None:
        message.expression.literal.CopyFrom(encode(default))
    return Parameter(message, kind)


# === Entry Point === #
