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
from pathlib import Path

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet
from harmos_tools.source.protocol import caller_options

# Local Imports
from ..process import spawn_flags

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


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
                raise TypeError(f"Check Public Exports: Declare Explicit Names in {path}")
            for item in node.value.elts:
                if not isinstance(item, ast.Constant) or not isinstance(item.value, str):
                    raise TypeError(f"Check Public Exports: Declare String Names in {path}")
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
        raise ValueError(
            "Check Public Surfaces: Update Public SDK Exports to Hide Wire Names and Remove Duplicate Exports\n" + "\n".join(findings)
        )
    print("Public Surfaces: Headers, Stubs and Exports Passed")


# === Private Functions === #


# === Entry Point === #

if __name__ == "__main__":
    main()
