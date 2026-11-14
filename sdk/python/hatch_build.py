# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Python Bindings From The Authoritative Protocol Sources."""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import keyword
import tempfile
from pathlib import Path
from typing import Any

# Third Party Imports
import grpc_tools
from google.protobuf.descriptor_pb2 import FileDescriptorSet
from grpc_tools import protoc
from hatchling.builders.hooks.plugin.interface import BuildHookInterface

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CustomBuildHook(BuildHookInterface):
    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        root = Path(self.root)
        sources = root / "proto" if (root / "proto").is_dir() else root.parents[1] / "proto"
        self.scratch = tempfile.TemporaryDirectory(prefix="dokime-protobuf-")
        output = Path(self.scratch.name)
        definitions = sorted(sources.glob("dokime/**/*.proto"))
        if not definitions:
            raise ValueError("Build Python SDK: Supply the Authoritative Protocol Sources")
        arguments = [
            "protoc",
            f"-I{sources}",
            f"-I{Path(grpc_tools.__file__).parent / '_proto'}",
            f"--python_out={output}",
            f"--mypy_out=quiet:{output}",
            f"--mypy_grpc_out=quiet:{output}",
            f"--grpc_python_out={output}",
            f"--descriptor_set_out={output / 'surface.pb'}",
            "--include_imports",
            *map(str, definitions),
        ]
        if protoc.main(arguments):
            raise RuntimeError("Dokime protocol generation failed")
        resource_arguments = [
            "protoc",
            f"-I{sources}",
            f"-I{Path(grpc_tools.__file__).parent / '_proto'}",
            f"--descriptor_set_out={output / 'resources.pb'}",
            "--include_imports",
            *map(str, sorted(sources.glob("**/*.proto"))),
        ]
        if protoc.main(resource_arguments):
            raise RuntimeError("Build Python Resources: Protocol Descriptor Generation Failed")
        descriptor_set = FileDescriptorSet.FromString((output / "resources.pb").read_bytes())
        table = json.loads((root / "src/dokime/resources.json").read_text(encoding="utf-8"))
        resource_package = output / "_dokime_resources"
        resource_package.mkdir()
        resources(resource_package / "__init__.py", descriptor_set, table)
        (resource_package / "py.typed").touch()
        build_data["force_include"][str(resource_package)] = "_dokime_resources"
        package = output / "dokime/v1"
        (output / "surface.pb").replace(package / "surface.pb")
        manifest = {
            path.relative_to(sources).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest() for path in sources.rglob("*.proto")
        }
        (package / "sources.json").write_text(json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8")
        build_data["force_include"][str(output / "dokime")] = "dokime"
        for name in ("LICENSE", "NOTICE"):
            source = root / name if (root / name).is_file() else root.parents[1] / name
            build_data["force_include"][str(source)] = f"dokime/{name}"
        build_data["pure_python"] = True
        build_data["infer_tag"] = False
        build_data["tag"] = "py3-none-any"

    def finalize(self, version: str, build_data: dict[str, Any], artifact_path: str) -> None:
        self.scratch.cleanup()


# === Public Functions === #


def resources(target: Path, descriptors: FileDescriptorSet, table: list[dict[str, Any]]) -> None:
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
        "from typing import Any",
        "from google.protobuf import descriptor_pool",
    ]
    lines.extend(f"import {module} as {aliases[module]}" for module in modules)
    stubs = list(lines)
    for path, methods in sorted(nodes.items(), key=lambda item: (-len(item[0]), item[0])):
        children = sorted(child for child in nodes if len(child) == len(path) + 1 and child[:-1] == path)
        lines.extend(["", f"class {name(path)}:", "    def __init__(self, client: Any) -> None:", "        self._client = client"])
        lines.extend(f"        self.{child[-1]} = {name(child)}(client)" for child in children)
        stubs.extend(["", f"class {name(path)}:", "    _client: Any", "    def __init__(self, client: Any) -> None: ..."])
        stubs.extend(f"    {child[-1]}: {name(child)}" for child in children)
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
                    f"        return {wait}self._client.call(request, method=method, timeout=timeout)",
                ]
            )
            stubs.extend(["", f"    {prefix}def {leaf}(self, request: {request}, *, timeout: float | None = None) -> {result}: ..."])
    target.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    target.with_suffix(".pyi").write_text("\n".join(stubs) + "\n", encoding="utf-8", newline="\n")


# === Private Functions === #

# === Entry Point === #
