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
import re
from collections.abc import Callable, Sequence
from copy import copy
from dataclasses import dataclass, field
from types import FunctionType
from typing import Any, ClassVar, TypeVar
from weakref import WeakKeyDictionary

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from dokime._generated import shared
from dokime._generated.shared import DataExpression, FileParameter, Parameter, TestCase, TestCaseTemplate

from ._payloads import Payload, message_type
from .documents import Document
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
    "prompt",
    "record",
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
    identifier: str | None = None
    retries: int = 0
    repeatable: bool = False


@dataclass(frozen=True)
class Criterion:
    label: str
    handler: Callable[..., object]
    identifier: str | None = None


@dataclass(frozen=True)
class Stream:
    identifier: str | None = None
    row: type[Payload] = float
    label: str = ""

    def field(self, name: str = "value") -> Expression:
        return Expression(DataExpression(stream={"identifier": self.identifier, "field": name}))


@dataclass(frozen=True)
class Record[T: Payload]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]

    row: type[T]
    label: str = ""
    identifier: str | None = None


@dataclass(frozen=True)
class PromptDeclaration:
    value: shared.Prompt
    identifier: str | None = None


@dataclass
class Template:
    instance: type
    label: str = ""
    identifier: str | None = None
    steps: Sequence[Step] = ()
    criteria: Sequence[Criterion] = ()
    streams: Sequence[Stream] = ()
    parameters: Sequence[Parameter] = ()
    documents: Sequence[Document] = ()
    files: Sequence[FileParameter] = ()
    records: Sequence[Record[Any]] = field(init=False)
    prompts: Sequence[PromptDeclaration] = field(init=False)
    _methods: dict[str, Any] = field(init=False, repr=False)
    _identifiers: dict[int, str] = field(default_factory=dict, init=False, repr=False)
    _lifecycle: dict[str, str] = field(default_factory=dict, init=False, repr=False)
    _prompts: dict[int, shared.Prompt] = field(default_factory=dict, init=False, repr=False)

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
        self.records = tuple(member for member in self._methods.values() if isinstance(member, Record))
        self.prompts = tuple(member for member in self._methods.values() if isinstance(member, PromptDeclaration))

    def bind(self, prefix: str, *, single: bool = False) -> "Template":
        local = build_identifier(self.instance.__name__, self.identifier, class_name=True)
        bound = copy(self)
        bound.identifier = prefix if single else build_identifier(local, prefix=prefix)
        bound._identifiers = {}
        bound._lifecycle = {name: build_identifier(name, prefix=bound.identifier) for name in ("materialize", "initialize", "cleanup")}
        bound._prompts = {}
        names: list[str] = []
        declarations: list[tuple[object, str, str | None]] = []
        members: tuple[Step | Criterion, ...] = (*self.steps, *self.criteria)
        for item in members:
            declarations.append((item.handler, getattr(item.handler, "__name__", ""), item.identifier))
        for name, member in self._methods.items():
            if isinstance(member, Record):
                declarations.append((member, name, member.identifier))
            if isinstance(member, PromptDeclaration):
                declarations.append((member, name, member.identifier))
            stream = getattr(member, "__dokime_stream__", None)
            if isinstance(stream, Stream):
                declarations.append((member, name, stream.identifier))
                bound._identifiers[id(stream)] = build_identifier(name, stream.identifier, prefix=bound.identifier)
        for member, name, explicit in declarations:
            qualified = build_identifier(name, explicit, prefix=bound.identifier)
            names.append(qualified)
            bound._identifiers[id(member)] = qualified
        if len(names) != len(set(names)) or len(declarations) != len({id(member) for member, _, _ in declarations}):
            raise ValueError("Build Plugin: Give Every Template Member a Unique Declaration and Identifier")
        if any(name in bound._lifecycle.values() for name in names):
            raise ValueError("Build Plugin: Rename Members That Collide With Lifecycle Operations")
        for declaration in self.prompts:
            value = shared.Prompt()
            value.CopyFrom(declaration.value)
            value.metadata.identifier = bound.resolve(declaration)
            value.owner_suite = prefix
            value.template = bound.identifier
            bound._prompts[id(declaration)] = value
        return bound

    def resolve(self, declaration: object) -> str:
        member = getattr(declaration, "__func__", declaration)
        identity = self._identifiers.get(id(member))
        if identity is None:
            raise ValueError("Resolve Declaration: Supply a Member of This Template")
        return identity

    def definition(self) -> TestCaseTemplate:
        value = TestCaseTemplate(
            metadata={"identifier": self.identifier, "label": self.label or self.instance.__name__},
            materialize=self._lifecycle["materialize"],
            cleanup=self._lifecycle["cleanup"],
            parameters=self.parameters,
            files=self.files,
            documents=[item.message for item in self.documents],
        )
        if "initialize" in self._methods:
            value.initialize = self._lifecycle["initialize"]
        # Declare the Procedure on the Template So Catalogs Show It Before Materialization.
        for declared in self.steps:
            entry = value.steps.add(
                metadata={"identifier": self.resolve(declared.handler), "label": declared.label},
                operation=self.resolve(declared.handler),
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
                raise TypeError("Materialize Test Case: Use a Synchronous, Pure Materializer")
            return case
        for value in self.steps:
            case.step(value.label, value.handler, identifier=None)
        for criterion in self.criteria:
            case.criterion(criterion.label, criterion.handler, identifier=None)
        return case


@dataclass(frozen=True)
class Suite:
    identifier: str | None
    templates: Sequence[Template]
    label: str = ""
    _single: bool = False

    def definition(self) -> shared.Suite:
        return shared.Suite(
            metadata={"identifier": self.identifier, "label": self.label or self.identifier},
            templates=[item.definition() for item in self.templates],
        )


class Case:
    def __init__(self, template: Template, parameters: dict[str, Scalar]) -> None:
        self.template = template
        self.message = TestCase(
            parameters={key: encode(value) for key, value in parameters.items()},
            streams=[template.resolve(item) for item in template.streams],
        )
        self.bindings: dict[tuple[str, str], tuple[Callable[..., object], dict[str, object]]] = {}

    def step(self, label: str, handler: Callable[..., object], *, identifier: str | None = None, **arguments: object) -> "Case":
        return self._add("step", label, handler, identifier, arguments)

    def criterion(self, label: str, handler: Callable[..., object], *, identifier: str | None = None, **arguments: object) -> "Case":
        return self._add("criterion", label, handler, identifier, arguments)

    def _add(self, kind: str, label: str, handler: Callable[..., object], identifier: str | None, arguments: dict[str, object]) -> "Case":
        handler = getattr(handler, "__func__", handler)
        definitions = self.template.steps if kind == "step" else self.template.criteria
        definition = next((item for item in definitions if item.handler is handler), None)
        if definition is None:
            raise ValueError(f"Materialize {kind.title()}: Declare the Handler on This Template Before Adding It")
        identity = (
            self.template.resolve(handler)
            if identifier is None
            else build_identifier(getattr(handler, "__name__", ""), identifier, prefix=self.template.identifier)
        )
        if not identity or (kind, identity) in self.bindings:
            raise ValueError(f"Materialize {kind.title()}: Supply a Unique, Nonempty Local Identifier: {identity}")
        operation = self.template.resolve(handler)
        entries = self.message.steps if kind == "step" else self.message.criteria
        value = entries.add(metadata={"identifier": identity, "label": label}, operation=operation)
        if isinstance(definition, Step) and isinstance(value, shared.Step):
            value.repeatable = definition.repeatable
        self.bindings[kind, identity] = (handler, arguments)
        return self


# === Public Functions === #


def build_identifier(name: str, explicit: str | None = None, *, prefix: str | None = None, class_name: bool = False) -> str:
    if explicit is not None:
        local = explicit
    else:
        if class_name:
            name = re.sub(r"(?:Template|Suite)$", "", name)
        name = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
        name = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
        local = name.lower()
    if not local or not local.strip():
        raise ValueError("Build Plugin: Supply a Nonempty Declaration Identifier")
    return f"{prefix}.{local}" if prefix else local


def record[T: Payload](row_type: type[T], *, label: str = "", identifier: str | None = None) -> Record[T]:
    message_type(row_type)
    return Record(row_type, label, identifier)


def prompt(
    text: str | shared.Prompt,
    *,
    choices: Sequence[str] = (),
    label: str = "",
    identifier: str | None = None,
) -> PromptDeclaration:
    value = shared.Prompt()
    if isinstance(text, shared.Prompt):
        value.CopyFrom(text)
        if identifier is None and value.metadata.identifier:
            identifier = value.metadata.identifier
    else:
        value.message = text
        value.metadata.label = label or text
        for index, choice in enumerate(choices or ("OK",)):
            value.actions.add(identifier=str(index), label=choice)
    return PromptDeclaration(value, identifier)


def step[**P, R](
    label: str, *, identifier: str | None = None, retries: int = 0, repeatable: bool = False
) -> Callable[[Callable[P, R]], Callable[P, R]]:
    if retries < 0:
        raise ValueError("Declare Step: Supply a Nonnegative Retry Count")

    def decorate(handler: Callable[P, R]) -> Callable[P, R]:
        if not isinstance(handler, FunctionType):
            raise TypeError("Register Definition: Supply a Function")
        vars(handler)["__dokime_step__"] = Step(label, handler, identifier, retries, repeatable)
        return handler

    return decorate


def criterion[**P, R](label: str, *, identifier: str | None = None) -> Callable[[Callable[P, R]], Callable[P, R]]:
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


def stream[**P, R](label: str, *, identifier: str | None = None, row: type[Payload] = float) -> Callable[[Callable[P, R]], Callable[P, R]]:
    message_type(row)

    def decorate(handler: Callable[P, R]) -> Callable[P, R]:
        if not isinstance(handler, FunctionType):
            raise TypeError("Register Definition: Supply a Function")
        vars(handler)["__dokime_stream__"] = Stream(identifier, row, label)
        return handler

    return decorate


def suite[ClassT](
    identifier: str | None = None, *, label: str = "", parameters: Sequence[Parameter] = (), documents: Sequence[Document] = ()
) -> Callable[[type[ClassT]], type[ClassT]]:
    def decorate(instance: type[ClassT]) -> type[ClassT]:
        template = Template(instance, label, identifier, parameters=parameters, documents=documents)
        _SUITES[instance] = Suite(identifier, (template,), label, True)
        return instance

    return decorate


# === Private Functions === #


def _sampling[**P, R](sampling: Sampling) -> Callable[[Callable[P, R]], Callable[P, R]]:
    if sampling.interval_ms <= 0:
        raise ValueError("Declare Observer: Supply a Positive Interval")

    def decorate(handler: Callable[P, R]) -> Callable[P, R]:
        if not isinstance(handler, FunctionType):
            raise TypeError("Register Definition: Supply a Function")
        vars(handler)["__dokime_observer__"] = sampling
        return handler

    return decorate


# === Entry Point === #
