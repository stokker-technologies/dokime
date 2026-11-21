# ***
#
# Thermal Example
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Protobuf Row Schemas Shared by the Thermal Fixture and Client."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from google.protobuf.descriptor_pb2 import DescriptorProto, FieldDescriptorProto, FileDescriptorProto
from google.protobuf.descriptor_pool import Default
from google.protobuf.message_factory import GetMessageClass

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

DEFINITION = FileDescriptorProto(
    name="thermal/rows.proto",
    package="thermal",
    syntax="proto3",
    message_type=[
        DescriptorProto(name="Temperature", field=[FieldDescriptorProto(name="celsius", number=1, type=FieldDescriptorProto.TYPE_DOUBLE)]),
        DescriptorProto(name="Counter", field=[FieldDescriptorProto(name="value", number=1, type=FieldDescriptorProto.TYPE_UINT64)]),
        DescriptorProto(
            name="CounterColumns",
            field=[
                FieldDescriptorProto(
                    name="value",
                    number=1,
                    type=FieldDescriptorProto.TYPE_UINT64,
                    label=FieldDescriptorProto.LABEL_REPEATED,
                )
            ],
        ),
        DescriptorProto(name="Door", field=[FieldDescriptorProto(name="closed", number=1, type=FieldDescriptorProto.TYPE_BOOL)]),
    ],
)
POOL = Default()
POOL.Add(DEFINITION)


# === Types === #

# === Classes === #

Temperature = GetMessageClass(POOL.FindMessageTypeByName("thermal.Temperature"))
Counter = GetMessageClass(POOL.FindMessageTypeByName("thermal.Counter"))
CounterColumns = GetMessageClass(POOL.FindMessageTypeByName("thermal.CounterColumns"))
Door = GetMessageClass(POOL.FindMessageTypeByName("thermal.Door"))


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
