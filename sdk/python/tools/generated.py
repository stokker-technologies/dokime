# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Check The Installed Wheel Against Every Authoritative Proto And RPC."""

# === Imports === #

# Standard Library Imports
import hashlib
import json
from difflib import unified_diff
from importlib.resources import files
from pathlib import Path

# Third Party Imports
from google.protobuf import descriptor_pb2, descriptor_pool

# Local Imports
from dokime.v1 import descriptors, message_type, services

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class RecordingChannel:
    def __init__(self):
        self.methods = {}

    def __getattr__(self, cardinality):
        def register(path, request_serializer, response_deserializer, **options):
            self.methods[path] = (cardinality, request_serializer, response_deserializer)

        return register


# === Public Functions === #


def main():
    root = Path(__file__).resolve().parents[3]
    sources = root / "proto"
    manifest = json.loads(files("dokime.v1").joinpath("sources.json").read_text())
    expected = {path.relative_to(sources).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest() for path in sources.rglob("*.proto")}
    if manifest != expected:
        raise ValueError("Installed generated surface differs from proto/; rebuild and reinstall the SDK wheel")
    channel = RecordingChannel()
    for stub in services().values():
        stub(channel)
    methods = {}
    count = 0
    for definition in descriptors().file:
        if not definition.package.startswith(("dokime.", "harmos.")):
            continue
        runtime = descriptor_pool.Default().FindFileByName(definition.name)
        actual_definition = descriptor_pb2.FileDescriptorProto()
        runtime.CopyToProto(actual_definition)
        pending = [(node, actual_definition.package) for node in actual_definition.message_type]
        while pending:
            node, parent = pending.pop()
            name = f"{parent}.{node.name}"
            runtime_message = descriptor_pool.Default().FindMessageTypeByName(name)
            for field in node.field:
                field.json_name = runtime_message.fields_by_name[field.name].json_name
            pending.extend((child, name) for child in node.nested_type)
        if actual_definition != definition:
            difference = "\n".join(
                unified_diff(
                    str(definition).splitlines(),
                    str(actual_definition).splitlines(),
                    fromfile="Build Descriptor",
                    tofile="Runtime Descriptor",
                )
            )
            raise ValueError(f"Generated message descriptors differ: {definition.name}\n{difference}")
        if not definition.package.startswith("dokime."):
            continue
        for name in runtime.message_types_by_name:
            message_type(f"{definition.package}.{name}")().SerializeToString()
            count += 1
        for service in definition.service:
            for method in service.method:
                path = f"/dokime.v1.{service.name}/{method.name}"
                cardinality = ("stream" if method.client_streaming else "unary") + "_" + ("stream" if method.server_streaming else "unary")
                methods[path] = cardinality
                actual, serialize, deserialize = channel.methods[path]
                if actual != cardinality:
                    raise ValueError(f"Generated method cardinality differs: {path}")
                serialize(message_type(method.input_type.removeprefix("."))())
                reply = deserialize(b"")
                if reply.DESCRIPTOR.full_name != method.output_type.removeprefix("."):
                    raise ValueError(f"Generated reply differs: {path}")
    if set(methods) != set(channel.methods):
        raise ValueError("Generated stubs and service descriptors differ")
    print(f"Generated surface matches {len(expected)} protos: {count} messages, {len(methods)} RPCs")
    if not methods:
        print("Host integration prerequisite: proto/dokime/v1 currently declares no client services")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
