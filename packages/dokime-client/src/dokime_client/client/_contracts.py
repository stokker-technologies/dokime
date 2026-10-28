# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Construct Product Messages From Shared Generated Operation Descriptors"""

from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, cast

from google.protobuf.descriptor import Descriptor
from google.protobuf.message import Message
from google.protobuf.message_factory import GetMessageClass

from ..operation import Operation
from .errors import ProtocolError

if TYPE_CHECKING:
    from google._upb._message import Descriptor as UpbDescriptor


def operation(service: str, method: str) -> Operation:
    import dokime_client

    path = f"/dokime.v1.{service}/{method}"
    entries = [item for item in vars(dokime_client).values() if isinstance(item, Operation) and item.grpc_path == path]
    if len(entries) != 1:
        raise ProtocolError(f"the prepared operation catalog does not contain {path}")
    return entries[0]


def request(service: str, method: str, **fields: Any) -> Message:
    return construct(operation(service, method).request.DESCRIPTOR, fields)


def nested(service: str, method: str, path: str, *, direction: str = "request", **fields: Any) -> Message:
    descriptor = getattr(operation(service, method), direction).DESCRIPTOR
    for name in path.split("."):
        field = descriptor.fields_by_name.get(name)
        if field is None or field.message_type is None:
            raise ProtocolError(f"the operation has no message field {path}")
        descriptor = field.message_type
    return construct(descriptor, fields)


def construct(descriptor: Descriptor | UpbDescriptor, fields: Mapping[str, Any]) -> Message:
    # Protobuf's Native Descriptor Implements The Same Factory Contract
    result = GetMessageClass(cast(Descriptor, descriptor))()
    for name, value in fields.items():
        if value is None:
            continue
        field = descriptor.fields_by_name.get(name)
        if field is None:
            raise ValueError(f"unknown message field {name}")
        if field.is_repeated:
            container = getattr(result, name)
            if field.message_type is None:
                container.extend(value)
            else:
                for item in value:
                    container.add().CopyFrom(construct(field.message_type, values(item)))
        elif field.message_type is not None:
            getattr(result, name).CopyFrom(construct(field.message_type, values(value)))
        else:
            setattr(result, name, value)
    return result


def values(message: Message | Mapping[str, Any]) -> Mapping[str, Any]:
    if isinstance(message, Mapping):
        return message
    if not isinstance(message, Message):
        raise TypeError("a canonical message or named field mapping is required")
    return {field.name: value for field, value in message.ListFields()}


def copy(target: Message, source: Message | Mapping[str, Any]) -> None:
    target.CopyFrom(construct(target.DESCRIPTOR, values(source)))


def equal(message: Message, other: Message | Mapping[str, Any]) -> bool:
    return message == construct(message.DESCRIPTOR, values(other))


def variant(message: Message) -> str:
    names = [oneof.name for oneof in message.DESCRIPTOR.oneofs if oneof.name.startswith("_harmos_variant")]
    if len(names) != 1:
        raise ProtocolError("expected a canonical variant message")
    selected = message.WhichOneof(names[0])
    if selected is None:
        raise ProtocolError("canonical variant has no selected branch")
    return selected
