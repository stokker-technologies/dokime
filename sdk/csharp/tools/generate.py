# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Regenerate C# in Memory and Verify the Frozen Outputs Without Rewriting Them."""

# === Imports === #

# Standard Library Imports
import tomllib
from pathlib import Path

# isort: split

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# isort: split

# Local Imports
from dokime_tools.build.csharp_protocol import protocols
from dokime_tools.build.protocol_callers import narrow_callers
from dokime_tools.build.resources import csharp, schema

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    descriptors = FileDescriptorSet.FromString(
        (root / "crates/dokime-protocol/src/descriptors.bin").read_bytes()
    )
    composition = tomllib.loads(
        (root / "protocol/composition.toml").read_text(encoding="utf-8")
    )
    narrow_callers(descriptors, composition.get("callers", {}))
    public_names = schema.public_names(descriptors)
    clients = []
    plugins = []
    for method in schema.table(descriptors):
        if method["face"] == "client":
            clients.append(method)
        if method["face"] == "context":
            plugins.append(method)
    csharp.render(
        root,
        clients,
        public_names=public_names,
        namespace="dokime",
        target=root / "sdk/csharp/src/Dokime/Generated/Client.g.cs",
        check=True,
    )
    csharp.host(root, plugins, check=True)
    protocols(root, check=True)
    print("C# Generation: Frozen Roots and Wire Match Their Descriptors")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
