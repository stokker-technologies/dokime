# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Parameter, credential, and file declarations for authored templates."""

from __future__ import annotations

# === Imports === #

# Standard Library Imports

import inspect
import re
from collections.abc import Callable, Iterable, Mapping, Sequence
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Any, Self, cast, get_args, get_origin, overload

# Third Party Imports

# Local Imports

from dokime.client.models import DatetimeValue, UnsignedValue

from dokime import _native
from dokime.core.declarations import JsonObject
from dokime.core.declarations import value_type_name

# Type Checking Imports

# === Public Exports === #

# === Constants === #

DEFAULT_UNSET = object()


# === Types === #

type ParameterValidator = Callable[[Any], bool]


# === Classes === #


class TestCaseMaterializationError(ValueError):
    """Raised when template parameters cannot materialize concrete test cases."""


@dataclass(frozen=True)
class AuthoredParameterDefinition:
    name: str
    values: list[Any]
    label: str | None = None
    description: str | None = None
    unit: str | None = None
    value_type: str | None = None
    choices: list[Any] = field(default_factory=list)
    expansion: str | dict[str, str] = "single"
    metadata: JsonObject = field(default_factory=dict)
    expression: ParameterExpression | None = None
    min: float | None = None
    max: float | None = None
    element_type: str | None = None
    min_items: int | None = None
    max_items: int | None = None

    def with_name(self, name: str) -> Self:
        return type(self)(
            name=name,
            values=list(self.values),
            label=self.label,
            description=self.description,
            unit=self.unit,
            value_type=self.value_type,
            choices=list(self.choices),
            expansion=self.expansion,
            metadata=dict(self.metadata),
            expression=self.expression,
            min=self.min,
            max=self.max,
            element_type=self.element_type,
            min_items=self.min_items,
            max_items=self.max_items,
        )


@dataclass(frozen=True)
class CredentialDefinition:
    credential_id: str
    label: str
    description: str | None = None
    required: bool = True
    scopes: list[str] = field(default_factory=list)

    def to_contract(self) -> JsonObject:
        payload: JsonObject = {
            "id": self.credential_id,
            "label": self.label,
            "required": self.required,
            "scopes": list(self.scopes),
        }
        if self.description is not None:
            payload["description"] = self.description
        return payload


@dataclass(frozen=True)
class CredentialSchema:
    credentials: list[CredentialDefinition] = field(default_factory=list)

    def to_contract(self) -> JsonObject:
        return {
            "credentials": [credential.to_contract() for credential in self.credentials]
        }


@dataclass(frozen=True)
class ParameterExpression:
    data: JsonObject

    def to_contract(self) -> JsonObject:
        return cast(JsonObject, _contract_value(self.data))

    @classmethod
    def literal(cls, value: object) -> Self:
        return cls({"kind": "literal", "value": _contract_value(value)})

    @classmethod
    def array(cls, values: Iterable[object]) -> Self:
        return cls(
            {
                "kind": "array",
                "values": [_parameter_expression_contract(value) for value in values],
            }
        )

    @classmethod
    def integer_range(cls, start: int, stop: int, step: int = 1) -> Self:
        return cls(
            {
                "kind": "range",
                "start": int(start),
                "stop": int(stop),
                "step": int(step),
            }
        )

    @classmethod
    def arange(cls, start: float, stop: float, step: float = 1.0) -> Self:
        return cls(
            {
                "kind": "arange",
                "start": float(start),
                "stop": float(stop),
                "step": float(step),
            }
        )

    @classmethod
    def linspace(cls, start: float, stop: float, count: int) -> Self:
        return cls(
            {
                "kind": "linspace",
                "start": float(start),
                "stop": float(stop),
                "count": int(count),
            }
        )

    @classmethod
    def reference(cls, parameter: str) -> Self:
        return cls({"kind": "reference", "parameter": parameter})

    @classmethod
    def dynamic_default(
        cls,
        key: str,
        fallback: object = DEFAULT_UNSET,
    ) -> Self:
        data: JsonObject = {"kind": "dynamic_default", "key": key}
        if fallback is not DEFAULT_UNSET:
            data["fallback"] = _parameter_expression_contract(fallback)
        return cls(data)

    @classmethod
    def unary(cls, op: str, value: object) -> Self:
        return cls(
            {"kind": "unary", "op": op, "value": _parameter_expression_contract(value)}
        )

    @classmethod
    def binary(cls, op: str, left: object, right: object) -> Self:
        return cls(
            {
                "kind": "binary",
                "op": op,
                "left": _parameter_expression_contract(left),
                "right": _parameter_expression_contract(right),
            }
        )


@dataclass(frozen=True)
class FileDefinition:
    name: str
    label: str | None = None
    description: str | None = None
    default: str | None = None
    required: bool = False
    media_types: list[str] = field(default_factory=list)
    metadata: JsonObject = field(default_factory=dict)

    def with_name(self, name: str) -> Self:
        return type(self)(
            name=name,
            label=self.label,
            description=self.description,
            default=self.default,
            required=self.required,
            media_types=list(self.media_types),
            metadata=dict(self.metadata),
        )


class ParameterDescriptor:
    """Descriptor used by ``parameter(...)`` declarations on templates."""

    def __init__(
        self,
        values: object,
        *,
        label: str | None = None,
        description: str | None = None,
        unit: str | None = None,
        value_type: object | None = None,
        choices: Sequence[Any] | type[Enum] = (),
        expansion: str | dict[str, str] = "single",
        element_type: object | None = None,
        min_items: int | None = None,
        max_items: int | None = None,
        min: float | None = None,
        max: float | None = None,
        validate: ParameterValidator | None = None,
        validation_message: str | None = None,
        metadata: JsonObject | None = None,
    ) -> None:
        self.name: str | None = None
        self.owner_name: str | None = None
        self._raw_values = values
        self._expression = _parameter_expression(values)
        self._label = label
        self._description = description
        self._unit = unit
        self._value_type = value_type
        self._element_type = _list_element_type(value_type, element_type)
        self._min_items = min_items
        self._max_items = max_items
        self._min = min
        self._max = max
        choice_source = choices
        if self._expression is None and _is_enum_type(values) and not choices:
            choice_source = cast(Sequence[Any] | type[Enum], values)
        self._choices = _parameter_choices(choice_source)
        self._expansion = expansion
        self._validate = validate
        self._validation_message = validation_message
        self._metadata = dict(metadata or {})

    def __set_name__(self, owner: type[object], name: str) -> None:
        self.name = name
        self.owner_name = owner.__name__
        try:
            annotation = inspect.get_annotations(owner, eval_str=True).get(name)
        except (NameError, TypeError):
            annotation = getattr(owner, "__annotations__", {}).get(name)
        if _value_type_is_list(annotation):
            self._element_type = _list_element_type(annotation, self._element_type)

    @property
    def definition(self) -> AuthoredParameterDefinition:
        return self.resolved_definition()

    def resolved_definition(self) -> AuthoredParameterDefinition:
        name = self.name or "pending"
        values = (
            []
            if self._expression is not None
            else _parameter_values(
                self._raw_values,
                expansion=self._expansion,
            )
        )
        return AuthoredParameterDefinition(
            name=name,
            values=values,
            label=self._label,
            description=self._description,
            unit=self._unit,
            value_type="array"
            if self._element_type is not None
            else (
                value_type_name(self._value_type)
                if self._value_type is not None
                else _infer_value_type(values, self._choices, self._expression)
            ),
            choices=list(self._choices),
            expansion=self._expansion,
            metadata=_parameter_metadata(self._metadata, self._expression),
            expression=self._expression,
            min=self._min,
            max=self._max,
            element_type=self._element_type,
            min_items=self._min_items,
            max_items=self._max_items,
        )

    def validate_value(self, value: object) -> None:
        if self._element_type is not None:
            if not isinstance(value, list):
                raise TestCaseMaterializationError(
                    f"Parameter {self._qualified_name()} requires a list."
                )
            if self._min_items is not None and len(value) < self._min_items:
                raise TestCaseMaterializationError(
                    f"Parameter {self._qualified_name()} requires at least {self._min_items} items."
                )
            if self._max_items is not None and len(value) > self._max_items:
                raise TestCaseMaterializationError(
                    f"Parameter {self._qualified_name()} allows at most {self._max_items} items."
                )
        values = (
            value
            if self._element_type is not None and isinstance(value, list)
            else [value]
        )
        for item in values:
            if isinstance(item, (int, float)) and not isinstance(item, bool):
                if (
                    self._min is not None
                    and item < self._min
                    or self._max is not None
                    and item > self._max
                ):
                    raise TestCaseMaterializationError(
                        f"Parameter {self._qualified_name()} value {item!r} is outside its numeric bounds."
                    )
        if self._choices:
            allowed = {_choice_key(choice) for choice in self._choices}
            for item in values:
                if _choice_key(item) not in allowed:
                    raise TestCaseMaterializationError(
                        f"Parameter {self._qualified_name()} value {item!r} is not allowed."
                    )
        if self._validate is None:
            return
        for item in values:
            if not self._validate(item):
                message = self._validation_message or f"Invalid value {item!r}."
                raise TestCaseMaterializationError(
                    f"Parameter {self._qualified_name()} failed validation: {message}"
                )

    @overload
    def __get__(self, instance: None, owner: type[object] | None = None) -> Self: ...

    @overload
    def __get__(self, instance: object, owner: type[object] | None = None) -> Any: ...

    def __get__(
        self, instance: object | None, owner: type[object] | None = None
    ) -> Any:
        if instance is None:
            return self
        parameters = getattr(instance, "_case_parameters", {})
        if isinstance(parameters, Mapping) and self.name is not None:
            return parameters.get(self.name, _default_value(self.definition))
        return _default_value(self.definition)

    def _qualified_name(self) -> str:
        return f"{self.owner_name or 'Template'}.{self.name or 'parameter'}"


class FileDescriptor:
    def __init__(
        self,
        *,
        label: str | None = None,
        description: str | None = None,
        default: str | Path | None = None,
        required: bool = False,
        media_types: Iterable[str] = (),
        metadata: JsonObject | None = None,
    ) -> None:
        self.name: str | None = None
        self.definition = FileDefinition(
            name="pending",
            label=label,
            description=description,
            default=None if default is None else str(default),
            required=required,
            media_types=[str(item) for item in media_types],
            metadata=dict(metadata or {}),
        )

    def __set_name__(self, owner: type[object], name: str) -> None:
        self.name = name
        self.definition = self.definition.with_name(name)

    def __get__(
        self, instance: object | None, owner: type[object] | None = None
    ) -> Any:
        if instance is None:
            return self
        parameters = getattr(instance, "_case_parameters", {})
        if isinstance(parameters, Mapping) and self.name is not None:
            value = parameters.get(self.name, self.definition.default)
            return None if value is None else str(value)
        return self.definition.default


# === Public Functions === #


def parameter(values: object, **kwargs: object) -> Any:
    """Declare an input descriptor; its value type comes from the field annotation."""
    if isinstance(values, str) and not kwargs:
        from dokime.core.declarations import parameter as parameter_expression

        return parameter_expression(values)
    return ParameterDescriptor(values, **cast(Any, kwargs))


def literal(value: object) -> ParameterExpression:
    return ParameterExpression.literal(value)


def parameter_array(values: Iterable[object]) -> ParameterExpression:
    return ParameterExpression.array(values)


def integer_range(start: int, stop: int, step: int = 1) -> ParameterExpression:
    return ParameterExpression.integer_range(start, stop, step)


def arange(start: float, stop: float, step: float = 1.0) -> ParameterExpression:
    return ParameterExpression.arange(start, stop, step)


def linspace(start: float, stop: float, count: int) -> ParameterExpression:
    return ParameterExpression.linspace(start, stop, count)


def reference(parameter: str) -> ParameterExpression:
    return ParameterExpression.reference(parameter)


def dynamic_default(
    key: str,
    fallback: object = DEFAULT_UNSET,
) -> ParameterExpression:
    return ParameterExpression.dynamic_default(key, fallback)


def negate(value: object) -> ParameterExpression:
    return ParameterExpression.unary("negate", value)


def add(left: object, right: object) -> ParameterExpression:
    return ParameterExpression.binary("add", left, right)


def subtract(left: object, right: object) -> ParameterExpression:
    return ParameterExpression.binary("subtract", left, right)


def multiply(left: object, right: object) -> ParameterExpression:
    return ParameterExpression.binary("multiply", left, right)


def divide(left: object, right: object) -> ParameterExpression:
    return ParameterExpression.binary("divide", left, right)


def file(**kwargs: object) -> FileDescriptor:
    return FileDescriptor(**cast(Any, kwargs))


# === Private Functions === #


def _parameter_expression(value: object) -> ParameterExpression | None:
    return value if isinstance(value, ParameterExpression) else None


def _infer_expression_value_type(expression: ParameterExpression | None) -> str | None:
    if expression is None:
        return None
    contract = expression.to_contract()
    kind = contract.get("kind")
    if kind == "range":
        return "integer"
    if kind in {"arange", "linspace", "unary", "binary"}:
        return "number"
    if kind == "literal":
        value = contract.get("value")
        return None if value is None else value_type_name(type(value))
    if kind == "array":
        values = contract.get("values")
        if isinstance(values, list) and values:
            first = values[0]
            if isinstance(first, Mapping):
                return _infer_expression_value_type(
                    ParameterExpression(cast(JsonObject, dict(first)))
                )
    return None


def _value_type_is_list(value_type: object | None) -> bool:
    if value_type is None:
        return False
    if isinstance(value_type, str):
        normalized = value_type.strip().lower()
        return normalized in {"array", "list"} or normalized.startswith(
            ("list[", "tuple[", "set[")
        )
    origin = get_origin(value_type)
    return origin in {list, tuple, set} or value_type in {list, tuple, set}


def _list_element_type(
    annotation: object | None, explicit: object | None = None
) -> str | None:
    element = explicit
    if element is None:
        args = get_args(annotation)
        if get_origin(annotation) is list and len(args) == 1:
            element = args[0]
        elif isinstance(annotation, str):
            match = re.fullmatch(
                r"(?:typing\.)?[Ll]ist\[\s*(\w+)\s*\]", annotation.strip()
            )
            if match:
                element = match.group(1)
    if element is None:
        return None
    aliases = {
        "bool": "boolean",
        "boolean": "boolean",
        "int": "integer",
        "integer": "integer",
        "float": "number",
        "number": "number",
        "str": "text",
        "text": "text",
    }
    if _is_enum_type(element):
        enum_values = list(cast(type[Enum], element))
        if not enum_values:
            raise TypeError("A list element enum must declare at least one value.")
        name = value_type_name(type(enum_values[0].value))
    else:
        name = element if isinstance(element, str) else value_type_name(element)
    if name not in aliases:
        raise TypeError(
            "List parameters require bool, int, float, str, or scalar-valued enum elements."
        )
    return aliases[name]


def _is_enum_type(value: object) -> bool:
    return isinstance(value, type) and issubclass(value, Enum)


def _parameter_values(value: object, *, expansion: str | dict[str, str]) -> list[Any]:
    if isinstance(value, ParameterExpression):
        return []
    kind = expansion if isinstance(expansion, str) else expansion.get("kind")
    if kind == "single":
        return [_wire_value(value)]
    return [_wire_value(item) for item in _iter_parameter_items(value)]


def _iter_parameter_items(value: object) -> Iterable[Any]:
    if isinstance(value, ParameterExpression):
        return ()
    if _is_enum_type(value):
        return cast(type[Enum], value)
    if _is_scalar(value):
        return (value,)
    return cast(Iterable[Any], value)


def _is_scalar(value: object) -> bool:
    if value is None:
        return True
    if isinstance(
        value, str | bytes | Path | Enum | bool | int | float | ParameterExpression
    ):
        return True
    if get_origin(value) is not None:
        return True
    return not isinstance(value, Iterable)


def _parameter_choices(choices: object) -> list[Any]:
    if _is_enum_type(choices):
        return [_wire_value(item) for item in cast(type[Enum], choices)]
    if choices is None:
        return []
    return [_wire_value(item) for item in cast(Iterable[Any], choices)]


def _wire_value(value: object) -> object:
    return value.value if isinstance(value, Enum) else value


def _choice_key(value: object) -> str:
    return _native.stable_wire_key(_wire_value(value))


def _infer_value_type(
    values: Sequence[Any],
    choices: Sequence[Any],
    expression: ParameterExpression | None = None,
) -> str | None:
    expression_value_type = _infer_expression_value_type(expression)
    if expression_value_type is not None:
        return expression_value_type
    candidates = [*choices]
    for value in values:
        if isinstance(value, list):
            candidates.extend(value)
        else:
            candidates.append(value)
    for value in candidates:
        if value is not None:
            return value_type_name(type(value))
    return None


def _parameter_expression_contract(value: object) -> JsonObject:
    expression = _parameter_expression(value) or ParameterExpression.literal(value)
    return expression.to_contract()


def _parameter_metadata(
    metadata: Mapping[str, object],
    expression: ParameterExpression | None,
) -> JsonObject:
    resolved = dict(metadata)
    if expression is not None:
        resolved["expression"] = expression.to_contract()
    return resolved


def _default_value(definition: AuthoredParameterDefinition) -> object:
    if not definition.values:
        return None
    return definition.values[0]


def _contract_value(value: object) -> object:
    value = _wire_value(value)
    if isinstance(value, ParameterExpression):
        return value.to_contract()
    to_contract = getattr(value, "to_contract", None)
    if callable(to_contract):
        return to_contract()
    if value is None or isinstance(value, bool | int | float | str | datetime):
        return value
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, Mapping):
        return {str(key): _contract_value(item) for key, item in value.items()}
    if isinstance(value, Iterable) and not isinstance(value, str | bytes):
        return [_contract_value(item) for item in value]
    return str(value)


def _value_from_contract(value: object) -> object:
    """Decode a canonical Value at the authored Python data boundary."""
    if not isinstance(value, dict):
        raise ValueError("expected a tagged value")
    kind = value["type"]
    payload = value.get("value")
    if kind == "null":
        return None
    if kind in {"bool", "text"}:
        return payload
    if kind == "integer":
        return int(str(payload))
    if kind == "unsigned":
        return UnsignedValue(int(str(payload)))
    if kind == "number":
        return float(str(payload))
    if kind == "datetime":
        return DatetimeValue.from_rfc3339(str(payload))
    if kind == "array" and isinstance(payload, list):
        return [_value_from_contract(item) for item in payload]
    if kind == "object" and isinstance(payload, dict):
        return {key: _value_from_contract(item) for key, item in payload.items()}
    raise ValueError(f"unknown value kind: {kind}")


# === Entry Point === #
