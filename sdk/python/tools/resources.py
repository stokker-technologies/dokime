# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Python Bindings From The Authoritative Protocol Sources."""

# === Imports === #

import keyword
from pathlib import Path
from typing import TypedDict

from google.protobuf.descriptor_pb2 import FileDescriptorSet

# === Types === #


class ResourceMethod(TypedDict):
    path: list[str]
    request: str
    response: str
    package: str
    service: str
    method: str
    server_streaming: bool


# === Public Functions === #


def resources(target: Path, descriptors: FileDescriptorSet, table: list[ResourceMethod]) -> None:
    """Render Typed Client Resources Without Dynamic Public Attributes."""
    owners = {}
    for definition in descriptors.file:
        module = definition.name.removesuffix(".proto").replace("/", ".") + "_pb2"
        for message in definition.message_type:
            owners[f"{definition.package}.{message.name}"] = (module, message.name)
    modules = sorted({owners[item[field]][0] for item in table for field in ("request", "response")})
    aliases = {module: f"wire_{index}" for index, module in enumerate(modules)}
    nodes = {(): {}}
    for item in table:
        path = tuple(item["path"])
        for depth in range(1, len(path)):
            nodes.setdefault(path[:depth], {})
        nodes[path[:-1]][path[-1]] = item

    def name(path: tuple[str, ...]) -> str:
        return "".join(part.title().replace("_", "") for part in path) + "Resource"

    def message_type(qualified: str) -> str:
        module, message = owners[qualified]
        return f"{aliases[module]}.{message}"

    lines = [
        '"""Typed Resources Generated From the Frozen Descriptor Table."""',
        "from collections.abc import AsyncIterator",
        "from collections.abc import Awaitable",
        "from typing import Protocol, cast",
        "from google.protobuf.message import Message",
        "from google.protobuf.descriptor import MethodDescriptor",
        "from google.protobuf import descriptor_pool",
    ]
    lines.extend(f"import {module} as {aliases[module]}" for module in modules)
    lines.extend(
        [
            "",
            "class Caller(Protocol):",
            "    def call(self, request: Message, *, method: MethodDescriptor, timeout: float | None = None) -> Awaitable[Message] | AsyncIterator[Message]: ...",
        ]
    )
    for path, methods in sorted(nodes.items(), key=lambda item: (-len(item[0]), item[0])):
        children = sorted(child for child in nodes if len(child) == len(path) + 1 and child[:-1] == path)
        lines.extend(["", f"class {name(path)}:", "    def __init__(self, client: Caller) -> None:", "        self._client = client"])
        lines.extend(f"        self.{child[-1]} = {name(child)}(client)" for child in children)
        for leaf, method in sorted(methods.items()):
            leaf = leaf + "_" if keyword.iskeyword(leaf) else leaf
            request = message_type(method["request"])
            response = message_type(method["response"])
            stream = method["server_streaming"]
            result = f"AsyncIterator[{response}]" if stream else response
            prefix = "" if stream else "async "
            wait = "" if stream else "await "
            service = method["package"] + "." + method["service"]
            lines.extend(
                [
                    "",
                    f"    {prefix}def {leaf}(self, request: {request}, *, timeout: float | None = None) -> {result}:",
                    f'        method = descriptor_pool.Default().FindServiceByName("{service}").methods_by_name["{method["method"]}"]',
                    f"        return {wait}cast({result if stream else f'Awaitable[{response}]'}, self._client.call(request, method=method, timeout=timeout))",
                ]
            )
    target.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
