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
import re
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


def table(descriptors: FileDescriptorSet, snake) -> list[dict]:
    methods = []
    leaves = set()
    csharp_types = {}
    for file in descriptors.file:
        namespace = file.options.csharp_namespace or ".".join(part.title() for part in file.package.split("."))
        for message in file.message_type:
            csharp_types[f".{file.package}.{message.name}"] = f"{namespace}.{message.name}"
    for file in descriptors.file:
        parts = file.package.split(".")
        version = next((index for index, part in enumerate(parts) if re.fullmatch(r"v[0-9]+", part)), None)
        if version is None or version == 0:
            continue
        audience = parts[1:version]
        face = "context" if audience in (["context"], ["plugin", "host"]) else "plugin" if audience == ["plugin"] else "client"
        for service in file.service:
            for method in service.method:
                path = [*parts[version + 1 :], snake(service.name), snake(method.name)]
                leaf = (face, *path)
                if leaf in leaves:
                    raise ValueError(f"Client Resource Collision: {'.'.join(path)}")
                leaves.add(leaf)
                methods.append(
                    {
                        "owner": parts[0],
                        "face": face,
                        "version": parts[version],
                        "parents": parts[version + 1 :],
                        "service": service.name,
                        "method": method.name,
                        "package": file.package,
                        "path": path,
                        "request": method.input_type.lstrip("."),
                        "response": method.output_type.lstrip("."),
                        "request_csharp": csharp_types[method.input_type],
                        "response_csharp": csharp_types[method.output_type],
                        "client_streaming": method.client_streaming,
                        "server_streaming": method.server_streaming,
                        "file": file.name,
                    }
                )
    for leaf in leaves:
        if any(leaf[:length] in leaves for length in range(2, len(leaf))):
            raise ValueError(f"Client Resource Collision: {'.'.join(leaf[1:])}")
    return methods


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
    packages = {file.package for file in descriptors.file if file.package.startswith("dokime.")}
    for path in (root / "crates/dokime-protocol/src/generated").glob("dokime.*.rs"):
        if path.stem not in packages:
            if check:
                raise ValueError(f"Check Protocol: Retired Generated Module Remains: {path}")
            path.unlink()
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
    # Ordinary Contracts Awaiting Direct Library Composition Stay Off SDK Roots.
    for file in descriptors.file:
        if file.name in (
            "dokime/client/v1/plugins.proto",
            "dokime/client/v1/content.proto",
            "harmos/v1/storage.proto",
            "harmos/context/v1/storage.proto",
        ):
            file.ClearField("service")
    methods = table(descriptors, generator["snake"])
    clients = [method for method in methods if method["face"] == "client"]
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
        (output / outputs[0]).write_text(json.dumps(methods, indent=2) + "\n", encoding="utf-8", newline="\n")
        (output / outputs[1]).write_text(json.dumps(clients, indent=2) + "\n", encoding="utf-8", newline="\n")
        generator["rust"](output, clients, target=output / outputs[2])
        # The Pinned Library's Rust Renderer Predates Explicit Product Client Packages.
        rust = (output / outputs[2]).read_text(encoding="utf-8")
        for method in clients:
            for field in ("request", "response"):
                name = method[field]
                if name.startswith("dokime.client.v1."):
                    suffix = name.removeprefix("dokime.client.v1.").replace(".", "::")
                    rust = rust.replace(f"dokime_protocol::v1::{suffix}", f"dokime_protocol::client::{suffix}")
        (output / outputs[2]).write_text(rust, encoding="utf-8", newline="\n")
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
