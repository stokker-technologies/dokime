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
import sys
import tomllib
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import cargo, configuration, paths, wheel
from dokime_tools.ci import transport
from dokime_tools.process import spawn_flags
from dokime_tools.test import archive

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str, language: str = "auto") -> None:
    root = configuration.ROOT
    output = paths.output(root, "build", profile) / "fixtures/host"
    output.mkdir(parents=True, exist_ok=True)
    extension = ".exe" if os.name == "nt" else ""

    languages = {language}
    if language == "auto":
        languages = {"rust", "python"}
        if shutil.which("dotnet"):
            languages.add("csharp")
        if shutil.which("cmake"):
            languages.add("cpp")

    # Build The Rust Reference And Execution Fixtures.
    if "rust" in languages:
        subprocess.run(
            ["mise", "-C", "sdk/rust", "run", "sdk:build"],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        for name in ("dokime-rust-thermal", "dokime-rust-execution"):
            shutil.copy2(root / "target/debug" / f"{name}{extension}", output / f"{name}{extension}")

        subprocess.run(
            [
                "mise",
                "run",
                "build",
                "--",
                "--cargo",
                "build",
                "--locked",
                "--package",
                "dokime-runtime-next",
                "--example",
                "content_fixture",
                "--bin",
                "dokime-host-next",
            ],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        shutil.copy2(root / "target/debug/examples" / f"content_fixture{extension}", output / f"content_fixture{extension}")
        shutil.copy2(root / "target/debug" / f"dokime-host-next{extension}", output / f"dokime-host-next{extension}")

        # Build The Reference Host From The Same Library Revision As The Product.
        manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
        library = manifest["workspace"]["dependencies"]["harmos"]
        subprocess.run(
            [
                "cargo",
                "install",
                "--git",
                library["git"],
                "--rev",
                library["rev"],
                "--locked",
                "--debug",
                "--force",
                "--root",
                str(root / "target/reference-host"),
                "--target-dir",
                str(root / "target"),
                "--bin",
                "harmos-conformance",
                "harmos-conformance",
            ],
            cwd=root,
            env=cargo.compiler_environment("dev"),
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        shutil.copy2(root / "target/reference-host/bin" / f"harmos-conformance{extension}", output / f"harmos-conformance{extension}")

    # Publish The C# Fixture With Its Runtime And Local SDK Package.
    if "csharp" in languages:
        subprocess.run(
            ["mise", "-C", "sdk/csharp", "run", "build"],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        subprocess.run(
            ["mise", "-C", "sdk/csharp", "run", "package"],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        subprocess.run(
            ["mise", "-C", "sdk/csharp", "run", "publish:system", "--", str(output / "csharp")],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )

    # Build The C++ Fixture With The Pinned Native Dependency Graph.
    if "cpp" in languages:
        directory = Path(os.environ.get("DOKIME_CPP_BUILD_DIR", root / "target/cpp")).resolve()
        subprocess.run(
            ["mise", "-C", "sdk/cpp", "run", "configure", "--", "--fetch-dependencies", "--build-dir", str(directory)],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        subprocess.run(
            ["mise", "-C", "sdk/cpp", "run", "build", "--", "--target", "dokime-system-suite", "--build-dir", str(directory)],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        native = directory / "tests"
        if os.name == "nt":
            native /= "Debug"
        shutil.copy2(native / f"dokime-system-suite{extension}", output / f"dokime-system-suite{extension}")
        for library in native.glob("*.dll"):
            shutil.copy2(library, output / library.name)

    # Build The Pure Python Fixture Without A Host Dependency.
    if "python" in languages:
        wheel.build(root, profile)

    # Extend The Tests Producer's Existing Fixture Inventory.
    stamp = paths.output(root, "build", profile) / "fixtures/stamp.json"
    if language == "auto" or stamp.is_file():
        record = json.loads(stamp.read_text(encoding="utf-8")) if stamp.is_file() else {"files": {}, "metadata": {}}
        files = [*record["files"], *(path.relative_to(root).as_posix() for path in output.rglob("*") if path.is_file())]
        archive.stamp(root, profile, files, record["metadata"])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=configuration.PROFILES, default="dev")
    parser.add_argument(
        "--language",
        choices=("auto", "none", "rust", "python", "csharp", "cpp"),
        default=os.environ.get("DOKIME_HOST_FIXTURE_LANGUAGE", "auto"),
    )
    parser.add_argument("--export", action="store_true", help="Export the Declared CI Artifact")
    options = parser.parse_args()
    build(options.profile, options.language)
    if options.export:
        transport.send(options.profile)
