# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Generate the Composed Client, Plugin Context and Public Payload Barrels."""

# === Imports === #

# Standard Library Imports
import argparse
import json
import re
import subprocess
import tomllib
from pathlib import Path

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# Local Imports
from ..process import spawn_flags
from . import csharp_protocol, render_resources
from .protocol_callers import narrow_callers

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def generate(check: bool = False) -> None:
    root = Path.cwd()
    composition = root / "protocol/composition.toml"
    settings = tomllib.loads(composition.read_text(encoding="utf-8"))
    descriptors = FileDescriptorSet.FromString((root / "crates/dokime-protocol/src/descriptors.bin").read_bytes())
    present = {file.package for file in descriptors.file}
    for library in settings["libraries"]:
        source = (composition.parent / library["descriptors"]).resolve()
        if source != (root / "crates/dokime-protocol/src/descriptors.bin").resolve():
            raise ValueError("Compose SDK: Declare the Vendored Descriptor Artifact")
        if set(library["packages"]) - present:
            raise ValueError("Compose SDK: Add the Missing Library Package to the Vendored Descriptors")
    narrow_callers(descriptors, settings.get("callers", {}))
    methods = render_resources.table(descriptors)
    clients = [method for method in methods if method["face"] == "client"]
    context = [method for method in methods if method["face"] == "context"]
    for folder in ("sdk/rust/src/client", "sdk/rust/src/plugin", "sdk/python/src/dokime/_generated", "sdk/csharp/src/Dokime/Generated"):
        (root / folder).mkdir(parents=True, exist_ok=True)
    packages = {file.package for file in descriptors.file if file.package.startswith("dokime.")}
    for path in (root / "crates/dokime-protocol/src/generated").glob("dokime.*.rs"):
        if path.stem not in packages:
            path.unlink()
    render_resources.rust_exports(root, descriptors)
    render_resources.rust(root, clients, target=root / "sdk/rust/src/client/resources.rs")
    render_resources.rust_host(root, context)
    # The Renderer and Native Carrier Own These Internal Names; Public SDK Faces
    # Expose Call, Client and SampleWindow Without Leaking Their Implementation Names.
    bindings = [
        "// @generated from protocol descriptors; do not edit.",
        "use crate::message::Wire;",
        "use prost_reflect::ReflectMessage;",
        "pub use crate::message::Wire as Call;",
        "pub trait Request: ReflectMessage + Default + 'static {\n    type Response: ReflectMessage + Default + 'static;\n    const ID: &'static str;\n}",
        "impl<T: Request> harmos::OperationRequest for Wire<T> { type Response = Wire<T::Response>; const ID: &'static str = T::ID; }",
    ]
    for method in context:
        request = render_resources.rust_message(method["request"])
        response = render_resources.rust_message(method["response"])
        identity = f"{method['package']}.{method['service']}/{method['method']}"
        bindings.append(f"impl Request for {request} {{ type Response = {response}; const ID: &'static str = {json.dumps(identity)}; }}")
    (root / "sdk/rust/src/host.rs").write_text("\n".join(bindings) + "\n", encoding="utf-8", newline="\n")

    render_resources.python_exports(root / "sdk/python/src/dokime/_generated", descriptors)
    render_resources.python(root / "sdk/python/src/dokime/_generated/client.py", clients, descriptors)
    render_resources.python_host(root / "sdk/python/src/dokime/_generated/host.py", context, descriptors)
    render_resources.csharp(root, clients, namespace="dokime", target=root / "sdk/csharp/src/Dokime/Generated/Client.g.cs")
    render_resources.csharp_host(root, context)
    csharp_protocol.protocols(root)
    render_resources.cpp_exports(root, descriptors)
    render_resources.cpp(root, clients, namespace="dokime", target=root / "sdk/cpp/include/dokime/_generated/client.hpp")
    render_resources.cpp_host(root, context)
    # Generated Public Barrel Points Hide the Wire Packages.
    shared_names = [
        line.rsplit(" import ", 1)[-1].rsplit(" as ", 1)[-1]
        for line in (root / "sdk/python/src/dokime/_generated/shared.py").read_text().splitlines()
        if line.startswith("from ")
    ]
    client_names = [
        line.rsplit(" import ", 1)[-1].rsplit(" as ", 1)[-1]
        for line in (root / "sdk/python/src/dokime/_generated/client_types.py").read_text().splitlines()
        if line.startswith("from ")
    ]
    plugin_names = [
        line.rsplit(" import ", 1)[-1].rsplit(" as ", 1)[-1]
        for line in (root / "sdk/python/src/dokime/_generated/plugin_types.py").read_text().splitlines()
        if line.startswith("from ")
    ]
    root_barrel = [
        "# @generated from protocol descriptors; do not edit.",
        "from harmos import Failure",
    ]
    root_barrel += [f"from ._generated.shared import {name}" for name in sorted(shared_names)]
    root_barrel += ["", "__all__ = " + repr(["Failure", *sorted(shared_names)])]
    (root / "sdk/python/src/dokime/__init__.py").write_text("\n".join(root_barrel) + "\n", encoding="utf-8", newline="\n")
    client_barrel = ["# @generated from protocol descriptors; do not edit.", "from ._generated.client import ClientRoot as Client"]
    client_barrel += [f"from ._generated.client_types import {name}" for name in sorted(client_names)]
    client_barrel += ["", "__all__ = " + repr(["Client", *sorted(client_names)])]
    (root / "sdk/python/src/dokime/client.py").write_text("\n".join(client_barrel) + "\n", encoding="utf-8", newline="\n")
    plugin_barrel = [
        "# @generated from protocol descriptors; do not edit.",
        "from .._plugin import Plugin",
        "from .._context import Context",
        "from .._generated.host import HostOperation",
        "from .._generated.host import HostRoot",
        "from harmos.plugin import Window as SampleWindow",
    ]
    plugin_barrel += [f"from .._generated.plugin_types import {name}" for name in sorted(plugin_names)]
    plugin_barrel += ["", "__all__ = " + repr(["Plugin", "Context", "HostOperation", "HostRoot", "SampleWindow", *sorted(plugin_names)])]
    (root / "sdk/python/src/dokime/plugin/__init__.py").write_text("\n".join(plugin_barrel) + "\n", encoding="utf-8", newline="\n")
    paths = [
        root / "sdk/rust/src" / name
        for name in (
            "exports.rs",
            "host.rs",
            "client/types.rs",
            "client/resources.rs",
            "plugin/host.rs",
            "plugin/host_exports.rs",
            "plugin/types.rs",
        )
    ]
    subprocess.run(
        ["rustfmt", "--edition", "2024", "--config-path", str(root / "rustfmt.toml"), *map(str, paths)],
        check=True,
        creationflags=spawn_flags(),
    )
    python_paths = [
        root / "sdk/python/src/dokime" / name
        for name in (
            "__init__.py",
            "client.py",
            "plugin/__init__.py",
            "_generated/shared.py",
            "_generated/client_types.py",
            "_generated/plugin_types.py",
            "_generated/host.py",
            "_generated/client.py",
        )
    ]
    for path in python_paths:
        source = re.sub(r"\bimport (\w+) as \1\b", r"import \1", path.read_text(encoding="utf-8"))
        path.write_text(source, encoding="utf-8", newline="\n")
    subprocess.run(
        [
            "uv",
            "run",
            "--no-sync",
            "--project",
            "tools/python",
            "ruff",
            "check",
            "--select",
            "I,F401,RUF022",
            "--fix",
            *map(str, python_paths),
        ],
        check=True,
        creationflags=spawn_flags(),
    )
    subprocess.run(
        ["uv", "run", "--no-sync", "--project", "tools/python", "ruff", "format", *map(str, python_paths)],
        check=True,
        creationflags=spawn_flags(),
    )
    # Check Uses the Same Full Generation and Compares Against Git, Including New Outputs.
    if check:
        command = ["git", "status", "--porcelain", "--untracked-files=all", "--", "crates/dokime-protocol", "sdk"]
        result = subprocess.run(command, capture_output=True, text=True, check=True, creationflags=spawn_flags())
        if result.stdout:
            raise ValueError("Check SDK Definitions: Regenerate and Commit the Candidate\n" + result.stdout)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    generate(parser.parse_args().check)
