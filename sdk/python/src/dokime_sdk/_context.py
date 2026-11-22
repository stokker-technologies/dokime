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
from collections.abc import AsyncIterator, Mapping
from datetime import UTC, datetime, timedelta
from typing import TYPE_CHECKING, Any, ClassVar, Protocol, TypeVar, cast
from uuid import uuid4

# Third Party Imports
import harmos
from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from harmos import plugin

# Local Imports
from . import _logs
from ._errors import project_failure, refused
from ._generated.host import HostRoot
from ._payloads import Payload, pack

# Type Checking Imports
if TYPE_CHECKING:
    from . import operations
    from ._authoring import Record, Stream
    from ._plugin import Plugin

# === Public Exports === #

Gap = plugin.Gap
StreamRow = plugin.StreamRow

# === Constants === #

# === Types === #

# === Classes === #


class Context:
    def __init__(self, transport: plugin.Context, owner: "Plugin") -> None:
        self._transport = transport
        self._calls = cast(plugin.Context, _logs.HostCalls(transport))
        self._host = HostRoot(self._calls)
        self.plugin = owner
        self.attempt = transport.attempt
        self._stream_identities: Mapping[int, str] = owner._streams

    async def operation(
        self, operation: str, request: dict[str, Any] | None = None, *, policy: "operations.OperationPolicy | None" = None
    ) -> dict[str, Any]:
        await _logs.flush()
        return await self._transport.operation(operation, request, policy=policy._definition() if policy is not None else None)

    async def sleep(self, duration: float | timedelta) -> None:
        seconds = duration.total_seconds() if isinstance(duration, timedelta) else duration
        nanoseconds = round(seconds * 1_000_000_000)
        value = Duration(seconds=nanoseconds // 1_000_000_000, nanos=nanoseconds % 1_000_000_000)
        await _logs.flush()
        await self._transport.sleep(value)

    async def now(self) -> datetime:
        await _logs.flush()
        return (await self._transport.now()).ToDatetime(tzinfo=UTC)

    async def cancelled(self) -> str:
        await _logs.flush()
        return await self._transport.cancelled()

    @property
    def stop_requested(self) -> bool:
        return self._transport.wind_down_requested

    async def subscribe[T: Message](self, stream: "Stream[T]", *, max_item_bytes: int) -> "StreamSubscription[T]":
        await _logs.flush()
        identity = self._stream_identities.get(id(stream), stream.identifier)
        if identity is None:
            raise ValueError("Subscribe to Stream: Supply a Bound Stream Declaration or Its Explicit Identifier")
        if not issubclass(stream.row, Message):
            raise TypeError("Subscribe to Stream: Declare a Protobuf Row Type Supported by the Library")
        subscription = await self._transport.subscribe(identity, stream.row, max_item_bytes=max_item_bytes)
        return cast(StreamSubscription[T], subscription)

    async def secret(self, identifier: str) -> str:
        declaration = next((item for item in self.plugin._secrets if item.identifier == identifier), None)
        if declaration is None:
            raise ValueError("Read Secret: Select a Secret Declared on This Plugin")
        try:
            value = (await self._host.secrets.get(declaration.identifier)).secret
        except harmos.OperationError as error:
            failure = project_failure(error.value)
            while failure.cause is not None:
                failure = failure.cause
            if failure.code not in ("NOT_FOUND", "not_found"):
                raise
            value = b""
        if not value:
            raise refused(
                "secrets.get",
                f"The {declaration.label} Secret Is Missing or Empty",
                f"Store the {declaration.label} secret for this plugin, then Retry",
            )
        try:
            return value.decode("utf-8")
        except UnicodeDecodeError as error:
            raise refused(
                "secrets.get",
                f"The {declaration.label} Secret Is Not UTF-8 Text",
                f"Store the {declaration.label} secret as text for this plugin, then Retry",
            ) from error

    async def resource[T: operations.Resource](self, resource: type[T]) -> T:
        if resource not in self.plugin._resources:
            raise ValueError("Open Resource: Select a Resource Declared on This Plugin")
        async with self.plugin._resource_locks[resource]:
            if self.plugin._resources_closed:
                raise refused("resource.open", "The Plugin Is Deactivating", "Restart the Plugin, Then Retry")
            if resource not in self.plugin._resource_values:
                value = resource()
                await value.open(self)
                self.plugin._resource_values[resource] = value
            return cast(T, self.plugin._resource_values[resource])

    async def record[T: Payload](self, declaration: "Record[T]", payload: T, *, request_id: str | None = None) -> int:
        identity = self.plugin._record_identities.get(id(declaration))
        if identity is None:
            raise ValueError("Write Plugin Record: Declare the Record on This Plugin")
        row = pack(declaration.row, payload)
        at = await self._transport.now()
        reply = await self._host.evidence.records.append("", identity, row.SerializeToString(), at, request_id=request_id or str(uuid4()))
        return reply.ordinal

    def settings[T: operations.Settings](self, settings: type[T]) -> T:
        value = self.plugin._settings_values.get(settings)
        if value is None:
            raise ValueError("Read Settings: Declare the Settings Class and Activate the Plugin")
        return cast(T, value)


class StreamSubscription[T: Message](AsyncIterator[StreamRow[T] | Gap], Protocol):
    __type_params__: ClassVar[tuple[TypeVar, ...]]

    async def close(self) -> None: ...


# === Public Functions === #

# === Private Functions === #


# === Entry Point === #
