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

# isort: split

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# isort: split

# Local Imports
from ..process import spawn_flags
from . import csharp_protocol
from .protocol_callers import narrow_callers
from .resources import cpp, csharp, python, rust, schema

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
            raise ValueError("Build Resource Roots: Compose SDK: Declare the Vendored Descriptor Artifact")
        if set(library["packages"]) - present:
            raise ValueError("Build Resource Roots: Compose SDK: A Library Package Is Missing")
    narrow_callers(descriptors, settings.get("callers", {}))
    methods = schema.table(descriptors)
    public_names = schema.public_names(descriptors)
    clients = []
    for clients_method in methods:
        if clients_method["face"] != "client":
            continue
        clients.append(clients_method)

    context = []
    for context_method in methods:
        if context_method["face"] != "context":
            continue
        context.append(context_method)

    for folder in ("sdk/rust/src/client", "sdk/rust/src/plugin", "sdk/python/src/dokime/_generated", "sdk/csharp/src/Dokime/Generated"):
        (root / folder).mkdir(parents=True, exist_ok=True)
    packages = set()
    for packages_file in descriptors.file:
        if not (packages_file.package.startswith("dokime.")):
            continue
        packages.add(packages_file.package)

    for path in (root / "crates/dokime-protocol/src/generated").glob("dokime.*.rs"):
        if path.stem not in packages:
            path.unlink()
    for retired in (
        "sdk/resources.json",
        "sdk/python/src/dokime/generated/resources.py",
        "sdk/python/src/dokime/generated/__init__.py",
        "sdk/python/src/dokime/generated/py.typed",
        "sdk/cpp/include/dokime/_generated/client_wire.hpp",
        "sdk/cpp/include/dokime/detail/messages_wire.hpp",
    ):
        (root / retired).unlink(missing_ok=True)
    rust.exports(root, descriptors)
    rust.render(root, clients, public_names=public_names, target=root / "sdk/rust/src/client/resources.rs")
    (root / "sdk/rust/src/client/client_types.rs").unlink(missing_ok=True)
    rust.host(root, context)
    bindings = [
        "// @generated from protocol descriptors; do not edit.",
        "use crate::message::Wire;",
        "use prost_reflect::ReflectMessage;",
        "pub use crate::message::Wire as Call;",
        "pub trait Request: ReflectMessage + Default + 'static { type Response: ReflectMessage + Default + 'static; const ID: &'static str; }",
        "impl<T: Request> harmos::OperationRequest for Wire<T> { type Response = Wire<T::Response>; const ID: &'static str = T::ID; }",
    ]
    for method in context:
        q = rust.message(method["request"])
        r = rust.message(method["response"])
        identity = f"{method['package']}.{method['service']}/{method['method']}"
        bindings.append(f"impl Request for {q} {{ type Response = {r}; const ID: &'static str = {json.dumps(identity)}; }}")
    (root / "sdk/rust/src/host.rs").write_text("\n".join(bindings) + "\n", encoding="utf-8", newline="\n")

    python.exports(root / "sdk/python/src/dokime/_generated", descriptors)
    python.render(root / "sdk/python/src/dokime/_generated/client.py", clients, descriptors)
    python.host(root / "sdk/python/src/dokime/_generated/host.py", context, descriptors)
    csharp.render(root, clients, public_names=public_names, namespace="dokime", target=root / "sdk/csharp/src/Dokime/Generated/Client.g.cs")
    csharp.host(root, context)
    csharp_protocol.protocols(root)
    type_names = cpp.exports(root, descriptors)
    cpp.render(root, clients, type_names=type_names, namespace="dokime", target=root / "sdk/cpp/include/dokime/_generated/client.hpp")
    cpp.host(root, context, type_names)
    # Generated Public Barrel Points Hide the Wire Packages.
    shared_names = []
    for shared_names_line in (root / "sdk/python/src/dokime/_generated/shared.py").read_text().splitlines():
        if not (shared_names_line.startswith("from ")):
            continue
        shared_names.append(shared_names_line.split(" as ")[-1])

    client_names = []
    for client_names_line in (root / "sdk/python/src/dokime/_generated/client_types.py").read_text().splitlines():
        if not (client_names_line.startswith("from ")):
            continue
        client_names.append(client_names_line.split(" as ")[-1])

    plugin_names = []
    for plugin_names_line in (root / "sdk/python/src/dokime/_generated/plugin_types.py").read_text().splitlines():
        if not (plugin_names_line.startswith("from ")):
            continue
        plugin_names.append(plugin_names_line.split(" as ")[-1])

    root_barrel = [
        "# @generated from protocol descriptors; do not edit.",
        "from harmos import Failure as Failure",
    ]
    root_barrel += [f"from ._generated.shared import {name} as {name}" for name in sorted(shared_names)]
    root_barrel += ["", "__all__ = " + repr(["Failure", *sorted(shared_names)])]
    (root / "sdk/python/src/dokime/__init__.py").write_text("\n".join(root_barrel) + "\n", encoding="utf-8", newline="\n")
    client_barrel = ["# @generated from protocol descriptors; do not edit.", "from ._generated.client import ClientRoot as Client"]
    client_barrel += [f"from ._generated.client_types import {name} as {name}" for name in sorted(client_names)]
    client_barrel += ["", "__all__ = " + repr(["Client", *sorted(client_names)])]
    (root / "sdk/python/src/dokime/client.py").write_text("\n".join(client_barrel) + "\n", encoding="utf-8", newline="\n")
    plugin_barrel = [
        "# @generated from protocol descriptors; do not edit.",
        "from .._plugin import Plugin as Plugin",
        "from .._context import Context as Context",
        "from .._generated.host import HostOperation as HostOperation",
        "from .._generated.host import HostRoot as HostRoot",
        "from harmos.plugin import Window as SampleWindow",
    ]
    plugin_barrel += [f"from .._generated.plugin_types import {name} as {name}" for name in sorted(plugin_names)]
    plugin_barrel += ["", "__all__ = " + repr(["Plugin", "Context", "HostOperation", "HostRoot", "SampleWindow", *sorted(plugin_names)])]
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
        result = subprocess.run(
            ["git", "diff", "--exit-code", "--", "crates/dokime-protocol", "sdk"], check=False, creationflags=spawn_flags()
        )
        if result.returncode:
            raise ValueError("Check SDK Definitions: Regenerate and Commit the Candidate")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    generate(parser.parse_args().check)
