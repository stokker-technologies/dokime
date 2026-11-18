# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Invocation Context Shared by Product Operations."""

# === Imports === #

# Standard Library Imports
from datetime import UTC, datetime, timedelta
from typing import TYPE_CHECKING, ClassVar, TypeVar

# Third Party Imports
from google.protobuf.duration_pb2 import Duration
from harmos import plugin

# Local Imports
from ._generated.host import HostRoot
from ._payloads import Payload, pack, unpack

# Type Checking Imports
if TYPE_CHECKING:
    from . import operations
    from ._plugin import Plugin

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Context:
    def __init__(self, transport: plugin.Context, owner: "Plugin") -> None:
        self._transport = transport
        self._host = HostRoot(transport)
        self.plugin = owner
        self.attempt = transport.attempt
        self._secrets = frozenset(item.identifier for item in owner.secrets)

    def operation[Q: Payload, R: Payload](self, operation: "operations.Operation[Q, R]") -> "BoundOperation[Q, R]":
        return BoundOperation(self._transport, operation)

    async def sleep(self, duration: float | timedelta) -> None:
        seconds = duration.total_seconds() if isinstance(duration, timedelta) else duration
        nanoseconds = round(seconds * 1_000_000_000)
        value = Duration(seconds=nanoseconds // 1_000_000_000, nanos=nanoseconds % 1_000_000_000)
        await self._host.time.sleep(value)

    async def now(self) -> datetime:
        return (await self._host.time.get()).ToDatetime(tzinfo=UTC)

    async def cancelled(self) -> str:
        return await self._transport.cancelled()

    async def secret(self, identifier: str) -> bytes:
        if identifier not in self._secrets:
            raise ValueError(f"Read Secret: Declare {identifier} on the Plugin Before Reading It")
        return (await self._host.secrets.get(identifier)).secret

    async def settings(self) -> dict[str, "operations.SettingValue"]:
        from . import operations

        response = await self._host.plugins.settings.list()
        values = {}
        for key, value in response.value.values.items():
            kind = value.WhichOneof("kind")
            match kind:
                case "flag":
                    result = operations.SettingValue(flag=value.flag)
                case "integer":
                    result = operations.SettingValue(integer=value.integer)
                case "number":
                    result = operations.SettingValue(number=value.number)
                case "text":
                    result = operations.SettingValue(text=value.text)
                case "duration":
                    result = operations.SettingValue(duration=value.duration.ToTimedelta())
                case _:
                    raise ValueError("Read Settings: Save Exactly One Typed Value for Each Setting")
            values[key] = result
        return values


class BoundOperation[RequestT: Payload, ResponseT: Payload]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]

    def __init__(self, transport: plugin.Context, operation: "operations.Operation[RequestT, ResponseT]") -> None:
        self._transport = transport
        self._operation = operation

    async def __call__(self, request: RequestT, *, policy: "operations.OperationPolicy | None" = None) -> ResponseT:
        wire = self._operation._definition()
        response = await self._transport.call(
            wire, pack(self._operation.request, request), policy=policy._definition() if policy is not None else None
        )
        return unpack(self._operation.response, response)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
