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
from dokime_tools.build.render_resources import Method, csharp, csharp_host, table

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

root: Path = Path(__file__).resolve().parents[3]
descriptors: FileDescriptorSet = FileDescriptorSet.FromString((root / "crates/dokime-protocol/src/descriptors.bin").read_bytes())
settings: dict[str, object] = tomllib.loads((root / "protocol/composition.toml").read_text(encoding="utf-8"))
callers: dict[str, str] = cast(dict[str, str], settings.get("callers", {}))
narrow_callers(descriptors, callers)
methods: list[Method] = table(descriptors)
csharp(
    root,
    [method for method in methods if method["face"] == "client"],
    namespace="dokime",
    target=root / "sdk/csharp/src/Dokime/Generated/Client.g.cs",
    check=True,
)
csharp_host(root, [method for method in methods if method["face"] == "context"], check=True)
protocols(root, check=True)
print("C# Generation: Frozen Roots and Wire Match Their Descriptors")
