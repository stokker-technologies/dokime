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
import tempfile
import tomllib
from pathlib import Path

# isort: split

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# isort: split

# Local Imports
from dokime_tools.build.protocol_callers import narrow_callers
from dokime_tools.build.resources import cpp, schema

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
    methods = schema.table(descriptors)
    clients = []
    plugins = []
    for method in methods:
        if method["face"] == "client":
            clients.append(method)
        if method["face"] == "context":
            plugins.append(method)
    directory = tempfile.TemporaryDirectory(prefix="dokime-cpp-generation-")
    try:
        staging = Path(directory.name)
        descriptor_path = staging / "crates/dokime-protocol/src/descriptors.bin"
        descriptor_path.parent.mkdir(parents=True)
        descriptor_path.write_bytes(descriptors.SerializeToString())
        target = staging / "sdk/cpp/include/dokime/_generated/client.hpp"
        type_names = cpp.exports(staging, descriptors)
        cpp.render(staging, clients, type_names=type_names, namespace="dokime", target=target)
        cpp.host(staging, plugins, type_names)
        changed = []
        for path in sorted((staging / "sdk/cpp").rglob("*")):
            if not path.is_file():
                continue
            relative = path.relative_to(staging)
            if path.read_bytes() == (root / relative).read_bytes():
                continue
            changed.append(str(relative))
        if changed:
            raise RuntimeError("Check C++ Generation: Frozen Output Drift: " + ", ".join(changed))
        print("C++ Generation Matches All Six Frozen Outputs")
    finally:
        directory.cleanup()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
