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
import subprocess
import tomllib
from pathlib import Path

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# Local Imports
from ..process import spawn_flags
from . import render_resources as render
from .csharp_protocol import protocols as csharp_protocols
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
            raise ValueError("Compose SDK: A Library Package Is Missing")
    narrow_callers(descriptors, settings.get("callers", {}))
    methods = render.table(descriptors)
    clients = [method for method in methods if method["face"] == "client"]
    context = [method for method in methods if method["face"] == "context"]
    for folder in ("sdk/rust/src/client", "sdk/rust/src/plugin", "sdk/python/src/dokime/generated", "sdk/csharp/src/Dokime/Generated"):
        (root / folder).mkdir(parents=True, exist_ok=True)
    packages = {file.package for file in descriptors.file if file.package.startswith("dokime.")}
    for path in (root / "crates/dokime-protocol/src/generated").glob("dokime.*.rs"):
        if path.stem not in packages:
            path.unlink()
    (root / "sdk/resources.json").write_text(json.dumps(methods, indent=2) + "\n", encoding="utf-8", newline="\n")
    render.rust_exports(root, descriptors)
    render.rust(root, clients, target=root / "sdk/rust/src/client/resources.rs")
    (root / "sdk/rust/src/client/client_types.rs").unlink(missing_ok=True)
    render.rust_host(root, context)
    bindings = [
        "// @generated from protocol descriptors; do not edit.",
        "use crate::message::Wire;",
        "use prost_reflect::ReflectMessage;",
        "pub use crate::message::Wire as Call;",
        "pub trait Request: ReflectMessage + Default + 'static { type Response: ReflectMessage + Default + 'static; const ID: &'static str; }",
        "impl<T: Request> harmos::OperationRequest for Wire<T> { type Response = Wire<T::Response>; const ID: &'static str = T::ID; }",
    ]
    for method in context:
        q = render.rust_message(method["request"])
        r = render.rust_message(method["response"])
        identity = f"{method['package']}.{method['service']}/{method['method']}"
        bindings.append(f"impl Request for {q} {{ type Response = {r}; const ID: &'static str = {json.dumps(identity)}; }}")
    (root / "sdk/rust/src/host.rs").write_text("\n".join(bindings) + "\n", encoding="utf-8", newline="\n")

    render.python_exports(root / "sdk/python/src/dokime/_generated", descriptors)
    render.python(root / "sdk/python/src/dokime/generated/resources.py", clients, descriptors)
    render.python_host(root / "sdk/python/src/dokime/_generated/host.py", context, descriptors)
    render.csharp(root, clients, namespace="dokime", target=root / "sdk/csharp/src/Dokime/Generated/Client.g.cs")
    render.csharp_host(root, context)
    csharp_protocols(root)
    render.cpp_exports(root, descriptors)
    render.cpp(root, clients, namespace="dokime", target=root / "sdk/cpp/include/dokime/_generated/client.hpp")
    render.cpp_host(root, context)
    # Generated Public Barrel Points Hide the Wire Packages.
    shared_names = [
        line.split(" as ")[-1]
        for line in (root / "sdk/python/src/dokime/_generated/shared.py").read_text().splitlines()
        if line.startswith("from ")
    ]
    client_names = [
        line.split(" as ")[-1]
        for line in (root / "sdk/python/src/dokime/_generated/client_types.py").read_text().splitlines()
        if line.startswith("from ")
    ]
    plugin_names = [
        line.split(" as ")[-1]
        for line in (root / "sdk/python/src/dokime/_generated/plugin_types.py").read_text().splitlines()
        if line.startswith("from ")
    ]
    root_barrel = [
        "# @generated from protocol descriptors; do not edit.",
        "from .client import Client as Client",
        "from ._plugin import Plugin as Plugin",
        "from ._context import Context as Context",
        "from harmos import Failure as Failure",
    ]
    root_barrel += [f"from ._generated.shared import {name} as {name}" for name in sorted(shared_names)]
    (root / "sdk/python/src/dokime/__init__.py").write_text("\n".join(root_barrel) + "\n", encoding="utf-8", newline="\n")
    client_barrel = ["# @generated from protocol descriptors; do not edit.", "from .generated.resources import ClientRoot as Client"]
    client_barrel += [f"from ._generated.client_types import {name} as {name}" for name in sorted(client_names)]
    (root / "sdk/python/src/dokime/client.py").write_text("\n".join(client_barrel) + "\n", encoding="utf-8", newline="\n")
    plugin_barrel = [
        "# @generated from protocol descriptors; do not edit.",
        "from .._plugin import Plugin as Plugin",
        "from .._context import Context as Context",
        "from .._generated.host import HostOperation as HostOperation",
        "from .._generated.host import HostRoot as HostRoot",
    ]
    plugin_barrel += [f"from .._generated.plugin_types import {name} as {name}" for name in sorted(plugin_names)]
    (root / "sdk/python/src/dokime/plugin/__init__.py").write_text("\n".join(plugin_barrel) + "\n", encoding="utf-8", newline="\n")
    (root / "sdk/python/src/dokime/model.py").unlink(missing_ok=True)
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
            "generated/resources.py",
        )
    ]
    subprocess.run(
        ["uv", "run", "--no-sync", "--project", "tools/python", "ruff", "check", "--select", "I,F401", "--fix", *map(str, python_paths)],
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
        result = subprocess.run(["git", "diff", "--exit-code", "--", "crates/dokime-protocol", "sdk"], creationflags=spawn_flags())
        if result.returncode:
            raise ValueError("Check SDK Definitions: Regenerate and Commit the Candidate")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    generate(parser.parse_args().check)
