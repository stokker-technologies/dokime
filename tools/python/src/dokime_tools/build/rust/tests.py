# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build The Shared Test Archive And Its Fixtures Without Replacing Product Exports"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path
from time import perf_counter

# Third Party Imports
# Local Imports
from dokime_tools.build import cargo, configuration, frontend, outputs, paths
from dokime_tools.package import plugins
from dokime_tools.test import archive, policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str, *, extended: bool = False, timings: bool = False) -> list[str]:
    """Compile The Independent Canonical Test Archive And Its Metadata."""
    if profile not in configuration.PROFILES:
        raise ValueError("test archives require dev or release")
    features = configuration.test_features(profile, extended=extended)

    # Compile Test Targets Through The Shared Archive Owner.
    started = perf_counter()
    print(f"[build:tests] Archive started profile={profile} features={features}", flush=True)
    built = cargo.archive(ROOT, profile, extra=[*features, *(["--timings"] if timings else [])])
    print(f"[build:tests] Archive finished in {perf_counter() - started:.2f}s", flush=True)

    metadata = built.with_name("metadata.json")
    with metadata.open("w", encoding="utf-8") as written:
        command = ["cargo", "metadata", "--locked", "--format-version", "1", *features]
        subprocess.run(
            command,
            cwd=ROOT,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    messages = built.with_name("compiler.json")
    staged = [built, messages, metadata]
    output = paths.output(ROOT, "build", profile)
    # Package Fixtures With The Archive's Own Collector, Never A Separately Staged Product Build.
    tool = outputs.executables(messages, ["cargo-dokime"])["cargo-dokime"]
    staged += plugins.prepare(ROOT, messages, profile, tool=tool, output=output / "fixtures/plugins", packages=plugins.FIXTURES)
    # Transfer The Pinned Native Protobuf Compiler With The Archived Tests.
    document = json.loads(metadata.read_text(encoding="utf-8"))
    platform = {"linux-x86_64": "linux-x86_64", "windows-x86_64": "win32"}[paths.target()]
    prefix = f"protoc-bin-vendored-{platform}"
    package = next(package for package in document["packages"] if package["name"] == prefix)
    source = Path(package["manifest_path"]).parent
    destination = output / "fixtures/inputs/vendor" / f"{prefix}-{package['version']}"
    shutil.copytree(source, destination, dirs_exist_ok=True)
    staged += [path for path in destination.rglob("*") if path.is_file()]
    names = configuration.relative(staged)
    archive.stamp(ROOT, profile, names)
    return names


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=configuration.PROFILES, default="dev")
    parser.add_argument("--extended", action="store_true", help="include local-only fault-injection stories")
    parser.add_argument("--timings", action="store_true")
    options = parser.parse_args(arguments)
    try:
        build(options.profile, extended=options.extended, timings=options.timings)
    except (policy.TestError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build tests: {error}", file=sys.stderr)
        return 1
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
