# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Build and Export the Shared Execution Host Fixtures Through SDK Tasks."

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

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.test.policy
from dokime_tools.build import cargo, configuration, paths
from dokime_tools.ci import transport
from dokime_tools.package.staging import sync_output
from dokime_tools.prepare import generated
from dokime_tools.process import spawn_flags
from dokime_tools.test import archive

# Type Checking Imports

# === Public Exports === #

# === Constants === #

RUST_FIXTURES = (
    "dokime-rust-thermal",
    "dokime-rust-execution",
    "dokime-rust-boundary",
    "dokime-rust-secrets",
    "dokime-rust-capture",
    "dokime-rust-plugin",
)

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str, language: str = "auto", fixture: str | None = None) -> None:
    if fixture is not None and (fixture not in RUST_FIXTURES or language not in ("auto", "rust")):
        raise ValueError("Build Host Fixture: Select a Declared Rust Fixture With --language rust")
    root = configuration.ROOT
    output = paths.output(root, "build", profile) / "fixtures/host"
    output.mkdir(parents=True, exist_ok=True)
    extension = ".exe" if os.name == "nt" else ""
    compiler_profile = "debug" if profile == "dev" else profile

    languages = {"rust"} if fixture else {language}
    if language == "auto" and fixture is None:
        languages = {"rust", "python"}
        if shutil.which("dotnet"):
            languages.add("csharp")
        if shutil.which("cmake"):
            languages.add("cpp")

    # Prepare Product Resources for the Complete Rust Producer.
    if "rust" in languages and fixture is None:
        generated.prepare()

    # Build the Rust SDK Fixtures.
    if "rust" in languages:
        subprocess.run(
            [
                "mise",
                "run",
                "build",
                "--",
                "--cargo",
                "build",
                "--locked",
                "--manifest-path",
                "sdk/rust/tests/consumer/Cargo.toml",
                "--package",
                "dokime-rust-stories",
                *(["--bin", fixture] if fixture else ["--bins"]),
                "--profile",
                profile,
            ],
            cwd=root,
            env=dict(cargo.compiler_environment(profile), CARGO_TARGET_DIR=str(root / "target")),
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        for name in (fixture,) if fixture else RUST_FIXTURES:
            shutil.copy2(root / "target" / compiler_profile / f"{name}{extension}", output / f"{name}{extension}")

    # Build the Remaining Rust Host Fixtures Only for the Complete Producer.
    if "rust" in languages and fixture is None:
        subprocess.run(
            [
                "mise",
                "run",
                "build",
                "--",
                "--cargo",
                "build",
                "--locked",
                "--profile",
                profile,
                "--package",
                "dokime-runtime",
                "--package",
                "dokime-client-internal",
                "--bin",
                "content_fixture",
                "--bin",
                "dokime-host",
            ],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        shutil.copy2(root / "target" / compiler_profile / f"content_fixture{extension}", output / f"content_fixture{extension}")
        shutil.copy2(root / "target" / compiler_profile / f"dokime-host{extension}", output / f"dokime-host{extension}")
        # Build the OBC Charger Plugin the Desktop Stories and Screenshots Install
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
                "plugin-obc",
                "--bin",
                "plugin-obc",
                "--profile",
                profile,
            ],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        shutil.copy2(root / "target" / compiler_profile / f"plugin-obc{extension}", output / f"plugin-obc{extension}")

        # Build the Reference Host From the Same Library Revision as the Product.
        # Its library crates are path dependencies of the pinned checkout. Cargo fingerprints their workspace-relative paths,
        # so each revision builds in its own target directory rather than reusing another revision's artifacts.
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
                "--profile",
                profile,
                "--force",
                "--root",
                str(root / "target/reference-host"),
                "--target-dir",
                str(root / "target/reference-host/build" / library["rev"]),
                "--bin",
                "harmos-conformance",
                "harmos-conformance",
            ],
            cwd=root,
            env=cargo.compiler_environment(profile),
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        shutil.copy2(root / "target/reference-host/bin" / f"harmos-conformance{extension}", output / f"harmos-conformance{extension}")

        # Stage the Python Fixture From the Same Locked Source as the Reference Host.
        metadata = subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            cwd=root,
            capture_output=True,
            text=True,
            check=True,
            creationflags=spawn_flags(),
        )
        package = next(
            (
                package
                for package in json.loads(metadata.stdout)["packages"]
                if package["name"] == "harmos" and (package.get("source") or "").endswith("#" + library["rev"])
            ),
            None,
        )
        if package is None:
            raise ValueError("Stage Python Conformance Fixture: Resolve the Pinned Harmos Package")
        source = Path(package["manifest_path"]).parents[2] / "sdk/python/tests/fixtures/conformance"
        sync_output(source, output / "python/conformance")

    # Publish the C# Fixture With Its Runtime and Local SDK Package.
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

    # Build the C++ Fixture With the Pinned Native Dependency Graph.
    # The release preset keeps it within the bundle bound.
    if "cpp" in languages:
        directory = root / "target/cp/release"
        subprocess.run(
            ["mise", "-C", "sdk/cpp", "run", "prepare"],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        subprocess.run(
            ["mise", "-C", "sdk/cpp", "run", "build", "--", "--preset", "release", "--target", "dokime-system-suite"],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        native = directory / "tests"
        shutil.copy2(native / f"dokime-system-suite{extension}", output / f"dokime-system-suite{extension}")
        for library in native.glob("*.dll"):
            shutil.copy2(library, output / library.name)

    # Build the Pure Python Fixture Without a Host Dependency.
    if "python" in languages:
        subprocess.run(
            ["mise", "-C", "sdk/python", "run", "build"],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            env=dict(os.environ, DOKIME_BUILD_PROFILE=profile),
        )

    # Extend the Tests Producer's Existing Fixture Inventory.
    stamp = paths.output(root, "build", profile) / "fixtures/stamp.json"
    if language == "auto" or stamp.is_file():
        record = json.loads(stamp.read_text(encoding="utf-8")) if stamp.is_file() else {"files": {}, "metadata": {}}
        files = [*record["files"], *(path.relative_to(root).as_posix() for path in output.rglob("*") if path.is_file())]
        archive.stamp(root, profile, files, record["metadata"])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=dokime_tools.test.policy.PROFILES, default="dev")
    parser.add_argument(
        "--language",
        choices=("auto", "none", "rust", "python", "csharp", "cpp"),
        default=os.environ.get("DOKIME_HOST_FIXTURE_LANGUAGE", "auto"),
    )
    parser.add_argument("--export", action="store_true", help="Export the Declared CI Artifact")
    parser.add_argument("--fixture", choices=RUST_FIXTURES, help="Build and Stage Only One Rust Fixture")
    options = parser.parse_args()
    build(options.profile, options.language, options.fixture)
    if options.export:
        transport.send(options.profile)
