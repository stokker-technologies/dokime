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
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from enum import IntEnum
from typing import Any, cast

# Third Party Imports
import harmos
from google.protobuf.message import Message
from harmos import plugin

# Local Imports
from ._context import Context
from ._generated.shared import OperationPolicy
from ._payloads import Payload, message_type, pack, unpack
from .authoring import build_identifier

# Type Checking Imports


# === Public Exports === #

__all__ = ["Operation", "Visibility", "operation"]

# === Constants === #

# === Types === #

# === Classes === #


class Visibility(IntEnum):
    PRIVATE = harmos.VISIBILITY_PRIVATE
    PROTECTED = harmos.VISIBILITY_PROTECTED
    PUBLIC = harmos.VISIBILITY_PUBLIC


@dataclass(frozen=True)
class Operation[Q: Payload, R: Payload]:
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
        message_type(self.request)
        message_type(self.response)

    def _wire(self) -> plugin.Operation[Message, Message]:
        return plugin.Operation(
            build_identifier(self.identifier, self.identifier),
            message_type(self.request),
            message_type(self.response),
            cast(Any, self.visibility.value),
            self.policy,
            self.description,
            self.replay_safe,
        )


# === Public Functions === #


def operation[Q: Payload, R: Payload, F: Callable](
    identifier: str,
    *,
    request: type[Q],
    response: type[R],
    visibility: Visibility,
    policy: OperationPolicy | None = None,
    description: str = "",
    replay_safe: bool = False,
) -> Callable[[F], F]:
    definition = Operation(identifier, request, response, visibility, policy, description, replay_safe)

    def decorate(handler: F) -> F:
        vars(handler)["__dokime_operation__"] = definition
        return handler

    return decorate


def _bind_operation(handler: Callable[..., object]) -> Callable[..., Awaitable[Message]]:
    definition = cast(Operation[Any, Any], vars(handler)["__dokime_operation__"])
    wire = definition._wire()

    async def execute(adapter: object, transport: plugin.Context, request: Message) -> Message:
        context = Context(transport)
        result = handler(context, unpack(definition.request, request))
        if inspect.isawaitable(result):
            result = await result
        return pack(definition.response, cast(Payload, result))

    return plugin.operation(
        wire.id,
        request=wire.request,
        response=wire.response,
        visibility=wire.visibility,
        policy=wire.policy,
        description=wire.description,
        replay_safe=wire.replay_safe,
    )(execute)


# === Private Functions === #

# === Entry Point === #
