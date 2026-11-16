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
from typing import cast

# Local Imports
from dokime_tools.build.csharp_protocol import protocols
from dokime_tools.build.protocol_callers import narrow_callers
from dokime_tools.build import render_resources

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    descriptors = FileDescriptorSet.FromString((root / "crates/dokime-protocol/src/descriptors.bin").read_bytes())
    settings = tomllib.loads((root / "protocol/composition.toml").read_text(encoding="utf-8"))
    callers = cast(dict[str, str], settings.get("callers", {}))
    narrow_callers(descriptors, callers)
    methods = render_resources.table(descriptors)
    render_resources.csharp(
        root,
        [method for method in methods if method["face"] == "client"],
        namespace="dokime",
        target=root / "sdk/csharp/src/Dokime/Generated/Client.g.cs",
        check=True,
    )
    render_resources.csharp_host(root, [method for method in methods if method["face"] == "context"], check=True)
    protocols(root, check=True)
    print("C# Generation: Frozen Roots and Wire Match Their Descriptors")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
