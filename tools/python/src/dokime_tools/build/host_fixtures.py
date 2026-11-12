# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build And Export The Shared Execution Host Fixtures Through SDK Tasks."""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import shutil
import subprocess

# Third Party Imports
# Local Imports
from dokime_tools.build import configuration, paths
from dokime_tools.process import spawn_flags
from dokime_tools.test import archive

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> None:
    root = configuration.ROOT
    output = paths.output(root, "build", profile) / "fixtures/host"
    output.mkdir(parents=True, exist_ok=True)
    extension = ".exe" if os.name == "nt" else ""

    # Build The Rust Reference And Execution Fixtures.
    subprocess.run(["mise", "-C", "sdk/rust", "run", "sdk:build"], cwd=root, check=True, creationflags=spawn_flags())
    for name in ("dokime-rust-thermal", "dokime-rust-execution"):
        shutil.copy2(root / "target/debug" / f"{name}{extension}", output / f"{name}{extension}")

    # Publish The C# Fixture With Its Runtime And Local SDK Package.
    subprocess.run(["mise", "-C", "sdk/csharp", "run", "build"], cwd=root, check=True, creationflags=spawn_flags())
    subprocess.run(["mise", "-C", "sdk/csharp", "run", "package"], cwd=root, check=True, creationflags=spawn_flags())
    subprocess.run(
        ["mise", "-C", "sdk/csharp", "run", "publish:system", "--", str(output / "csharp")],
        cwd=root,
        check=True,
        creationflags=spawn_flags(),
    )

    # Build The C++ Fixture With The Pinned Native Dependency Graph.
    subprocess.run(
        ["mise", "-C", "sdk/cpp", "run", "configure", "--", "--fetch-dependencies"],
        cwd=root,
        check=True,
        creationflags=spawn_flags(),
    )
    subprocess.run(["mise", "-C", "sdk/cpp", "run", "build"], cwd=root, check=True, creationflags=spawn_flags())
    native = root / "target/sdk/cpp/tests"
    if os.name == "nt":
        native /= "Debug"
    shutil.copy2(native / f"dokime-system-suite{extension}", output / f"dokime-system-suite{extension}")

    # Extend The Tests Producer's Existing Fixture Inventory.
    stamp = paths.output(root, "build", profile) / "fixtures/stamp.json"
    record = json.loads(stamp.read_text(encoding="utf-8"))
    files = [*record["files"], *(path.relative_to(root).as_posix() for path in output.rglob("*") if path.is_file())]
    archive.stamp(root, profile, files, record["metadata"])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=configuration.PROFILES, default="dev")
    build(parser.parse_args().profile)
