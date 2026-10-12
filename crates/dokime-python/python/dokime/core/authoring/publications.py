# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Typed record and stream row declarations for shared runtime publication."""

# === Imports === #

# Standard Library Imports
import types
from datetime import datetime
from dataclasses import asdict, dataclass, fields, is_dataclass
from typing import (
    Any,
    cast,
    Callable,
    Union,
    dataclass_transform,
    get_args,
    get_origin,
    get_type_hints,
)

# Third Party Imports

# Local Imports
from dokime.core.authoring.parameters import _value_from_contract
from dokime.core.metadata import Metadata
from dokime.client.models import UnsignedValue
from dokime.core.declarations.record_schemas import FieldDeclaration, RecordDeclaration
from dokime.core.declarations.telemetry_schemas import TelemetryStreamDeclaration

# Type Checking Imports

# === Public Exports === #

__all__ = ["record", "stream"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@dataclass_transform(frozen_default=True)
def record[T](
    *,
    id: str,
    label: str | None = None,
    description: str | None = None,
    severity: str = "info",
) -> Callable[[type[T]], type[T]]:
    """Declare a typed record; publish an instance with ``await ctx.publish``."""

    def decorate(cls: type[T]) -> type[T]:
        cls = dataclass(frozen=True)(cls)
        declaration = RecordDeclaration(
            id,
            _columns(cls),
            label,
            description,
            severity,
        )
        declaration.to_contract()

        def publication(self: T) -> tuple[str, dict[str, object]]:
            assert is_dataclass(self) and not isinstance(self, type)
            return "record", {
                "record_type": id,
                "severity": severity,
                "details": asdict(self),
            }

        setattr(cls, "__dokime_record__", declaration)
        setattr(cls, "__dokime_publication__", publication)
        return cls

    return decorate


@dataclass_transform(frozen_default=True)
def stream[T](
    *,
    id: str,
    label: str | None = None,
    default_x: str | None = None,
) -> Callable[[type[T]], type[T]]:
    """Declare a typed stream row and its protobuf field schema."""

    def decorate(cls: type[T]) -> type[T]:
        cls = dataclass(frozen=True)(cls)
        declaration = TelemetryStreamDeclaration(
            id,
            label,
            _columns(cls),
            default_x,
        )
        contract = declaration.to_contract()
        wire = {
            "metadata": Metadata(id, label or cls.__name__).to_contract(),
            "id": id,
            "schema": contract["schema_version"],
            "window": 4096,
            "fields": _fields(cls, ()),
        }

        def publication(self: T) -> tuple[str, dict[str, object]]:
            assert is_dataclass(self) and not isinstance(self, type)
            return "row", {"stream": id, "values": asdict(self)}

        setattr(cls, "__dokime_stream__", declaration)
        setattr(cls, "__dokime_wire__", wire)
        setattr(cls, "__dokime_publication__", publication)
        return cls

    return decorate


def decode[T](cls: type[T], values: dict[str, object]) -> T:
    """Restore declared nested dataclasses from the canonical evidence values."""
    return _decode(
        cls, {name: _value_from_contract(item) for name, item in values.items()}
    )


# === Private Functions === #


def _columns(cls: type) -> tuple[FieldDeclaration, ...]:
    hints = get_type_hints(cls)
    return tuple(
        FieldDeclaration(field.name, _column_type(hints[field.name]))
        for field in fields(cls)
    )


def _column_type(annotation: object) -> object:
    if annotation is UnsignedValue:
        return int
    if isinstance(annotation, type) and is_dataclass(annotation):
        return dict
    if get_origin(annotation) in (types.UnionType, Union):
        args = get_args(annotation)
        if len(args) == 2 and type(None) in args:
            item = next(item for item in args if item is not type(None))
            return cast(Any, _column_type(item)) | None
    return annotation


def _decode[T](cls: type[T], values: dict[str, object]) -> T:
    if not is_dataclass(cls):
        raise TypeError("evidence rows must be dataclasses")
    hints = get_type_hints(cls)

    def value(annotation: object, item: object) -> object:
        if item is None:
            return None
        if get_origin(annotation) in (types.UnionType, Union):
            annotation = next(
                arg for arg in get_args(annotation) if arg is not type(None)
            )
        if annotation is datetime:
            if not isinstance(item, datetime):
                raise TypeError(
                    "timestamp evidence must contain a canonical datetime value"
                )
            return item
        if get_origin(annotation) is list:
            if not isinstance(item, list):
                raise TypeError("list evidence must contain a list")
            (element,) = get_args(annotation)
            return [value(element, child) for child in item]
        if isinstance(annotation, type) and is_dataclass(annotation):
            if not isinstance(item, dict):
                raise TypeError("nested evidence must contain an object")
            return _decode(annotation, item)
        return item

    return cls(
        **{
            field.name: value(hints[field.name], values[field.name])
            for field in fields(cls)
            if field.name in values
        }
    )


def _fields(cls: type, ancestors: tuple[type, ...]) -> list[dict[str, object]]:
    if cls in ancestors or len(ancestors) >= 32:
        raise TypeError("stream rows require bounded, nonrecursive message fields")
    hints = get_type_hints(cls)
    result = []
    for tag, field in enumerate(fields(cls), start=1):
        annotation = hints[field.name]
        cardinality = "required"
        origin = get_origin(annotation)
        if origin in (types.UnionType, Union):
            args = get_args(annotation)
            if len(args) != 2 or type(None) not in args:
                raise TypeError(
                    f"stream field {field.name!r} only supports optional unions"
                )
            annotation = next(item for item in args if item is not type(None))
            cardinality = "optional"
        if get_origin(annotation) is list:
            if cardinality == "optional":
                raise TypeError("protobuf repeated fields cannot be optional")
            (annotation,) = get_args(annotation)
            cardinality = "repeated"
        kinds = {
            bool: "bool",
            int: "i64",
            UnsignedValue: "u64",
            float: "f64",
            str: "string",
        }
        if annotation in kinds:
            kind = {"type": kinds[annotation]}
        elif isinstance(annotation, type) and is_dataclass(annotation):
            kind = {"type": "message", "fields": _fields(annotation, (*ancestors, cls))}
        else:
            raise TypeError(
                f"unsupported stream field type for {field.name!r}: {annotation!r}"
            )
        result.append(
            {"name": field.name, "tag": tag, "kind": kind, "cardinality": cardinality}
        )
    return result


# === Entry Point === #
