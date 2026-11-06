# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Construct Product Messages From Reflected Native Operation Descriptors"""

# === Imports === #

# Standard Library Imports
from __future__ import annotations

from collections.abc import Mapping
from types import SimpleNamespace
from typing import TYPE_CHECKING, Any, cast

# Third Party Imports
from google.protobuf.descriptor import Descriptor
from google.protobuf.message import Message
from google.protobuf.message_factory import GetMessageClass

# Local Imports
from .contracts import catalog, operation
from .errors import ProtocolError

# Type Checking Imports
if TYPE_CHECKING:
    from google._upb._message import Descriptor as UpbDescriptor

# === Public Exports === #

__all__ = ["catalog", "construct", "copy", "equal", "messages", "nested", "operation", "request", "values", "variant"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def messages(service: str, method: str) -> Any:
    """Private Constructors From A Reflected Native Descriptor File."""
    descriptor = operation(service, method).request.DESCRIPTOR.file
    return SimpleNamespace(**{name: GetMessageClass(item) for name, item in descriptor.message_types_by_name.items()})


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


# === Private Functions === #

# === Entry Point === #
