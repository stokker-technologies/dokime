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
from typing import TYPE_CHECKING, ClassVar, Protocol, TypeVar, cast

# Third Party Imports
import harmos
from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from harmos import VISIBILITY_PUBLIC, plugin

# Local Imports
import dokime_sdk._generated.client

from ._errors import Failure, FailureError, call, invalid, project_failure, refused
from ._generated.host import HostRoot
from ._payloads import Payload, message_type, pack, unpack

# Type Checking Imports
if TYPE_CHECKING:
    from . import operations
    from ._authoring import Stream
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
        self._host = HostRoot(transport)
        self.plugin = owner
        self.attempt = transport.attempt
        self._secrets = frozenset(item.identifier for item in owner.secrets)
        self._stream_identities: Mapping[int, str] = owner._streams

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

    async def subscribe[T: Message](self, stream: "Stream[T]", *, max_item_bytes: int) -> "StreamSubscription[T]":
        identity = self._stream_identities.get(id(stream), stream.identifier)
        if identity is None:
            raise ValueError("Subscribe to Stream: Supply a Bound Stream Declaration or Its Explicit Identifier")
        if not issubclass(stream.row, Message):
            raise TypeError("Subscribe to Stream: Declare a Protobuf Row Type Supported by the Library")
        subscription = await self._transport.subscribe(identity, stream.row, max_item_bytes=max_item_bytes)
        return cast(StreamSubscription[T], subscription)

    async def secret(self, identifier: str) -> bytes:
        if identifier not in self._secrets:
            raise ValueError(f"Read Secret: Declare {identifier} on the Plugin Before Reading It")
        return (await self._host.secrets.get(identifier)).secret

    async def settings(self) -> dict[str, "operations.SettingValue"]:
        from . import operations

        response = await self._host.plugins.settings.list()
        values = {}
        for key, value in response.value.values.items():
            kind = value.WhichOneof("value")
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


class StreamSubscription[T: Message](AsyncIterator[StreamRow[T] | Gap], Protocol):
    __type_params__: ClassVar[tuple[TypeVar, ...]]

    async def close(self) -> None: ...


class BoundOperation[RequestT: Payload, ResponseT: Payload]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]

    def __init__(
        self, transport: plugin.Context | dokime_sdk._generated.client.Client, operation: "operations.Operation[RequestT, ResponseT]"
    ) -> None:
        self._transport = transport
        self._operation = operation

    async def __call__(self, request: RequestT, *, policy: "operations.OperationPolicy | None" = None) -> ResponseT:
        wire = self._operation._definition()
        encoded = pack(self._operation.request, request)
        if isinstance(self._transport, dokime_sdk._generated.client.Client):
            if policy is not None:
                raise refused(
                    "invocations.create",
                    "The Host Invocation Contract Has No Operation Policy Override",
                    "Publish the Product Invocation Policy Contract and Regenerate Bindings",
                )
            observed = await call("operations.get", self._transport.operations.get(wire.id))
            if observed.contract.fingerprint != wire.fingerprint or observed.contract.visibility != VISIBILITY_PUBLIC:
                raise invalid("operations.get", "The Public Operation Contract Differs from the Declared Contract")
            payload = await call(
                "invocations.create", self._transport.invocations.create(wire.id, wire.fingerprint, encoded.SerializeToString())
            )
            response = message_type(self._operation.response).FromString(payload)
        else:
            try:
                response = await self._transport.call(wire, encoded, policy=policy._definition() if policy is not None else None)
            except harmos.FailureError as error:
                raise FailureError(project_failure(error.value)) from error
            except ValueError as error:
                raise FailureError(
                    Failure(
                        "Refused",
                        "invalid_operation_contract",
                        str(error),
                        "Import the Matching Typed Operation Reference Before Constructing the Plugin, Then Register It Again",
                        wire.id,
                    )
                ) from error
        return unpack(self._operation.response, response)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
