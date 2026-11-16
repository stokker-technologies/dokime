# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Generate Product C# Wire Messages and Internal Service Stubs."""

# === Imports === #

# Standard Library Imports
import platform
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorProto, FileDescriptorSet

# Local Imports
from ..process import spawn_flags
from . import render_resources

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def protocols(
    root: Path,
    *,
    descriptors: Path | None = None,
    destination: Path | None = None,
    check: bool = False,
    package: str | None = None,
    files: set[str] | None = None,
    namespace_override: str | None = None,
) -> None:
    """Keep Wire Requests and Carriers Internal; Give Payloads Their Public Namespace."""
    descriptors = descriptors or root / "crates/dokime-protocol/src/descriptors.bin"
    schema = FileDescriptorSet.FromString(descriptors.read_bytes())
    requests = {method["request"].rsplit(".", 1)[-1] for method in render_resources.table(schema)}
    requests.update(
        {"Request", "Response", "ChildRequest", "ChildResponse", "ChildCancel", "ChildWatch", "ChildWatchResponse", "ChildWatchCancel"}
    )
    # The Staged Harmos Project Restores Grpc.Tools Into the SDK's Own Package Folder (Harmos.csproj).
    tools = root / "target/cs/packages/grpc.tools/2.76.0/tools"
    systems = {"Windows": "windows", "Darwin": "macosx"}
    system = systems.get(platform.system(), "linux")
    architecture = "arm64" if platform.machine().lower() in ("arm64", "aarch64") else "x64"
    executable_suffix = ".exe" if system == "windows" else ""
    protoc = tools / f"{system}_{architecture}" / ("protoc" + executable_suffix)
    plugin = protoc.with_name("grpc_csharp_plugin" + executable_suffix)
    if not protoc.is_file():
        raise FileNotFoundError("Generate C# Protocol: Restore Grpc.Tools With mise -C sdk/csharp run prepare:harmos")
    scratch = Path(tempfile.mkdtemp())
    destination = destination or root / "sdk/csharp/src/Dokime/Generated/Wire"
    try:
        for file in schema.file:
            if not file.package.startswith("dokime."):
                continue
            if package is not None and not (file.package == package or file.package.startswith(package + ".")):
                continue
            if files is not None and file.name not in files:
                continue
            output = scratch / Path(file.name).with_suffix("")
            output.mkdir(parents=True, exist_ok=True)
            command = [str(protoc), f"--descriptor_set_in={descriptors}", f"--csharp_out={output}"]
            if file.service and namespace_override is None:
                command += [f"--plugin=protoc-gen-grpc={plugin}", f"--grpc_out={output}", "--grpc_opt=internal_access"]
            subprocess.run([*command, file.name], check=True, creationflags=spawn_flags())
            for path in output.glob("*.cs"):
                source = _wire_source(path.read_text(encoding="utf-8"), path.name, file, requests, namespace_override)
                path.write_text(source, encoding="utf-8", newline="\n")
        if check:
            expected = {path.relative_to(scratch) for path in scratch.rglob("*.cs")}
            existing = {path.relative_to(destination) for path in destination.rglob("*.cs")}
            if expected != existing or any((scratch / path).read_bytes() != (destination / path).read_bytes() for path in expected):
                raise ValueError("Check C# Protocol: Regenerate With mise run generate -- --protocol")
        else:
            destination.mkdir(parents=True, exist_ok=True)
            for old in destination.rglob("*.cs"):
                old.unlink()
            shutil.copytree(scratch, destination, dirs_exist_ok=True)
    finally:
        shutil.rmtree(scratch)


# === Private Functions === #


def _wire_source(
    source: str,
    filename: str,
    file: FileDescriptorProto,
    requests: set[str],
    namespace_override: str | None,
) -> str:
    """Apply the Public Payload and Internal Carrier Namespace Contract."""
    source = render_resources.csharp_message(source)
    if namespace_override is not None:
        source = source.replace("namespace Harmos.Clients {", f"namespace {namespace_override} {{")
        source = source.replace("global::Harmos.Clients.", f"global::{namespace_override}.")
    for name in requests:
        source = source.replace(f"public sealed partial class {name} :", f"internal sealed partial class {name} :")
    if filename.endswith("Grpc.cs"):
        source = source.replace("namespace Harmos.Plugins {", "namespace Harmos.Internal.Host {")
        source = source.replace("namespace Harmos.Clients {", "namespace Harmos.Internal.Client {")
        namespace = render_resources.csharp_message(file.options.csharp_namespace)
        if namespace.endswith(".Clients"):
            source = source.replace(f"namespace {namespace} {{", f"namespace {namespace.removesuffix('.Clients')}.Internal.Client {{")
    # Generated Namespaces Follow the Same File-Scoped Rule as Authored C#.
    match = re.search(r"namespace ([\w.]+) \{", source)
    if match:
        source = source[: match.start()] + f"namespace {match[1]};" + source[match.end() :]
        ending = source.rfind("}")
        source = source[:ending] + source[ending + 1 :]
    return source


# === Entry Point === #
