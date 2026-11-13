# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Composed Suite Definitions And Python Authoring Decorators."""

# === Imports === #

# Standard Library Imports
import inspect
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from types import FunctionType
from typing import Any

# Third Party Imports
from google.protobuf.message import Message
from google.protobuf.wrappers_pb2 import DoubleValue

# Local Imports
from .documents import Document, metadata
from .plot import Expression
from .v1 import message
from .values import encode

# Type Checking Imports

# === Public Exports === #

__all__ = [
    "Case",
    "Criterion",
    "Record",
    "Step",
    "Stream",
    "Suite",
    "Template",
    "criterion",
    "materialize",
    "observer",
    "step",
    "stream",
    "suite",
]

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class Step:
    label: str
    handler: FunctionType
    identifier: str = ""
    retries: int = 0
    repeatable: bool = False


@dataclass(frozen=True)
class Criterion:
    label: str
    handler: FunctionType
    identifier: str = ""


@dataclass(frozen=True)
class Stream:
    identifier: str
    row: type[Message] = DoubleValue
    label: str = ""

    def field(self, name: str = "value") -> Expression:
        return Expression(message("DataExpression", stream={"identifier": self.identifier, "field": name}))


@dataclass(frozen=True)
class Record:
    identifier: str
    row: type[Message]
    label: str = ""


@dataclass
class Template:
    identifier: str
    instance: type
    label: str = ""
    steps: Sequence[Step] = ()
    criteria: Sequence[Criterion] = ()
    streams: Sequence[Stream] = ()
    parameters: Sequence[Message] = ()
    documents: Sequence[Document] = ()
    files: Sequence[Message] = ()
    _methods: dict[str, Any] = field(init=False, repr=False)

    def __post_init__(self) -> None:
        self._methods = {}
        for cls in reversed(self.instance.__mro__):
            self._methods.update(vars(cls))
        self.steps = tuple(self.steps) + tuple(
            method.__dokime_step__ for method in self._methods.values() if hasattr(method, "__dokime_step__")
        )
        self.criteria = tuple(self.criteria) + tuple(
            method.__dokime_criterion__ for method in self._methods.values() if hasattr(method, "__dokime_criterion__")
        )
        self.streams = tuple(self.streams) + tuple(
            method.__dokime_stream__ for method in self._methods.values() if hasattr(method, "__dokime_stream__")
        )

    def operation(self, handler: FunctionType, kind: str) -> str:
        definitions: Sequence[Step | Criterion] = self.steps if kind == "step" else self.criteria
        position = next(index for index, item in enumerate(definitions) if item.handler is handler)
        return f"{self.identifier}.{kind}.{position}"

    def definition(self) -> Any:
        value = message(
            "TestCaseTemplate",
            metadata={"identifier": self.identifier, "label": self.label or self.instance.__name__},
            materialize=f"{self.identifier}.materialize",
            cleanup=f"{self.identifier}.cleanup",
            parameters=self.parameters,
            files=self.files,
            documents=[item.message for item in self.documents],
        )
        if "initialize" in self._methods:
            value.initialize = f"{self.identifier}.initialize"
        return value

    def build(self, parameters: dict[str, Any]) -> "Case":
        case = Case(self, parameters)
        custom = next((method for method in self._methods.values() if getattr(method, "__dokime_materialize__", False)), None)
        if custom is not None:
            result = custom(self.instance, case, parameters)
            if inspect.isawaitable(result):
                if inspect.iscoroutine(result):
                    result.close()
                raise TypeError("Materialize must be synchronous and pure")
            return case
        for value in self.steps:
            case.step(value.label, value.handler, identifier=value.identifier)
        for criterion in self.criteria:
            case.criterion(criterion.label, criterion.handler, identifier=criterion.identifier)
        return case


@dataclass(frozen=True)
class Suite:
    identifier: str
    templates: Sequence[Template]
    label: str = ""

    def definition(self) -> Any:
        return message(
            "Suite",
            metadata={"identifier": self.identifier, "label": self.label or self.identifier},
            templates=[item.definition() for item in self.templates],
        )


class Case:
    def __init__(self, template: Template, parameters: dict[str, Any]) -> None:
        self.template = template
        self.message = message(
            "TestCase",
            parameters={key: encode(value) for key, value in parameters.items()},
            streams=[item.identifier for item in template.streams],
        )
        self.bindings: dict[tuple[str, str], tuple[FunctionType, dict[str, Any]]] = {}

    def step(self, label: str, handler: FunctionType, *, identifier: str = "", **arguments: Any) -> "Case":
        return self._add("step", label, handler, identifier, arguments)

    def criterion(self, label: str, handler: FunctionType, *, identifier: str = "", **arguments: Any) -> "Case":
        return self._add("criterion", label, handler, identifier, arguments)

    def _add(self, kind: str, label: str, handler: FunctionType, identifier: str, arguments: dict) -> "Case":
        definitions = self.template.steps if kind == "step" else self.template.criteria
        definition = next((item for item in definitions if item.handler is handler), None)
        if definition is None:
            raise ValueError(f"Undeclared {kind} handler: {handler}")
        identity = identifier or metadata(label).identifier
        if not identity or (kind, identity) in self.bindings:
            raise ValueError(f"Duplicate or empty {kind} identifier: {identity}")
        operation = self.template.operation(handler, kind)
        entries = self.message.steps if kind == "step" else self.message.criteria
        value = entries.add(metadata={"identifier": identity, "label": label}, operation=operation)
        if isinstance(definition, Step):
            value.repeatable = definition.repeatable
        self.bindings[kind, identity] = (handler, arguments)
        return self


# === Public Functions === #


def step(label: str, *, identifier: str = "", retries: int = 0, repeatable: bool = False) -> Callable[[Any], Any]:
    if retries < 0:
        raise ValueError("Retries must be nonnegative")

    def decorate(handler: Any) -> Any:
        handler.__dokime_step__ = Step(label, handler, identifier, retries, repeatable)
        return handler

    return decorate


def criterion(label: str, *, identifier: str = "") -> Callable[[Any], Any]:
    def decorate(handler: Any) -> Any:
        handler.__dokime_criterion__ = Criterion(label, handler, identifier)
        return handler

    return decorate


def materialize(handler: Any) -> Any:
    handler.__dokime_materialize__ = True
    return handler


def observer(*, interval_ms: int = 1000, interval_setting: str = "") -> Callable[[Any], Any]:
    if interval_ms <= 0:
        raise ValueError("Observer interval must be positive")

    def decorate(handler: Any) -> Any:
        handler.__dokime_observer__ = (interval_ms, interval_setting)
        return handler

    return decorate


def stream(label: str, *, identifier: str, row: type[Message] = DoubleValue) -> Callable[[Any], Any]:
    def decorate(handler: Any) -> Any:
        handler.__dokime_stream__ = Stream(identifier, row, label)
        return handler

    return decorate


def suite(
    identifier: str, *, label: str = "", parameters: Sequence[Message] = (), documents: Sequence[Document] = ()
) -> Callable[[Any], Any]:
    def decorate(instance: Any) -> Any:
        template = Template(identifier, instance, label, parameters=parameters, documents=documents)
        instance.__dokime_suite__ = Suite(identifier, (template,), label)
        return instance

    return decorate


# === Private Functions === #

# === Entry Point === #
