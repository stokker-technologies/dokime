# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Typed Product Operations with Plain Payloads."""

# === Imports === #

# Standard Library Imports
import inspect
import math
import re
from collections.abc import Awaitable, Callable, Sequence
from dataclasses import dataclass
from datetime import timedelta
from enum import IntEnum
from types import CodeType, FunctionType, ModuleType
from typing import TYPE_CHECKING, Any, ClassVar, TypeVar, cast, get_type_hints

# Third Party Imports
import harmos
from google.protobuf.message import Message
from harmos import plugin

# Local Imports
from . import _context, _errors
from ._authoring import build_identifier
from ._generated import shared
from ._payloads import Payload, message_type, pack, unpack

# Type Checking Imports
if TYPE_CHECKING:
    from ._plugin import Plugin


# === Public Exports === #

Context = _context.Context
BoundOperation = _context.BoundOperation
StreamSubscription = _context.StreamSubscription
FailureError = _errors.FailureError


# === Constants === #

# === Types === #

# === Classes === #


class Visibility(IntEnum):
    PRIVATE = int(harmos.VISIBILITY_PRIVATE)
    PROTECTED = int(harmos.VISIBILITY_PROTECTED)
    PUBLIC = int(harmos.VISIBILITY_PUBLIC)


@dataclass(frozen=True)
class RetryPolicy:
    total_attempts: int
    initial: timedelta
    maximum: timedelta | None = None
    deterministic_jitter: bool = False

    def __post_init__(self) -> None:
        if type(self.total_attempts) is not int or self.total_attempts < 1:
            raise ValueError("Declare Retry Policy: Supply Positive total_attempts")


@dataclass(frozen=True)
class OperationPolicy:
    timeout: timedelta | None = None
    attempt_timeout: timedelta | None = None
    cancellation_timeout: timedelta | None = None
    retries: RetryPolicy | None = None

    def __post_init__(self) -> None:
        for duration in (self.timeout, self.attempt_timeout, self.cancellation_timeout):
            if duration is not None and (not math.isfinite(duration.total_seconds()) or duration <= timedelta()):
                raise ValueError("Declare Operation Policy: Supply Positive Finite Durations")

    def _validate(self, operation: str) -> None:
        if self.retries is None:
            return
        if not math.isfinite(self.retries.initial.total_seconds()) or self.retries.initial <= timedelta():
            raise ValueError(f"Declare Operation {operation}: Set retries.initial to a Positive Finite Backoff")
        if self.retries.maximum is not None and self.retries.maximum < self.retries.initial:
            raise ValueError(f"Declare Operation {operation}: Set retries.maximum at Least retries.initial")

    def _definition(self) -> shared.OperationPolicy:
        value = shared.OperationPolicy()
        for name in ("timeout", "attempt_timeout", "cancellation_timeout"):
            duration = getattr(self, name)
            if duration is not None:
                getattr(value, name).FromTimedelta(duration)
        if self.retries is not None:
            value.retries.total_attempts = self.retries.total_attempts
            value.retries.initial.FromTimedelta(self.retries.initial)
            if self.retries.maximum is not None:
                value.retries.maximum.FromTimedelta(self.retries.maximum)
            value.retries.deterministic_jitter = self.retries.deterministic_jitter
        return value


@dataclass(frozen=True)
class SettingValue:
    flag: bool | None = None
    integer: int | None = None
    number: float | None = None
    text: str | None = None
    duration: timedelta | None = None

    def __post_init__(self) -> None:
        variants = (self.flag, self.integer, self.number, self.text, self.duration)
        if sum(item is not None for item in variants) != 1:
            raise ValueError("Declare Setting: Supply Exactly One Value Variant")
        for name, expected in (("flag", bool), ("integer", int), ("number", float), ("text", str), ("duration", timedelta)):
            item = getattr(self, name)
            if item is not None and type(item) is not expected:
                raise TypeError(f"Declare Setting: Supply {expected.__name__} for {name}")

    def _definition(self) -> shared.SettingValue:
        value = shared.SettingValue()
        for name in ("flag", "integer", "number", "text", "duration"):
            item = getattr(self, name)
            if item is None:
                continue
            if name == "duration":
                value.duration.FromTimedelta(item)
            else:
                setattr(value, name, item)
        return value


@dataclass(frozen=True)
class SettingDeclaration:
    name: str
    title: str
    description: str
    default: SettingValue
    minimum: SettingValue | None = None
    maximum: SettingValue | None = None

    def _definition(self) -> shared.SettingDeclaration:
        return shared.SettingDeclaration(
            name=self.name,
            title=self.title,
            description=self.description,
            default=self.default._definition(),
            minimum=self.minimum._definition() if self.minimum is not None else None,
            maximum=self.maximum._definition() if self.maximum is not None else None,
        )


@dataclass(frozen=True)
class SecretDeclaration:
    identifier: str
    label: str
    required: bool = True

    def _definition(self) -> shared.SecretDeclaration:
        return shared.SecretDeclaration(metadata={"identifier": self.identifier, "label": self.label}, required=self.required)


@dataclass(frozen=True)
class Operation[Q: Payload, R: Payload]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]

    identifier: str
    request: type[Q]
    response: type[R]
    visibility: Visibility
    policy: OperationPolicy | None = None
    description: str = ""
    replay_safe: bool = False
    lane: plugin.Lane | None = None

    def __post_init__(self) -> None:
        if not isinstance(self.visibility, Visibility):
            raise TypeError("Declare Operation: Supply a Visibility Member")
        if not re.fullmatch(r"[a-z0-9_]+(?:\.[a-z0-9_]+)*", self.identifier):
            raise ValueError("Declare Operation: Supply a Lowercase Dotted Identifier")
        if self.policy is not None:
            self.policy._validate(self.identifier)
        if self.policy is not None and self.policy.retries is not None and not self.replay_safe:
            raise ValueError("Declare Operation: Mark Retried Operations replay_safe")
        message_type(self.request)
        message_type(self.response)

    def _definition(self) -> plugin.Operation[Message, Message]:
        return plugin.Operation(
            self.identifier,
            message_type(self.request),
            message_type(self.response),
            cast(Any, self.visibility.value),
            self.policy._definition() if self.policy is not None else None,
            self.description,
            self.replay_safe,
            lane=self.lane,
        )


# === Public Functions === #


def operation[F: Callable[..., object]](
    identifier: str | None = None,
    *,
    visibility: Visibility = Visibility.PUBLIC,
    policy: OperationPolicy | None = None,
    description: str = "",
    replay_safe: bool = False,
    lane: plugin.Lane | None = None,
) -> Callable[[F], F]:
    def decorate(handler: F) -> F:
        if not isinstance(handler, FunctionType) or not inspect.iscoroutinefunction(handler):
            raise TypeError("Declare Operation: Supply an Async Plain Function")
        parameters = tuple(inspect.signature(handler).parameters.values())
        if len(parameters) != 2 or any(item.kind not in (item.POSITIONAL_ONLY, item.POSITIONAL_OR_KEYWORD) for item in parameters):
            raise TypeError("Declare Operation: Annotate Exactly ctx and One Request Argument")
        try:
            annotations = get_type_hints(handler)
            request = annotations[parameters[1].name]
            response = annotations["return"]
        except (KeyError, NameError, TypeError) as error:
            raise TypeError("Declare Operation: Supply Resolvable Request and Result Payload Annotations") from error
        definition = Operation(
            identifier if identifier is not None else build_identifier(handler.__name__),
            request,
            response,
            visibility,
            policy,
            description,
            replay_safe,
            lane,
        )
        vars(handler)["__dokime_operation__"] = definition
        return handler

    return decorate


def declaration[Q: Payload, R: Payload](handler: Callable[[Context, Q], Awaitable[R]]) -> Operation[Q, R]:
    """Read the Immutable Declaration Attached by the Operation Decorator."""
    definition = vars(handler).get("__dokime_operation__") if isinstance(handler, FunctionType) else None
    if not isinstance(definition, Operation):
        raise TypeError("Read Operation Declaration: Supply a Function Decorated With operation")
    return cast(Operation[Q, R], definition)


def _bind_operation(handler: Callable[..., object], owner: "Plugin", identifier: str) -> Callable[..., Awaitable[Message]]:
    definition = declaration(cast(Callable[[Context, Any], Awaitable[Any]], handler))
    transport = definition._definition()

    async def execute(adapter: object, transport: plugin.Context, request: Message) -> Message:
        context = Context(transport, owner)
        try:
            result = handler(context, unpack(definition.request, request))
            if inspect.isawaitable(result):
                result = await result
            return pack(definition.response, cast(Payload, result))
        except _errors.SdkError as error:
            raise harmos.FailureError(_errors.wire_failure(error.failure)) from error

    return plugin.operation(
        identifier,
        request=transport.request,
        response=transport.response,
        visibility=transport.visibility,
        policy=transport.policy,
        description=transport.description,
        replay_safe=transport.replay_safe,
        lane=transport.lane,
    )(execute)


# === Private Functions === #


def _requirements(
    handlers: Sequence[object], owners: Sequence[type], registered: Sequence[Callable[..., object]]
) -> tuple[plugin.Operation[Message, Message], ...]:
    """Freeze Typed References Reachable From Author Handlers Before Activation."""
    pending = [(item, frozenset[str]()) for item in handlers]
    visited: set[tuple[int, frozenset[str]]] = set()
    required: dict[str, plugin.Operation[Message, Message]] = {}
    while pending:
        value, attributes = pending.pop()
        identity = (id(value), attributes)
        if identity in visited:
            continue
        visited.add(identity)
        if isinstance(value, Operation):
            if value.visibility == Visibility.PRIVATE:
                continue
            definition = value._definition()
            previous = required.get(definition.id)
            if previous is not None and previous.fingerprint != definition.fingerprint:
                raise ValueError(f"Build Plugin: Import One Matching Contract for {definition.id}")
            required[definition.id] = definition
            continue
        if isinstance(value, (ModuleType, type)):
            pending.extend(
                (inspect.getattr_static(value, name), attributes)
                for name in attributes
                if inspect.getattr_static(value, name, None) is not None
            )
            continue
        if isinstance(value, (tuple, list, dict)):
            members = value.values() if isinstance(value, dict) else value
            pending.extend((item, attributes) for item in members)
            continue
        if isinstance(value, (staticmethod, classmethod)):
            pending.append((value.__func__, attributes))
            continue
        if not isinstance(value, FunctionType):
            continue
        declaration = vars(value).get("__dokime_operation__")
        if declaration is not None and value not in registered:
            pending.append((declaration, frozenset()))
        if value.__module__.startswith(("dokime_sdk.", "harmos.")):
            continue
        names: set[str] = set()
        code = [value.__code__]
        while code:
            current = code.pop()
            names.update(current.co_names)
            code.extend(item for item in current.co_consts if isinstance(item, CodeType))
        references = [value.__globals__[name] for name in names if name in value.__globals__]
        for cell in value.__closure__ or ():
            try:
                references.append(cell.cell_contents)
            except ValueError:
                # A Handler May Capture Its Plugin Before Construction Completes.
                continue
        references.extend(value.__defaults__ or ())
        references.extend((value.__kwdefaults__ or {}).values())
        for owner in owners:
            references.extend(
                inspect.getattr_static(owner, name) for name in names if inspect.getattr_static(owner, name, None) is not None
            )
        pending.extend((reference, frozenset(names)) for reference in references)
    return tuple(required[name] for name in sorted(required))


# === Entry Point === #
