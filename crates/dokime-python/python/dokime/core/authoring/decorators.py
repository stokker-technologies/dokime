# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Authoring decorators for template steps, criteria, and lifecycle hooks."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
import inspect
from collections.abc import Callable, Mapping
from typing import Any, cast

# Third Party Imports

# Local Imports
from dokime.core.authoring.definitions import StepDefinition

# Type Checking Imports

# === Public Exports === #

# === Constants === #

STEP_ATTRIBUTE = "__dokime_step__"
SAMPLER_ATTRIBUTE = "__dokime_sampler__"
CRITERION_ATTRIBUTE = "__dokime_criterion__"
INITIALIZE_ATTRIBUTE = "__dokime_initialize__"
CLEANUP_ATTRIBUTE = "__dokime_cleanup__"

# === Types === #

# === Classes === #

# === Public Functions === #


def step(
    func: Callable[..., object] | None = None,
    *,
    step_id: str | None = None,
    label: str | None = None,
    description: str | None = None,
    order: int | None = None,
    repeatable: bool = False,
) -> Callable[..., object]:
    if not isinstance(repeatable, bool):
        raise TypeError("step repeatable must be a boolean")
    return _decorator(
        STEP_ATTRIBUTE,
        func,
        {
            "step_id": step_id,
            "label": label,
            "description": description,
            "repeatable": repeatable,
            "order": order,
        },
    )


def sampler(
    *,
    stream: type,
    interval_ms: int,
) -> Callable[[Callable[..., object]], Callable[..., object]]:
    if (
        not isinstance(interval_ms, int)
        or isinstance(interval_ms, bool)
        or interval_ms <= 0
    ):
        raise ValueError("sampler interval_ms must be a positive integer")
    if not hasattr(stream, "__dokime_stream__"):
        raise TypeError("a sampler names a declared stream row type")
    return cast(
        Callable[[Callable[..., object]], Callable[..., object]],
        _decorator(
            SAMPLER_ATTRIBUTE, None, {"row_type": stream, "interval_ms": interval_ms}
        ),
    )


def criterion(
    func: Callable[..., object] | None = None,
    *,
    criterion_id: str | None = None,
    label: str | None = None,
    order: int | None = None,
) -> Callable[..., object]:
    return _decorator(
        CRITERION_ATTRIBUTE,
        func,
        {
            "criterion_id": criterion_id,
            "label": label,
            "order": order,
        },
    )


def initialize(
    func: Callable[..., object] | None = None,
    *,
    initialize_id: str | None = None,
    label: str | None = None,
    order: int | None = None,
) -> Callable[..., object]:
    return _decorator(
        INITIALIZE_ATTRIBUTE,
        func,
        {
            "initialize_id": initialize_id,
            "label": label,
            "order": order,
        },
    )


def cleanup(
    func: Callable[..., object] | None = None,
    *,
    cleanup_id: str | None = None,
    label: str | None = None,
    order: int | None = None,
) -> Callable[..., object]:
    return _decorator(
        CLEANUP_ATTRIBUTE,
        func,
        {
            "cleanup_id": cleanup_id,
            "label": label,
            "order": order,
        },
    )


def snake_case_identifier(value: str) -> str:
    chars: list[str] = []
    previous_lower = False
    for char in value:
        if char.isupper() and previous_lower:
            chars.append("_")
        if char.isalnum():
            chars.append(char.lower())
            previous_lower = char.islower() or char.isdigit()
        else:
            if chars and chars[-1] != "_":
                chars.append("_")
            previous_lower = False
    return "".join(chars).strip("_") or "template"


def title_from_identifier(value: str) -> str:
    return value.replace(".", " ").replace("_", " ").replace("-", " ").title()


# === Private Functions === #


def _decorator(
    attribute: str,
    func: Callable[..., object] | None,
    metadata: Mapping[str, object],
) -> Callable[..., object]:
    def decorate(method: Callable[..., object]) -> Callable[..., object]:
        resolved = dict(metadata)
        target = (
            method.__func__
            if isinstance(method, staticmethod | classmethod)
            else method
        )
        setattr(target, attribute, resolved)
        return cast(Callable[..., object], method)

    return decorate if func is None else decorate(func)


def _phase_definitions(
    cls: type[object], attribute: str, definition_type: type[Any], id_field: str
) -> list[Any]:
    definitions = []
    for order, (method_name, method) in enumerate(_decorated_methods(cls, attribute)):
        metadata = dict(getattr(method, attribute))
        identifier = str(metadata.pop(id_field, None) or method_name)
        label = cast(str | None, metadata.pop("label", None)) or title_from_identifier(
            identifier
        )
        configured_order = metadata.pop("order", None)
        values = {
            id_field: identifier,
            "method_name": method_name,
            "label": label,
            "order": order
            if configured_order is None
            else int(cast(Any, configured_order)),
        }
        if definition_type is StepDefinition:
            values["repeatable"] = metadata.pop("repeatable", False)
            values["description"] = metadata.pop("description", None) or inspect.getdoc(
                method
            )
        definitions.append(definition_type(**values))
    return sorted(definitions, key=lambda definition: definition.order)


def _class_members(
    cls: type[object], expected_type: type[Any]
) -> list[tuple[str, Any]]:
    members: dict[str, Any] = {}
    for base in reversed(cls.mro()):
        for name, value in base.__dict__.items():
            if isinstance(value, expected_type):
                members[name] = value
    return list(members.items())


def _decorated_methods(
    cls: type[object], attribute: str
) -> list[tuple[str, Callable[..., object]]]:
    methods: dict[str, Callable[..., object]] = {}
    for base in reversed(cls.mro()):
        for name, value in base.__dict__.items():
            target = (
                value.__func__
                if isinstance(value, staticmethod | classmethod)
                else value
            )
            if callable(target) and hasattr(target, attribute):
                methods[name] = target
    return list(methods.items())


def _stream_id(stream: object) -> str:
    if isinstance(stream, str):
        return stream
    for name in ("stream", "stream_id", "id"):
        value = getattr(stream, name, None)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return str(stream)


# === Entry Point === #
