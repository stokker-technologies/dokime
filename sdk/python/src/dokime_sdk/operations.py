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
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from datetime import timedelta
from enum import IntEnum
from types import FunctionType
from typing import TYPE_CHECKING, Any, cast, get_type_hints

# Third Party Imports
import harmos
from google.protobuf.message import Message
from harmos import plugin

# Local Imports
from . import _context, _errors, _logs
from ._authoring import build_identifier
from ._generated import shared
from ._payloads import Payload, message_type, pack, unpack

# Type Checking Imports
if TYPE_CHECKING:
    from ._plugin import Plugin


# === Public Exports === #

Context = _context.Context
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
    transient: Callable[[Exception], bool] | None = None

    def __post_init__(self) -> None:
        if self.transient is not None and not callable(self.transient):
            raise TypeError("Declare Operation Policy: Supply a Callable transient Predicate")
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

    def _failure(self, error: Exception, identity: str, owner: str) -> harmos.FailureError:
        transient = self.transient is not None and self.transient(error)
        failure = _errors.Failure(
            "Transient" if transient else "Permanent",
            "handler_failed",
            f"{identity}: {error}",
            "Retry the Operation" if transient else "Correct the Handler or Its Inputs, Then Retry",
            identity,
            owner,
        )
        return harmos.FailureError(_errors.wire_failure(failure))


@dataclass(frozen=True)
class Setting:
    title: str
    description: str
    default: harmos.SettingValue
    minimum: harmos.SettingValue | None = None
    maximum: harmos.SettingValue | None = None

    @staticmethod
    def flag(title: str, *, default: bool, description: str = "") -> bool:
        return cast(bool, Setting(title, description, harmos.SettingValue(flag=default)))

    @staticmethod
    def integer(title: str, *, default: int, min: int | None = None, max: int | None = None, description: str = "") -> int:
        return cast(
            int,
            Setting(
                title,
                description,
                harmos.SettingValue(integer=default),
                harmos.SettingValue(integer=min) if min is not None else None,
                harmos.SettingValue(integer=max) if max is not None else None,
            ),
        )

    @staticmethod
    def number(title: str, *, default: float, min: float | None = None, max: float | None = None, description: str = "") -> float:
        return cast(
            float,
            Setting(
                title,
                description,
                harmos.SettingValue(number=default),
                harmos.SettingValue(number=min) if min is not None else None,
                harmos.SettingValue(number=max) if max is not None else None,
            ),
        )

    @staticmethod
    def text(title: str, *, default: str, description: str = "") -> str:
        return cast(str, Setting(title, description, harmos.SettingValue(text=default)))

    @staticmethod
    def duration(
        title: str, *, default: timedelta, min: timedelta | None = None, max: timedelta | None = None, description: str = ""
    ) -> timedelta:
        value = harmos.SettingValue()
        value.duration.FromTimedelta(default)
        minimum = harmos.SettingValue() if min is not None else None
        maximum = harmos.SettingValue() if max is not None else None
        if minimum is not None and min is not None:
            minimum.duration.FromTimedelta(min)
        if maximum is not None and max is not None:
            maximum.duration.FromTimedelta(max)
        return cast(timedelta, Setting(title, description, value, minimum, maximum))

    def _definition(self, name: str) -> harmos.SettingDeclaration:
        return harmos.SettingDeclaration(
            name=name,
            title=self.title,
            description=self.description,
            default=self.default,
            minimum=self.minimum,
            maximum=self.maximum,
        )

    def __get__(self, instance: "Settings | None", owner: "type[Settings]") -> object:
        if instance is None:
            return self
        for name, declaration in owner._fields().items():
            if declaration is self:
                return instance._values[name]
        raise ValueError("Read Settings: Declare the Setting on a Settings Class")

    def __set__(self, instance: "Settings", value: object) -> None:
        raise AttributeError("Change Settings: Save Values on the Host, Then Retry or Restart the Plugin")


class Settings:
    """Typed Settings Read From One Activation Snapshot."""

    def __init__(self, values: harmos.SettingValues) -> None:
        self._values: dict[str, object] = {}
        for name, declaration in self._fields().items():
            if name not in values.values:
                raise ValueError(f"Read Settings: Configure {name}, Then Retry the Plugin")
            value = values.values[name]
            kind = value.WhichOneof("value")
            if kind != declaration.default.WhichOneof("value"):
                raise ValueError(f"Read Settings: Save the Declared Type for {name}, Then Retry the Plugin")
            self._values[name] = value.duration.ToTimedelta() if kind == "duration" else getattr(value, kind)

    @classmethod
    def _fields(cls) -> dict[str, Setting]:
        fields: dict[str, Setting] = {}
        for parent in reversed(cls.__mro__):
            for name, value in vars(parent).items():
                fields.pop(name, None)
                if isinstance(value, Setting):
                    fields[name] = value
        return fields


@dataclass(frozen=True)
class Secret:
    identifier: str
    label: str

    def _definition(self) -> shared.SecretDeclaration:
        return shared.SecretDeclaration(metadata={"identifier": self.identifier, "label": self.label}, required=True)


class Resource:
    """A Lazily Opened Class Released When the Plugin Activation Ends."""

    async def open(self, ctx: Context) -> None:
        pass

    async def close(self) -> None:
        pass


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
        identity = identifier if identifier is not None else build_identifier(handler.__name__)
        if not isinstance(visibility, Visibility):
            raise TypeError("Declare Operation: Supply a Visibility Member")
        if not re.fullmatch(r"[a-z0-9_]+(?:\.[a-z0-9_]+)*", identity):
            raise ValueError("Declare Operation: Supply a Lowercase Dotted Identifier")
        if policy is not None:
            policy._validate(identity)
        if policy is not None and policy.retries is not None and not replay_safe:
            raise ValueError("Declare Operation: Mark Retried Operations replay_safe")
        definition = plugin.Operation(
            identity,
            message_type(request),
            message_type(response),
            cast(Any, visibility.value),
            policy._definition() if policy is not None else None,
            description,
            replay_safe,
            lane=lane,
        )
        vars(handler)["__dokime_operation__"] = definition
        vars(handler)["__dokime_policy__"] = policy
        return handler

    return decorate


# === Private Functions === #


def _bind_operation(handler: Callable[..., object], owner: "Plugin", identifier: str) -> Callable[..., Awaitable[Message]]:
    definition = cast(plugin.Operation[Message, Message], vars(handler)["__dokime_operation__"])
    policy = cast(OperationPolicy | None, vars(handler)["__dokime_policy__"]) or OperationPolicy()
    annotations = get_type_hints(handler)
    parameters = tuple(inspect.signature(handler).parameters)
    request_type = annotations[parameters[1]]
    response_type = annotations["return"]

    async def execute(adapter: object, transport: plugin.Context, request: Message) -> Message:
        context = Context(transport, owner)
        try:
            result = await _logs.run(context, handler, context, unpack(request_type, request))
            return pack(response_type, cast(Payload, result))
        except _errors.SdkError as error:
            raise harmos.FailureError(_errors.wire_failure(error.failure)) from error
        except harmos.FailureError:
            raise
        except Exception as error:
            raise policy._failure(error, identifier, owner.identifier) from error

    return plugin.operation(
        identifier,
        request=definition.request,
        response=definition.response,
        visibility=definition.visibility,
        policy=definition.policy,
        description=definition.description,
        replay_safe=definition.replay_safe,
        lane=definition.lane,
    )(execute)


# === Entry Point === #
