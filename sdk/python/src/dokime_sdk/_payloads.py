# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Plain and Structured Payloads at the Protobuf Boundary."""

# === Imports === #

# Standard Library Imports

from typing import Any, cast

# Third Party Imports
from google.protobuf.message import Message
from google.protobuf.wrappers_pb2 import BoolValue, DoubleValue, Int64Value, StringValue

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

type Payload = Message | bool | str | int | float

# === Classes === #

# === Public Functions === #


def message_type[T: Payload](row_type: type[T]) -> type[Message]:
    wrappers = {bool: BoolValue, str: StringValue, int: Int64Value, float: DoubleValue}
    wrapper = wrappers.get(row_type)
    if wrapper is not None:
        return wrapper
    if isinstance(row_type, type) and issubclass(row_type, Message):
        return row_type
    raise TypeError("Declare Payload: Supply bool, str, int, float or a Protobuf Message")


def pack[T: Payload](row_type: type[T], value: T) -> Message:
    if type(value) is not row_type and not (isinstance(value, Message) and isinstance(value, row_type)):
        raise TypeError(f"Encode Payload: Supply a {row_type.__name__} Value")
    if isinstance(value, Message):
        return value
    return message_type(row_type)(value=value)


def unpack[T: Payload](row_type: type[T], message: Message) -> T:
    if not isinstance(message, message_type(row_type)):
        raise TypeError(f"Decode Payload: Supply the Declared {row_type.__name__} Message")
    value = message if issubclass(row_type, Message) else cast(Any, message).value
    return cast(T, value)


# === Private Functions === #

# === Entry Point === #
