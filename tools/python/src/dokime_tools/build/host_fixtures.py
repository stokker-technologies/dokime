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
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.test.policy
from dokime_tools.build import cargo, configuration, frontend, paths
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
    "dokime-rust-history",
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

    # Prepare the Report for Focused Rust Fixtures Too.
    if "rust" in languages and not (root / "target/generated/report/report.html").is_file():
        subprocess.run(
            ["mise", "-C", "crates/dokime-gui", "run", "generate:host-frontend"],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        frontend.build_report([])

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
                *[argument for name in ((fixture,) if fixture else RUST_FIXTURES) for argument in ("--bin", name)],
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

    # Build the Test-Only Content Fixture.
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
                "dokime-client-internal",
                "--bin",
                "content_fixture",
            ],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        shutil.copy2(root / "target" / compiler_profile / f"content_fixture{extension}", output / f"content_fixture{extension}")

        # Stage the Matching Product Host and Plugin Without Recompiling Them.
        product = paths.output(root, "build", profile) / "bin"
        for name in ("dokime-host", "plugin-obc"):
            source = product / f"{name}{extension}"
            if not source.is_file():
                raise ValueError(f"Stage Host Fixtures: Run build:product --profile {profile} or Download Its build-cli Artifact: {source}")
            shutil.copy2(source, output / source.name)

        # Build Both Conformance Fixtures From the Resolved Library Source.
        source = _library_source(root)
        reference = _reference(root, source, profile)
        shutil.copy2(reference, output / reference.name)
        sync_output(source / "sdk/python/tests/fixtures/conformance", output / "python/conformance")

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

    # Build the C++ Fixture Against the Bundled Release SDK Package, as an Author's Project Would.
    # The release preset keeps it within the bundle bound.
    if "cpp" in languages:
        subprocess.run(
            ["mise", "-C", "sdk/cpp", "run", "build", "--", "--preset", "release", "--target", "dokime_client"],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        subprocess.run(
            ["mise", "-C", "sdk/cpp", "run", "package", "--", "--preset", "release"],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        subprocess.run(
            [
                "mise",
                "-C",
                "sdk/cpp",
                "run",
                "consumer",
                "--",
                "--preset",
                "release",
                "--source",
                str(root / "sdk/tests/fixtures/cpp/thermal"),
            ],
            cwd=root,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        native = root / "target/cp/release/consumers/thermal"
        shutil.copy2(native / f"dokime-system-suite{extension}", output / f"dokime-system-suite{extension}")

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


def prepare_reference(profile: str) -> Path:
    """Build the Reference Host Used by Both SDK and Shared Integration Fixtures."""
    root = configuration.ROOT
    source = _library_source(root)
    return _reference(root, source, profile)


# === Private Functions === #


def _library_source(root: Path) -> Path:
    metadata = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
        creationflags=spawn_flags(),
    )
    packages = [package for package in json.loads(metadata.stdout)["packages"] if package["name"] == "harmos"]
    if len(packages) != 1:
        raise ValueError("Prepare Reference Host: Resolve Exactly One Harmos Package in the Workspace")
    return Path(packages[0]["manifest_path"]).parents[2]


def _reference(root: Path, source: Path, profile: str) -> Path:
    subprocess.run(
        [
            "cargo",
            "install",
            "--path",
            str(source / "tools/conformance"),
            "--locked",
            *(["--debug"] if profile == "dev" else ["--profile", profile]),
            "--force",
            "--root",
            str(root / "target/reference-host"),
            "--bin",
            "harmos-conformance",
            "--jobs",
            os.environ.get("CARGO_BUILD_JOBS", "6"),
            "--target-dir",
            str(root / "target"),
        ],
        cwd=root,
        env=cargo.compiler_environment(profile),
        check=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    extension = ".exe" if os.name == "nt" else ""
    return root / "target/reference-host/bin" / f"harmos-conformance{extension}"


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
    parser.add_argument("--reference-only", action="store_true", help="Build Only the Shared Reference Host")
    options = parser.parse_args()
    if options.reference_only:
        prepare_reference(options.profile)
    else:
        build(options.profile, options.language, options.fixture)
    if options.export:
        transport.send(options.profile)
