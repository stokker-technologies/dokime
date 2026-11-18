# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Typed Product Operations With Plain Payloads."""

# === Imports === #

# Standard Library Imports
import inspect
import math
import re
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from datetime import timedelta
from enum import IntEnum
from types import FunctionType
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
        if not math.isfinite(self.initial.total_seconds()) or self.initial < timedelta():
            raise ValueError("Declare Retry Policy: Supply a Nonnegative Initial Backoff")
        if self.maximum is not None and self.maximum < self.initial:
            raise ValueError("Declare Retry Policy: Set maximum at Least initial")


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

    def _definition(self) -> shared.SettingDeclaration:
        return shared.SettingDeclaration(name=self.name, title=self.title, description=self.description, default=self.default._definition())


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

    def __post_init__(self) -> None:
        if not isinstance(self.visibility, Visibility):
            raise TypeError("Declare Operation: Supply a Visibility Member")
        if not re.fullmatch(r"[a-z0-9_]+(?:\.[a-z0-9_]+)*", self.identifier):
            raise ValueError("Declare Operation: Supply a Lowercase Dotted Identifier")
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
        )


# === Public Functions === #


def operation[F: Callable[..., object]](
    identifier: str | None = None,
    *,
    visibility: Visibility,
    policy: OperationPolicy | None = None,
    description: str = "",
    replay_safe: bool = False,
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
        )
        vars(handler)["__dokime_operation__"] = definition
        return handler

    return decorate


def _bind_operation(handler: Callable[..., object], owner: "Plugin") -> Callable[..., Awaitable[Message]]:
    definition = cast(Operation[Any, Any], vars(handler)["__dokime_operation__"])
    transport = definition._definition()

    async def execute(adapter: object, transport: plugin.Context, request: Message) -> Message:
        context = Context(transport, owner)
        try:
            result = handler(context, unpack(definition.request, request))
            if inspect.isawaitable(result):
                result = await result
            return pack(definition.response, cast(Payload, result))
        except _errors.SdkError as error:
            raise harmos.Failure(_failure(error.failure)) from error

    return plugin.operation(
        transport.id,
        request=transport.request,
        response=transport.response,
        visibility=transport.visibility,
        policy=transport.policy,
        description=transport.description,
        replay_safe=transport.replay_safe,
    )(execute)


# === Private Functions === #


def _failure(value: _errors.Failure) -> harmos.FailureValue:
    return harmos.FailureValue(
        classification="CLASSIFICATION_" + value.classification.upper().replace(" ", "_"),
        code=value.code,
        message=value.message,
        action=value.action,
        operation=value.operation,
        plugin=value.plugin,
        cause=_failure(value.cause) if value.cause is not None else None,
        details=dict(value.details),
        diagnostics=value.diagnostics,
    )


# === Entry Point === #
