# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Canonical Document Messages From The Pinned Protobuf Descriptor Set."""

# === Imports === #

# Standard Library Imports
from importlib.resources import files
from typing import Any

# Third Party Imports
from google.protobuf import descriptor_pb2
from google.protobuf.descriptor_pool import DescriptorPool
from google.protobuf.message_factory import GetMessageClass

# Local Imports

# Type Checking Imports

# === Public Exports === #

__all__ = ["message", "enum_value"]

# === Constants === #

_POOL = DescriptorPool()
for _file in descriptor_pb2.FileDescriptorSet.FromString(files(__package__).joinpath("documents.pb").read_bytes()).file:
    _POOL.Add(_file)

# === Types === #

# === Classes === #

# === Public Functions === #


def message(name: str, **fields: Any) -> Any:
    """Construct A Generated dokime.v1 Message, Without A Native Extension."""
    return GetMessageClass(_POOL.FindMessageTypeByName(f"dokime.v1.{name}"))(**fields)


def enum_value(enum: str, name: str) -> int:
    return _POOL.FindEnumTypeByName(f"dokime.v1.{enum}").values_by_name[name].number


# === Private Functions === #

# === Entry Point === #
