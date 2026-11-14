# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Generated Messages And Services, Discovered From The Built Descriptor Set."""

# === Imports === #

# Standard Library Imports
from importlib import import_module
from importlib.resources import files
from typing import Any

# Third Party Imports
from google.protobuf import descriptor_pb2, descriptor_pool, message_factory

# Local Imports

# Type Checking Imports

# === Public Exports === #

__all__ = ["descriptors", "enum_value", "message", "message_type", "services"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def descriptors() -> descriptor_pb2.FileDescriptorSet:
    return descriptor_pb2.FileDescriptorSet.FromString(files(__package__).joinpath("surface.pb").read_bytes())


def services() -> dict[str, type]:
    result = {}
    for definition in descriptors().file:
        if not definition.package.startswith("dokime.client.v1"):
            continue
        module = import_module(definition.name.removesuffix(".proto").replace("/", ".") + "_pb2_grpc")
        for service in definition.service:
            result[f"{definition.package}.{service.name}"] = getattr(module, f"{service.name}Stub")
    return result


def message_type(name: str) -> Any:
    qualified = name if "." in name else f"dokime.v1.{name}"
    return message_factory.GetMessageClass(descriptor_pool.Default().FindMessageTypeByName(qualified))


def message(name: str, /, **fields: Any) -> Any:
    return message_type(name)(**fields)


def enum_value(enum: str, name: str) -> int:
    return descriptor_pool.Default().FindEnumTypeByName(f"dokime.v1.{enum}").values_by_name[name].number


# === Private Functions === #

for _definition in descriptors().file:
    if _definition.package.startswith("dokime."):
        import_module(_definition.name.removesuffix(".proto").replace("/", ".") + "_pb2")

# === Entry Point === #
