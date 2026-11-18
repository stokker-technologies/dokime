# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Build the Shared Test Archive and Its Fixtures Without Replacing Product Exports"

# === Imports === #

# Standard Library Imports
import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path
from time import perf_counter

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.test.policy
from dokime_tools.build import cargo, configuration, paths
from dokime_tools.prepare import generated
from dokime_tools.test import archive, policy

from ...process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str, *, timings: bool = False) -> list[str]:
    """Compile the Independent Canonical Test Archive and Its Metadata."""
    if profile not in dokime_tools.test.policy.PROFILES:
        raise ValueError("Build Tests: test archives require dev or release")

    # Generate the Report Template Before Compiling Its Runtime Consumer.
    generated.prepare()

    # Compile Test Targets Through The Shared Archive Owner.
    started = perf_counter()
    print(f"[build:tests] Archive started profile={profile}", flush=True)
    built = cargo.archive(configuration.ROOT, profile, extra=["--features", "dokime-gui/testing", *(["--timings"] if timings else [])])
    print(f"[build:tests] Archive finished in {perf_counter() - started:.2f}s", flush=True)

    metadata = built.with_name("metadata.json")
    written = metadata.open("w", encoding="utf-8")
    try:
        command = ["cargo", "metadata", "--locked", "--format-version", "1"]
        subprocess.run(
            command,
            cwd=configuration.ROOT,
            stdout=written,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    finally:
        written.close()
    messages = built.with_name("compiler.json")
    staged = [built, messages, metadata]
    output = paths.output(configuration.ROOT, "build", profile)
    # Transfer The Pinned Native Protobuf Compiler With The Archived Tests.
    document = json.loads(metadata.read_text(encoding="utf-8"))
    platform = {"linux-x86_64": "linux-x86_64", "windows-x86_64": "win32"}[paths.target()]
    prefix = f"protoc-bin-vendored-{platform}"
    package = next(package for package in document["packages"] if package["name"] == prefix)
    source = Path(package["manifest_path"]).parent
    destination = output / "fixtures/inputs/vendor" / f"{prefix}-{package['version']}"
    shutil.copytree(source, destination, dirs_exist_ok=True)
    staged_items = []
    for staged_path in destination.rglob("*"):
        if not (staged_path.is_file()):
            continue
        staged_items.append(staged_path)
    staged += staged_items
    names = configuration.relative(staged)
    archive.stamp(configuration.ROOT, profile, names)
    return names


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=dokime_tools.test.policy.PROFILES, default="dev")
    parser.add_argument("--timings", action="store_true")
    options = parser.parse_args(arguments)
    try:
        build(options.profile, timings=options.timings)
    except (policy.TestError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build tests: {error}", file=sys.stderr)
        return 1
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
