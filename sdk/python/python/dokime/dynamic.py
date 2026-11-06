# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Python Message Views Of Rust Operation Contracts."""

# === Imports === #

# Standard Library Imports
import json
from collections.abc import Mapping
from typing import Any

# Third Party Imports
from google.protobuf import descriptor_pb2
from google.protobuf.descriptor_pool import DescriptorPool
from google.protobuf.message_factory import GetMessageClass

# Local Imports
from . import _native
from .operation import Operation

# Type Checking Imports

# === Public Exports === #

__all__ = ["from_contract"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def from_contract(contract: Mapping[str, Any], *, generation: int = 0) -> Operation:
    """Reflect In Rust; Python Holds Only Interpreter-Specific Message Classes."""
    encoded = json.dumps(dict(contract), separators=(",", ":"))
    descriptors = descriptor_pb2.FileDescriptorSet.FromString(_native.descriptors(encoded))
    pool = DescriptorPool()
    for file in descriptors.file:
        pool.Add(file)
    binding = contract["grpc"]
    service = pool.FindServiceByName(f"{binding['package']}.{binding['service']}")
    method = service.methods_by_name[binding["method"]]
    root = contract["schema"]["definitions"][contract["schema"]["root"]]
    directions = {field["name"] for field in root}
    input_stream = "input" in directions
    output_stream = "output" in directions

    def message(descriptor, field, streamed):
        if streamed:
            descriptor = descriptor.fields_by_name[field].message_type
        return GetMessageClass(descriptor)

    request = message(method.input_type, "request", input_stream)
    response = message(method.output_type, "response", output_stream)
    mode = "bidi" if input_stream and output_stream else "client_stream" if input_stream else "server_stream" if output_stream else "unary"
    return Operation(
        id=contract["operation"]["id"],
        generation=generation,
        mode=mode,
        codec="protobuf",
        request=request,
        response=response,
        input=message(method.input_type, "item", True) if input_stream else None,
        output=message(method.output_type, "item", True) if output_stream else None,
        grpc_path=f"/{service.full_name}/{method.name}",
        rpc_request=GetMessageClass(method.input_type),
        rpc_response=GetMessageClass(method.output_type),
        run_request=GetMessageClass(service.methods_by_name[f"Run{method.name}"].input_type),
        run_response=GetMessageClass(service.methods_by_name[f"Run{method.name}"].output_type),
        contract=encoded,
        fingerprint=contract["fingerprint"],
    )


# === Private Functions === #

# === Entry Point === #
