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
from weakref import WeakKeyDictionary

# Third Party Imports
from google.protobuf.message import Message
from google.protobuf.wrappers_pb2 import DoubleValue

from dokime import DataExpression, FileParameter, Parameter, TestCase, TestCaseTemplate
from dokime import Step as StepValue
from dokime import Suite as SuiteValue

# Local Imports
from .documents import Document, metadata
from .plot import Expression
from .values import Scalar, encode

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
    "window_observer",
]

# === Constants === #

_SUITES: WeakKeyDictionary[type, "Suite"] = WeakKeyDictionary()

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class Sampling:
    """How an Observer Samples: Its Interval, Optional Settings and Window Columns."""

    interval_ms: float
    interval_setting: str
    window_setting: str
    columns: type[Message] | None


@dataclass(frozen=True)
class Step:
    label: str
    handler: Callable[..., object]
    identifier: str = ""
    retries: int = 0
    repeatable: bool = False


@dataclass(frozen=True)
class Criterion:
    label: str
    handler: Callable[..., object]
    identifier: str = ""


@dataclass(frozen=True)
class Stream:
    identifier: str
    row: type[Message] = DoubleValue
    label: str = ""

    def field(self, name: str = "value") -> Expression:
        return Expression(DataExpression(stream={"identifier": self.identifier, "field": name}))


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
    parameters: Sequence[Parameter] = ()
    documents: Sequence[Document] = ()
    files: Sequence[FileParameter] = ()
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

    def operation(self, handler: Callable[..., object], kind: str) -> str:
        definitions: Sequence[Step | Criterion] = self.steps if kind == "step" else self.criteria
        position = next(index for index, item in enumerate(definitions) if item.handler is handler)
        return f"{self.identifier}.{kind}.{position}"

    def definition(self) -> TestCaseTemplate:
        value = TestCaseTemplate(
            metadata={"identifier": self.identifier, "label": self.label or self.instance.__name__},
            materialize=f"{self.identifier}.materialize",
            cleanup=f"{self.identifier}.cleanup",
            parameters=self.parameters,
            files=self.files,
            documents=[item.message for item in self.documents],
        )
        if "initialize" in self._methods:
            value.initialize = f"{self.identifier}.initialize"
        # Declare the Procedure on the Template So Catalogs Show It Before Materialization.
        for declared in self.steps:
            entry = value.steps.add(
                metadata={"identifier": declared.identifier or metadata(declared.label).identifier, "label": declared.label},
                operation=self.operation(declared.handler, "step"),
            )
            entry.repeatable = declared.repeatable
        return value

    def build(self, parameters: dict[str, Scalar]) -> "Case":
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

    def definition(self) -> SuiteValue:
        return SuiteValue(
            metadata={"identifier": self.identifier, "label": self.label or self.identifier},
            templates=[item.definition() for item in self.templates],
        )


class Case:
    def __init__(self, template: Template, parameters: dict[str, Scalar]) -> None:
        self.template = template
        self.message = TestCase(
            parameters={key: encode(value) for key, value in parameters.items()},
            streams=[item.identifier for item in template.streams],
        )
        self.bindings: dict[tuple[str, str], tuple[Callable[..., object], dict[str, object]]] = {}

    def step(self, label: str, handler: Callable[..., object], *, identifier: str = "", **arguments: object) -> "Case":
        return self._add("step", label, handler, identifier, arguments)

    def criterion(self, label: str, handler: Callable[..., object], *, identifier: str = "", **arguments: object) -> "Case":
        return self._add("criterion", label, handler, identifier, arguments)

    def _add(self, kind: str, label: str, handler: Callable[..., object], identifier: str, arguments: dict[str, object]) -> "Case":
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
        if isinstance(definition, Step) and isinstance(value, StepValue):
            value.repeatable = definition.repeatable
        self.bindings[kind, identity] = (handler, arguments)
        return self


# === Public Functions === #


def step[**P, R](
    label: str, *, identifier: str = "", retries: int = 0, repeatable: bool = False
) -> Callable[[Callable[P, R]], Callable[P, R]]:
    if retries < 0:
        raise ValueError("Retries must be nonnegative")

    def decorate(handler: Callable[P, R]) -> Callable[P, R]:
        if not isinstance(handler, FunctionType):
            raise TypeError("Register Definition: Supply a Function")
        vars(handler)["__dokime_step__"] = Step(label, handler, identifier, retries, repeatable)
        return handler

    return decorate


def criterion[**P, R](label: str, *, identifier: str = "") -> Callable[[Callable[P, R]], Callable[P, R]]:
    def decorate(handler: Callable[P, R]) -> Callable[P, R]:
        if not isinstance(handler, FunctionType):
            raise TypeError("Register Definition: Supply a Function")
        vars(handler)["__dokime_criterion__"] = Criterion(label, handler, identifier)
        return handler

    return decorate


def materialize[**P, R](handler: Callable[P, R]) -> Callable[P, R]:
    if not isinstance(handler, FunctionType):
        raise TypeError("Register Materializer: Supply a Function")
    vars(handler)["__dokime_materialize__"] = True
    return handler


def observer[**P, R](
    *, interval_ms: float = 1000, interval_setting: str = "", window_setting: str = ""
) -> Callable[[Callable[P, R]], Callable[P, R]]:
    """Sample One Row per Call; Each Host Window Calls the Method Once per Sample, in Order.

    `interval_ms` may be fractional (`0.001` samples at 1 MHz). `window_setting` names a plugin
    duration setting that replaces the host's window length for this stream.
    """
    return _sampling(Sampling(interval_ms, interval_setting, window_setting, None))


def window_observer[**P, R](
    *, columns: type[Message], interval_ms: float = 1000, interval_setting: str = "", window_setting: str = ""
) -> Callable[[Callable[P, R]], Callable[P, R]]:
    """Answer a Whole Host Window per Call With `columns`, Which Repeats Every Field of the Row.

    The method receives the context and the `SampleWindow`; sample `i` is due at
    `window.start + i * window.period`, and every column holds `window.count` values.
    """
    return _sampling(Sampling(interval_ms, interval_setting, window_setting, columns))


def stream[**P, R](label: str, *, identifier: str, row: type[Message] = DoubleValue) -> Callable[[Callable[P, R]], Callable[P, R]]:
    def decorate(handler: Callable[P, R]) -> Callable[P, R]:
        if not isinstance(handler, FunctionType):
            raise TypeError("Register Definition: Supply a Function")
        vars(handler)["__dokime_stream__"] = Stream(identifier, row, label)
        return handler

    return decorate


def suite(
    identifier: str, *, label: str = "", parameters: Sequence[Parameter] = (), documents: Sequence[Document] = ()
) -> Callable[[type], type]:
    def decorate(instance: type) -> type:
        template = Template(identifier, instance, label, parameters=parameters, documents=documents)
        _SUITES[instance] = Suite(identifier, (template,), label)
        return instance

    return decorate


# === Private Functions === #


def _sampling[**P, R](sampling: Sampling) -> Callable[[Callable[P, R]], Callable[P, R]]:
    if sampling.interval_ms <= 0:
        raise ValueError("Observer interval must be positive")

    def decorate(handler: Callable[P, R]) -> Callable[P, R]:
        if not isinstance(handler, FunctionType):
            raise TypeError("Register Definition: Supply a Function")
        vars(handler)["__dokime_observer__"] = sampling
        return handler

    return decorate


# === Entry Point === #
