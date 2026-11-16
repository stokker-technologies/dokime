# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Check C++ Descriptor Projections Without Changing Frozen Outputs."""

# === Imports === #

# Standard Library Imports
import shutil
import tempfile
import tomllib
from pathlib import Path

# Third Party Imports
from dokime_tools.build import render_resources
from dokime_tools.build.protocol_callers import narrow_callers
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    descriptors = FileDescriptorSet.FromString((root / "crates/dokime-protocol/src/descriptors.bin").read_bytes())
    composition = tomllib.loads((root / "protocol/composition.toml").read_text(encoding="utf-8"))
    narrow_callers(descriptors, composition.get("callers", {}))
    methods = render_resources.table(descriptors)
    clients = [method for method in methods if method["face"] == "client"]
    plugins = [method for method in methods if method["face"] == "context"]
    staging = Path(tempfile.mkdtemp(prefix="dokime-cpp-generation-"))
    try:
        descriptor_path = staging / "crates/dokime-protocol/src/descriptors.bin"
        descriptor_path.parent.mkdir(parents=True)
        descriptor_path.write_bytes(descriptors.SerializeToString())
        target = staging / "sdk/cpp/include/dokime/_generated/client.hpp"
        render_resources.cpp_exports(staging, descriptors)
        render_resources.cpp(staging, clients, namespace="dokime", target=target)
        render_resources.cpp_host(staging, plugins)
        changed = [
            str(path.relative_to(staging))
            for path in sorted((staging / "sdk/cpp").rglob("*"))
            if path.is_file() and path.read_bytes() != (root / path.relative_to(staging)).read_bytes()
        ]
        if changed:
            raise RuntimeError("Check C++ Generation: Regenerate With mise run generate -- --protocol: " + ", ".join(changed))
        print("C++ Generation Matches All Six Frozen Outputs")
    finally:
        shutil.rmtree(staging)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
