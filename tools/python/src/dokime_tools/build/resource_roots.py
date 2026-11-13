# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compose Descriptor-Derived SDK Roots with the Pinned Library"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import runpy
import subprocess
import sys
import tempfile
from pathlib import Path

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# Local Imports
from dokime_tools.process import spawn_flags

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def generate(check: bool = False) -> None:
    root = Path.cwd()
    metadata = json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"],
            cwd=root,
            creationflags=spawn_flags(),
        )
    )
    manifest = next(Path(package["manifest_path"]) for package in metadata["packages"] if package["name"] == "harmos-protocol")
    library = manifest.parents[2]
    generator = runpy.run_path(str(library / "tools/python/src/harmos_tools/build/resource_roots.py"))
    descriptors = FileDescriptorSet.FromString((root / "crates/dokime-protocol/src/descriptors.bin").read_bytes())
    present = {file.name: file for file in descriptors.file}
    library_descriptors = FileDescriptorSet.FromString((library / "crates/harmos-protocol/src/descriptors.bin").read_bytes())
    for file in library_descriptors.file:
        file.ClearField("source_code_info")
        if file.name in present:
            present[file.name].ClearField("source_code_info")
            if present[file.name] != file:
                raise ValueError(f"Generate SDK Roots: Vendored Descriptor Differs from the Pinned Library: {file.name}")
        else:
            descriptors.file.append(file)
    # Superseded Client Groups Leave the Table as Their Resource Replacements Land.
    for file in descriptors.file:
        if file.name.startswith("dokime/v1/client_"):
            file.ClearField("service")
    table = generator["table"](descriptors)
    clients = [method for method in table if method["face"] == "client"]
    # Expose the Same Generated Children on the Channel-Owning C# Entry Class.
    roots = sorted({method["path"][0] for method in clients})
    declarations = [
        "// @generated from the shared resource table; do not edit.",
        "namespace Dokime;",
        "",
        "public sealed partial class Client",
        "{",
    ]
    for child in roots:
        name = "".join(word.title() for word in child.split("_"))
        declarations.append(f"  public Sdk.{name}Resource {name} => new(Calls);")
    declarations.append("}")
    outputs = [
        "sdk/resources.json",
        "sdk/python/src/dokime/resources.json",
        "sdk/rust/src/client/resources.rs",
        "sdk/csharp/Dokime/Resources.cs",
        "sdk/csharp/Dokime/ClientResources.cs",
        "sdk/cpp/include/dokime/resources.hpp",
    ]
    with tempfile.TemporaryDirectory(prefix="dokime-sdk-roots-") as scratch:
        output = Path(scratch)
        for path in outputs:
            (output / path).parent.mkdir(parents=True, exist_ok=True)
        (output / outputs[0]).write_text(json.dumps(table, indent=2) + "\n", encoding="utf-8", newline="\n")
        (output / outputs[1]).write_text(json.dumps(clients, indent=2) + "\n", encoding="utf-8", newline="\n")
        generator["rust"](output, clients, target=output / outputs[2])
        generator["csharp"](output, clients, namespace="dokime", target=output / outputs[3])
        (output / outputs[4]).write_text("\n".join(declarations) + "\n", encoding="utf-8", newline="\n")
        generator["cpp"](output, clients, namespace="dokime.resources", target=output / outputs[5])
        subprocess.run(
            [sys.executable, str(root / "sdk/cpp/tools/sdk.py"), "format", "--source", str(output / outputs[5])],
            check=True,
            creationflags=spawn_flags(),
        )
        for path in outputs:
            data = (output / path).read_bytes()
            if not (root / path).is_file() or (root / path).read_bytes() != data:
                if check:
                    raise ValueError(f"Check SDK Roots: Regenerate {path} Through mise run generate -- --protocol")
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                (root / path).write_bytes(data)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    generate(parser.parse_args().check)
