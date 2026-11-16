# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Check Protocol Caller Contracts"""

# === Imports === #

# Standard Library Imports
import ast
import re
import shutil
import subprocess
import tempfile
from collections.abc import Iterator
from pathlib import Path
from typing import Any, cast

# Third Party Imports
from google.protobuf import message_factory
from google.protobuf.descriptor import EnumDescriptor, FieldDescriptor
from google.protobuf.descriptor_pb2 import FileDescriptorSet, MethodDescriptorProto
from google.protobuf.descriptor_pool import DescriptorPool

# Local Imports
from ..process import spawn_flags

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def caller_options(descriptors: FileDescriptorSet) -> dict[str, str]:
    """Read Custom Options Without Depending on Generated SDK Imports."""
    pool = _descriptor_pool(descriptors)
    return _caller_options(descriptors, pool)


def narrow_callers(descriptors: FileDescriptorSet, overrides: dict[str, str]) -> None:
    """Apply Product Assembly Restrictions to the Generator's Descriptor Copy."""
    pool = _descriptor_pool(descriptors)
    declared = _caller_options(descriptors, pool)
    audiences = {"CLIENTS": {"CLIENTS"}, "PLUGINS": {"PLUGINS"}, "ALL": {"CLIENTS", "PLUGINS"}}
    for identity, selected in overrides.items():
        original = declared.get(identity)
        if original is None or selected not in audiences:
            raise ValueError(f"Narrow {identity}: Select a Declared Caller Subset")
        if not audiences[selected] <= audiences[original]:
            raise ValueError(f"Narrow {identity}: Select a Declared Caller Subset")

    # The Descriptor Owns the Extension Number and Enum Vocabulary.
    extension, callers = _caller_extension(pool)
    options_descriptor = pool.FindMessageTypeByName("google.protobuf.MethodOptions")
    options_type = message_factory.GetMessageClass(options_descriptor)
    for identity, method in _host_methods(descriptors):
        selected = overrides.get(identity)
        if selected is None:
            continue
        options = cast(Any, options_type.FromString(method.options.SerializeToString()))
        options.Extensions[extension] = callers.values_by_name[selected].number
        method.options.ParseFromString(options.SerializeToString())


def main() -> None:
    root = Path(__file__).resolve().parents[5]
    check_public_surfaces(root)

    scratch = Path(tempfile.mkdtemp())
    try:
        output = scratch / "descriptors.bin"
        command = ["buf", "build", "--as-file-descriptor-set", "-o", str(output)]
        subprocess.run(command, check=True, creationflags=spawn_flags())
        descriptors = FileDescriptorSet.FromString(output.read_bytes())
        methods = caller_options(descriptors)
        print(f"Protocol Callers: {len(methods)} Annotated Operations")
    finally:
        shutil.rmtree(scratch)


def check_public_surfaces(root: Path) -> None:
    """Keep Protocol Implementation Names Out of Consumer Entry Points.

    Allow internal generated directories (detail, _generated, _wire), canonical
    gRPC method paths. The separate ci:docs task checks guides.
    """
    wire = re.compile(r"\b_wire\b|\w+_wire\b|\b(?:Harmos|Dokime)\.Internal\b|\b(?:[A-Za-z_]\w*(?:\.|::))+v\d+\b|[/\\]v\d+[/\\]")
    grpc_path = re.compile(r"/?[a-zA-Z_]\w*(?:\.\w+)*\.v\d+(?:\.\w+)*/[A-Za-z_]\w*")
    candidates = sorted((root / "sdk/cpp/include").rglob("*.hpp"))
    candidates += sorted((root / "sdk/cpp/examples").glob("*/generated/*.hpp"))
    for path in (root / "sdk/python/src/dokime").rglob("*"):
        if path.suffix not in {".py", ".pyi"}:
            continue
        if any(part.startswith("_") for part in path.relative_to(root / "sdk/python/src/dokime").parts[:-1]):
            continue
        if path.name.startswith("_") and path.name != "__init__.py":
            continue
        candidates.append(path)
    findings = []
    for path in candidates:
        relative = path.relative_to(root)
        if any(part in {"detail", "_generated", "_wire"} for part in relative.parts):
            continue
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            line = grpc_path.sub("", line)
            if wire.search(line) or re.search(r"\bv\d+\b", line):
                findings.append(f"{relative.as_posix()}:{number}: Public Surface Names a Wire Package")
    exports: dict[str, Path] = {}
    for name in ("__init__.py", "client.py", "plugin/__init__.py"):
        path = root / "sdk/python/src/dokime" / name
        if not path.is_file():
            continue
        for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
            if not isinstance(node, ast.Assign) or not any(
                isinstance(target, ast.Name) and target.id == "__all__" for target in node.targets
            ):
                continue
            if not isinstance(node.value, (ast.List, ast.Tuple)):
                raise TypeError(f"Check Public Exports: {path} Must Declare Its Names Explicitly")
            for item in node.value.elts:
                if not isinstance(item, ast.Constant) or not isinstance(item.value, str):
                    raise TypeError(f"Check Public Exports: {path} Must Declare String Names")
                if item.value in exports:
                    findings.append(f"{path.relative_to(root)}: Duplicate Public Export {item.value}")
                exports[item.value] = path
    for path in sorted((root / "sdk/csharp/src/Dokime").rglob("*.cs")):
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if re.search(r"^namespace .*\.V\d+\b", line):
                findings.append(f"{path.relative_to(root)}:{number}: Public Wire Namespace")
    for path in sorted((root / "sdk/rust/src").rglob("*.rs")):
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if re.search(r"\bpub\s+mod\s+(?:wire|v\d+)\b", line):
                findings.append(f"{path.relative_to(root).as_posix()}:{number}: Public Wire Module")
    if findings:
        raise ValueError("Check Public Surfaces:\n" + "\n".join(findings))
    print("Public Surfaces: Headers, Stubs and Exports Passed")


# === Private Functions === #


def _descriptor_pool(descriptors: FileDescriptorSet) -> DescriptorPool:
    pool = DescriptorPool()
    pending = {}
    for file in descriptors.file:
        pending[file.name] = file
    while pending:
        ready = []
        for file in pending.values():
            if all(name not in pending for name in file.dependency):
                ready.append(file)
        if not ready:
            raise ValueError("Protocol Callers: Resolve Descriptor Dependencies Before Generation")
        for file in ready:
            pool.AddSerializedFile(file.SerializeToString())
            del pending[file.name]
    return pool


def _host_methods(descriptors: FileDescriptorSet) -> Iterator[tuple[str, MethodDescriptorProto]]:
    for file in descriptors.file:
        if "host" not in Path(file.name).parts[:-1]:
            continue
        for service in file.service:
            for method in service.method:
                identity = f"{file.package}.{service.name}/{method.name}"
                yield identity, method


def _caller_extension(pool: DescriptorPool) -> tuple[FieldDescriptor, EnumDescriptor]:
    try:
        extension = pool.FindExtensionByName("harmos.model.v1.callers")
    except KeyError as error:
        raise ValueError("Protocol Callers: Declare the Library Callers Option") from error
    if extension.enum_type is None:
        raise ValueError("Protocol Callers: Declare an Enum Option")
    return extension, extension.enum_type


def _caller_options(descriptors: FileDescriptorSet, pool: DescriptorPool) -> dict[str, str]:
    result: dict[str, str] = {}
    methods = list(_host_methods(descriptors))
    if not methods:
        return result
    extension, callers = _caller_extension(pool)
    options_descriptor = pool.FindMessageTypeByName("google.protobuf.MethodOptions")
    options_type = message_factory.GetMessageClass(options_descriptor)
    for identity, method in methods:
        # Protobuf Custom Extensions Are a Dynamic Boundary.
        options = cast(Any, options_type.FromString(method.options.SerializeToString()))
        allowed = callers.values_by_number.get(options.Extensions[extension])
        declared = options.HasExtension(extension)
        supported = allowed is not None and allowed.name in {"CLIENTS", "PLUGINS", "ALL"}
        if not declared or not supported:
            raise ValueError(f"Protocol Callers: {identity} Requires CLIENTS, PLUGINS or ALL")
        result[identity] = allowed.name
    return result


# === Entry Point === #

if __name__ == "__main__":
    main()
