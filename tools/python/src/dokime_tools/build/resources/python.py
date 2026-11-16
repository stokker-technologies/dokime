# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate Python Resource Surfaces."""

# === Imports === #

# Standard Library Imports
import keyword
import re
from pathlib import Path

# isort: split

# Third Party Imports
from google.protobuf.descriptor_pb2 import DescriptorProto, FileDescriptorSet

# isort: split

# Local Imports
from ... import policy
from . import schema

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def exports(target: Path, descriptors: FileDescriptorSet) -> None:
    """Generate Private Import Tables for the Three Public Barrel Points."""
    public_names = schema.public_names(descriptors)
    methods = schema.table(descriptors)
    audiences = schema.payload_audiences(descriptors)
    requests = set()
    for requests_method in methods:
        if requests_method["face"] == "plugin":
            continue
        requests.add(requests_method["request"])

    carriers = {
        "Request",
        "Response",
        "ChildRequest",
        "ChildResponse",
        "ChildCancel",
        "ChildWatch",
        "ChildWatchResponse",
        "ChildWatchCancel",
    }
    groups: dict[str, list[tuple[str, str, str]]] = {"shared": [], "client_types": [], "plugin_types": []}
    for file in descriptors.file:
        if not file.package.startswith(("dokime.", "harmos.")):
            continue
        if file.name.endswith("guest.proto"):
            continue
        if file.package.startswith("harmos.plugin."):
            continue
        module = file.name.removesuffix(".proto").replace("/", ".").replace("dokime.", "dokime._wire.", 1) + "_pb2"
        for definition in [*file.message_type, *file.enum_type]:
            if f"{file.package}.{definition.name}" in requests:
                continue
            if definition.name in carriers:
                continue
            name = "FailureValue" if definition.name == "Failure" else schema.public_name(f"{file.package}.{definition.name}", public_names)
            groups_for_type = (
                {"plugin_types"}
                if ".plugin." in file.package
                else {"shared"}
                if ".model." in file.package or len(audiences.get(f"{file.package}.{definition.name}", set())) > 1
                else {group + "_types" for group in audiences.get(f"{file.package}.{definition.name}", {"client"})}
            )
            for group in groups_for_type:
                source = module
                original = definition.name
                if file.package.startswith("harmos."):
                    library_shared = ".model." in file.package or len(audiences.get(f"{file.package}.{definition.name}", set())) > 1
                    source = "harmos" if library_shared else "harmos.client" if group == "client_types" else "harmos.plugin"
                    original = "FailureValue" if definition.name == "Failure" else definition.name
                groups[group].append((source, original, name))
        for definition in file.enum_type:
            groups_for_type = (
                {"plugin_types"}
                if ".plugin." in file.package
                else {"shared"}
                if ".model." in file.package or len(audiences.get(f"{file.package}.{definition.name}", set())) > 1
                else {group + "_types" for group in audiences.get(f"{file.package}.{definition.name}", {"client"})}
            )
            for group in groups_for_type:
                source = (
                    module
                    if file.package.startswith("dokime.")
                    else "harmos"
                    if group == "shared"
                    else "harmos.client"
                    if group == "client_types"
                    else "harmos.plugin"
                )
                groups[group].extend((source, value.name, value.name) for value in definition.value)
    messages = set()
    for messages_file in descriptors.file:
        if not (messages_file.package.startswith("dokime.")):
            continue
        for messages_definition in messages_file.message_type:
            messages.add(
                (
                    messages_file.name.removesuffix(".proto").replace("/", ".").replace("dokime.", "dokime._wire.", 1) + "_pb2",
                    messages_definition.name,
                )
            )

    target.mkdir(parents=True, exist_ok=True)
    for group, entries in groups.items():
        lines = ["# @generated from protocol descriptors; do not edit."]
        lines += [f"from {module} import {original} as {name}" for module, original, name in sorted(entries)]
        public_module = "dokime" if group == "shared" else "dokime." + group.removesuffix("_types")
        for module, original, name in sorted(entries):
            if (module, original) in messages:
                lines += [f'{name}.__module__ = "{public_module}"', f'{name}.__qualname__ = "{name}"']
        lines += ["", "__all__ = [", *[f"    {name!r}," for name in sorted(name for _, _, name in entries)], "]", ""]
        (target / f"{group}.py").write_text("\n".join(lines), encoding="utf-8", newline="\n")


def render(
    target: Path, methods: list[schema.Method], descriptors: FileDescriptorSet, *, module_prefixes: dict[str, str] | None = None
) -> None:
    """Generate Content Arguments and Exactly-One-Field Response Projection."""
    public_names = schema.public_names(descriptors)
    selected: list[schema.Method] = []
    for method in methods:
        method = method.copy()
        if method["request"] == "harmos.host.v1.plugins.SettingsListRequest":
            method["fields"] = [
                schema.Field(**{**field, "optional": False}) if field["name"] == "name" else field for field in method["fields"]
            ]
        selected.append(method)
    methods = selected
    nodes: set[tuple[str, ...]] = {()}
    leaves: set[tuple[str, ...]] = set()
    for method in methods:
        path = tuple(method["path"])
        if method["face"] != "client" or path in leaves:
            raise ValueError(f"Generate Python Client {'.'.join(path)}: Invalid or Duplicate Resource")
        leaves.add(path)
        if method["client_streaming"]:
            raise ValueError(f"Generate Python Client {method['method']}: Client Streaming Requires a Managed Contract")
        nodes.update(tuple(method["path"][:length]) for length in range(1, len(method["path"])))
    if leaves & nodes:
        raise ValueError("Generate Python Client: Resource Collides With a Method")
    names = {}
    for names_node in nodes:
        names[names_node] = "ClientRoot" if not names_node else "".join(part.title().replace("_", "") for part in names_node) + "Resource"

    composed = any(method["owner"] != "harmos" for method in methods)
    library = policy.root() / "crates/dokime-protocol/src/descriptors.bin"
    library_methods = schema.table(FileDescriptorSet.FromString(library.read_bytes()))
    library_leaves = set()
    for library_leaves_method in library_methods:
        if library_leaves_method["face"] != "client":
            continue
        library_leaves.add(tuple(library_leaves_method["path"]))

    narrowed = composed or not library_leaves <= leaves
    library_nodes: set[tuple[str, ...]] = {()}
    for method in methods:
        if method["owner"] == "harmos":
            library_nodes.update(tuple(method["path"][:length]) for length in range(1, len(method["path"])))
    messages: dict[str, tuple[str, str]] = {}

    def record(definitions: list[DescriptorProto], package: str, module: str, parent: str = "") -> None:
        for definition in definitions:
            name = parent + definition.name
            messages[f"{package}.{name}"] = (module, name)
            record(list(definition.nested_type), package, module, name + ".")

    for file in descriptors.file:
        module = file.name.removesuffix(".proto").replace("/", ".") + "_pb2"
        if module.startswith(("harmos.", "dokime.")):
            module = module.replace("harmos.", "harmos._wire.", 1).replace("dokime.", "dokime._wire.", 1)
        for prefix, replacement in (module_prefixes or {}).items():
            if module.startswith(prefix + "."):
                module = module.replace(prefix + ".", replacement + ".", 1)
        record(list(file.message_type), file.package, module)
        for enum in file.enum_type:
            messages[f"{file.package}.{enum.name}"] = (module, enum.name)
    used = set()
    for used_method in methods:
        for used_key in ("request", "response"):
            used.add(used_method[used_key])

    used.update(
        field["type"]
        for method in methods
        for field in [*method["fields"], *([method["payload_field"]] if method["payload_field"] else [])]
        if field["type"]
    )
    aliases = {full: schema.public_name(full, public_names) for full in used}
    for full in sorted(used):
        if sum(value == aliases[full] for value in aliases.values()) > 1:
            aliases[full] = (
                "".join(part.title().replace("_", "") for part in full.split(".")[:-1] if not re.fullmatch(r"v[0-9]+", part))
                + full.rsplit(".", 1)[-1]
            )

    def field_type(field: schema.Field) -> str:
        value = (
            aliases[field["type"]]
            if field["type"]
            else {1: "float", 2: "float", 8: "bool", 9: "str", 12: "bytes"}.get(field["kind"], "int")
        )
        return f"dict[str, {value}]" if field["map"] else f"Sequence[{value}]" if field["repeated"] else value

    lines = [
        "# @generated from protocol descriptors; do not edit.",
        '"""Typed Client Resources."""',
        "from collections.abc import AsyncGenerator, Sequence",
        "from uuid import uuid4",
        "from grpc import CallCredentials",
        "from grpc.aio import AioRpcError, Channel",
        "from harmos._client_transport import Metadata as CallMetadata, call_error, stream, unary",
    ]
    if narrowed:
        lines += ["from harmos._connection import Connection as LibraryConnection"]
    elif composed:
        lines += ["from harmos import Client as LibraryClient"]
        lines_items = []
        for lines_node in sorted(library_nodes):
            if not (lines_node):
                continue
            lines_items.append(f"from harmos.client import {names[lines_node]} as Library{names[lines_node]}")
        lines += lines_items
    for full in sorted(used):
        module, name = messages[full]
        if "." in name:
            parent, nested = name.split(".", 1)
            imported = "_" + parent + aliases[full]
            lines += [f"from {module} import {parent} as {imported}", f"{aliases[full]} = {imported}.{nested}"]
        else:
            lines += [f"from {module} import {name}" + (f" as {aliases[full]}" if aliases[full] != name else "")]

    def sort_key(value: tuple[str, ...]) -> tuple:
        return (-len(value), value)

    for node in sorted(nodes, key=sort_key):
        children = sorted(child for child in nodes if len(child) == len(node) + 1 and child[:-1] == node)
        parent = ""
        if narrowed:
            if not node:
                parent = "LibraryConnection"
        elif composed:
            if not node:
                parent = "LibraryClient"
            elif node in library_nodes:
                parent = "Library" + names[node]
        lines += ["", f"class {names[node]}" + (f"({parent})" if parent else "") + ":", "    def __init__(self, channel: Channel) -> None:"]
        if parent:
            lines += ["        super().__init__(channel)"]
        lines += ["        self._channel = channel"]
        storage = "_composed_" if composed else "_library_"
        lines += [f"        self.{storage}{child[-1]} = {names[child]}(channel)" for child in children]
        for child in children:
            lines += ["", "    @property", f"    def {child[-1]}(self) -> {names[child]}:", f"        return self.{storage}{child[-1]}"]
        for method in (method for method in methods if tuple(method["path"][:-1]) == node):
            request, response = (aliases[method[key]] for key in ("request", "response"))
            required = []
            for required_field in method["fields"]:
                if required_field["optional"]:
                    continue
                required.append(required_field)

            optional = []
            for optional_field in method["fields"]:
                if not (optional_field["optional"]):
                    continue
                optional.append(optional_field)

            parameters = [f"{field['name']}: {field_type(field)}" for field in required]
            parameters += ["*"]
            parameters += [f"{field['name']}: {field_type(field)} | None = None" for field in optional]
            parameters += [
                "call_metadata: CallMetadata | None = None",
                "timeout: float | None = None",
                "credentials: CallCredentials | None = None",
            ]
            payload = method["payload_field"]
            result = field_type(payload) if payload else response
            result = result.replace("Sequence[", "list[")
            stream_call = method["server_streaming"]
            returns = f"AsyncGenerator[{result}]" if stream_call else result
            lines += [
                "",
                f"    async def {method['path'][-1] + ('_' if keyword.iskeyword(method['path'][-1]) else '')}(self, {', '.join(parameters)}) -> {returns}:",
            ]
            values = []
            for values_field in method["fields"]:
                values.append(
                    f"{values_field['name']}="
                    + (
                        "request_id if request_id is not None else str(uuid4())"
                        if values_field["name"] == "request_id"
                        else values_field["name"]
                    )
                )

            lines += [f"        request = {request}({', '.join(values)})"]
            path = f"/{method['package']}.{method['service']}/{method['method']}"
            call = f"{'stream' if stream_call else 'unary'}(self._channel, {path!r}, request, {response}, metadata=call_metadata, timeout=timeout, credentials=credentials)"
            if stream_call:
                lines += [f"        call = {call}", "        try:", "            async for response in call:"]
                if payload and payload["kind"] == 11 and not payload["repeated"]:
                    lines += [
                        f'                if not response.HasField("{payload["name"]}"):',
                        f'                    raise ValueError("Read {method["service"]}.{method["method"]}: Missing Payload")',
                    ]
                value = f"response.{payload['name']}" if payload else "response"
                if payload and payload["repeated"]:
                    value = f"list({value})"
                lines += [
                    f"                yield {value}",
                    "        except AioRpcError as error:",
                    f"            raise call_error({'.'.join(method['path'])!r}, error) from error",
                    "        finally:",
                    "            call.cancel()",
                ]
            else:
                call = call.removesuffix(")") + f", action={'.'.join(method['path'])!r})"
                lines += [f"        response = await {call}"]
                if payload and payload["kind"] == 11 and not payload["repeated"]:
                    lines += [
                        f'        if not response.HasField("{payload["name"]}"):',
                        f'            raise ValueError("Read {method["service"]}.{method["method"]}: Missing Payload")',
                    ]
                value = f"response.{payload['name']}" if payload else "response"
                if payload and payload["repeated"]:
                    value = f"list({value})"
                lines += [f"        return {value}"]
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def host(target: Path, methods: list[schema.Method], descriptors: FileDescriptorSet) -> None:
    modules = {}
    for file in descriptors.file:
        module = file.name.removesuffix(".proto").replace("/", ".") + "_pb2"
        if module.startswith(("harmos.", "dokime.")):
            module = module.replace("harmos.", "harmos._wire.", 1).replace("dokime.", "dokime._wire.", 1)
        for definition in [*file.message_type, *file.enum_type]:
            modules[f"{file.package}.{definition.name}"] = module
    used = set()
    for used_method in methods:
        for used_key in ("request", "response"):
            used.add(used_method[used_key])

    used.update(
        field["type"]
        for method in methods
        for field in [*method["fields"], *([method["payload_field"]] if method["payload_field"] else [])]
        if field["type"]
    )

    def field_type(field: schema.Field) -> str:
        value = (
            field["type"].rsplit(".", 1)[-1]
            if field["type"]
            else {1: "float", 2: "float", 8: "bool", 9: "str", 12: "bytes"}.get(field["kind"], "int")
        )
        return f"dict[str, {value}]" if field["map"] else f"list[{value}]" if field["repeated"] else value

    nodes = sorted({tuple(method["path"][:length]) for method in methods for length in range(1, len(method["path"]))})
    names = {node: "Host" + "".join(part.title().replace("_", "") for part in node) + "Resource" for node in nodes}
    lines = [
        "# @generated from protocol descriptors; do not edit.",
        "from collections.abc import AsyncGenerator",
        "from enum import Enum",
        "from uuid import uuid4",
        "from typing import TYPE_CHECKING, cast",
        "from google.protobuf.message import Message",
        "from harmos._contracts import Operation",
        "from harmos._wire.model.v1.operation_pb2 import VISIBILITY_PUBLIC",
        "from harmos._wire.model.v1.policy_pb2 import OperationPolicy",
    ]
    lines += [f"from {modules[full]} import {full.rsplit('.', 1)[-1]}" for full in sorted(used)]
    lines += ["if TYPE_CHECKING:", "    from harmos._invocation import Context", "class HostOperation(Enum):"]
    if not methods:
        lines += ["    pass"]
    for method in methods:
        name = "_".join(method["path"]).upper()
        lines += [f'    {name} = "{method["package"]}.{method["service"]}/{method["method"]}"']
    for method in methods:
        name = "_".join(method["path"]).upper()
        q, r = (method[key].rsplit(".", 1)[-1] for key in ("request", "response"))
        lines += [
            f"_{name} = Operation(HostOperation.{name}.value, {q}, {r}, visibility=VISIBILITY_PUBLIC, watch={method['server_streaming']})"
        ]
    lines += ["def host_requirement(operation: HostOperation) -> Operation[Message, Message]:"]
    if methods:
        lines += ["    match operation:"]
    for method in methods:
        name = "_".join(method["path"]).upper()
        lines += [f"        case HostOperation.{name}:", f"            return cast(Operation[Message, Message], _{name})"]
    lines += [
        '    raise ValueError("Declare Host Requirement: Unknown Operation")',
        "class HostRoot:",
        "    def __init__(self, context: 'Context') -> None:",
    ]
    lines_items = []
    for lines_node in nodes:
        if len(lines_node) != 1:
            continue
        lines_items.append(f"        self.{lines_node[-1]} = {names[lines_node]}(context)")
    lines += lines_items
    if not nodes:
        lines += ["        pass"]
    for node in nodes:
        lines += [
            f"class {names[node]}:",
            "    def __init__(self, context: 'Context') -> None:",
            "        self._context = context",
        ]
        lines_items = []
        for lines_child in nodes:
            if lines_child[:-1] != node:
                continue
            lines_items.append(f"        self.{lines_child[-1]} = {names[lines_child]}(context)")
        lines += lines_items
        for method in (method for method in methods if tuple(method["path"][:-1]) == node):
            name = "_".join(method["path"]).upper()
            q, r = (method[key].rsplit(".", 1)[-1] for key in ("request", "response"))
            parameters = []
            for parameters_field in method["fields"]:
                if parameters_field["optional"]:
                    continue
                parameters.append(f"{parameters_field['name']}: {field_type(parameters_field)}")

            parameters += ["*"]
            parameters_items = []
            for parameters_field in method["fields"]:
                if not (parameters_field["optional"]):
                    continue
                parameters_items.append(f"{parameters_field['name']}: {field_type(parameters_field)} | None = None")
            parameters += parameters_items
            if method["server_streaming"]:
                parameters += ["max_item_bytes: int"]
            else:
                parameters += ["policy: OperationPolicy | None = None"]
            payload = method["payload_field"]
            result = field_type(payload) if payload else r
            if method["server_streaming"]:
                result = f"AsyncGenerator[{result}]"
            lines += [
                f"    async def {method['path'][-1] + ('_' if keyword.iskeyword(method['path'][-1]) else '')}(self{', ' if parameters else ''}{', '.join(parameters)}) -> {result}:"
            ]
            for group in sorted({field["oneof"] for field in method["fields"] if field["oneof"]}):
                variant_names = ", ".join(field["name"] for field in method["fields"] if field["oneof"] == group)
                lines += [
                    f"        if sum(value is not None for value in ({variant_names})) != 1:",
                    f'            raise ValueError("Choose Exactly One {group} Variant")',
                ]
            lines += [
                f"        request = {q}({', '.join(field['name'] + '=' + ('request_id if request_id is not None else str(uuid4())' if field['name'] == 'request_id' else field['name']) for field in method['fields'])})",
            ]
            indent = "        "
            if method["server_streaming"]:
                lines += [
                    f"        call = self._context.watch(_{name}, request, max_item_bytes=max_item_bytes)",
                    "        try:",
                    "            async for response in call:",
                ]
                indent = "                "
            else:
                lines += [f"        response = await self._context.call(_{name}, request, policy=policy)"]
            if payload and payload["kind"] == 11 and not payload["repeated"]:
                lines += [
                    f'{indent}if not response.HasField("{payload["name"]}"):',
                    f'{indent}    raise ValueError("Read {name}: Missing Payload")',
                ]
            value = f"response.{payload['name']}" if payload else "response"
            if payload and payload["repeated"]:
                value = f"list({value})"
            lines += [f"{indent}{'yield' if method['server_streaming'] else 'return'} {value}"]
            if method["server_streaming"]:
                lines += ["        finally:", "            await call.aclose()"]
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


# === Private Functions === #

# === Entry Point === #
