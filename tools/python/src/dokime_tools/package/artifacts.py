# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Collect And Verify Immutable Native Packages"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
from zipfile import ZIP_DEFLATED, ZipFile

# Third Party Imports

# Local Imports
from dokime_tools.ci.artifacts import job_id
from dokime_tools.check.native_wheel import verify_native_wheel
from dokime_tools.check.release import (
    inputs,
    TARGETS,
    _passed_tests,
    digest,
    verify_platform,
)
from dokime_tools.package.wheel import select_wheel

# Type Checking Imports


# === Public Exports === #

# === Constants === #
PLATFORMS = ("linux", "windows")
SUITES = (
    "integration-rust",
    "integration-python",
    "integration-javascript",
    "acceptance-rust",
    "acceptance-python",
)


# === Types === #

# === Classes === #

# === Public Functions === #


def directory(root: Path) -> Path:
    return root / "dist/build" / platform()


def collect(root: Path) -> None:
    if any(
        "instrument-coverage" in os.environ.get(name, "")
        for name in (
            "RUSTFLAGS",
            "CARGO_ENCODED_RUSTFLAGS",
        )
    ) or os.environ.get("LLVM_PROFILE_FILE"):
        raise ValueError("release packages must be uninstrumented")
    identity = inputs(root)
    if not identity["source_tree"]:
        changes = subprocess.check_output(
            ["git", "status", "--porcelain", "--untracked-files=normal"],
            cwd=root,
            text=True,
            encoding="utf-8",
        )
        raise ValueError(
            f"immutable packages require committed, clean source:\n{changes}"
        )
    destination = directory(root)
    destination.parent.mkdir(parents=True, exist_ok=True)
    products = _products(root)
    audits = verify_native_wheel(products["wheel"][0])
    with tempfile.TemporaryDirectory(dir=destination.parent) as temporary:
        staging = Path(temporary) / platform()
        staging.mkdir()
        files = []
        for role, (source, owner) in products.items():
            target = staging / owner / source.name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
            files.append(_entry(staging, target, role))
        for role, contents in audits.items():
            path = staging / "reports" / f"{role}.json"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents, encoding="utf-8")
            files.append(_entry(staging, path, role))
        for source in sorted((root / "target/cargo-timings").glob("cargo-timing.html")):
            path = staging / "reports" / source.name
            shutil.copy2(source, path)
            files.append(_entry(staging, path, f"timing:{source.name}"))
        manifest = {
            "version": 1,
            "platform": platform(),
            "architecture": "x86_64",
            "profile": "release",
            "target": TARGETS[platform()],
            "python": "cp313",
            "abi": "abi3",
            "package_version": tomllib.loads(
                (root / "Cargo.toml").read_text(encoding="utf-8")
            )["workspace"]["package"]["version"],
            "toolchain": _output(root, ["rustc", "--version", "--verbose"]),
            "build_revision": _output(root, ["git", "rev-parse", "HEAD"]),
            "build_job_id": job_id("build:release", platform()),
            "built_unix_seconds": int(time.time()),
            "pipeline_id": os.environ.get("GITHUB_RUN_ID"),
            "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
            "inputs": identity,
            "tools": {
                name: _output(root, [name, "--version"])
                for name in (
                    "mise",
                    "rustc",
                    "cargo",
                    "python",
                    "uv",
                    "node",
                    "pnpm",
                )
            },
            "files": files,
        }
        _write(staging / "manifest.json", manifest)
        if destination.exists():
            shutil.rmtree(destination)
        staging.rename(destination)
    verify(root)
    export(root)
    _artifact_output(f"build-{platform()}-{manifest['build_job_id']}")


def verify(root: Path) -> dict:
    manifest = verify_platform(directory(root), platform(), root)
    _report_template(root, manifest)
    return manifest


def validate(root: Path) -> None:
    manifest = verify(root)
    evidence = root / ".stkr/tests"
    results = json.loads((evidence / "latest.json").read_text(encoding="utf-8"))
    for suite in SUITES:
        entry = results.get("tests", {}).get(suite, {})
        if (
            entry.get("outcome") != "passed"
            or entry.get("revision") != _output(root, ["git", "rev-parse", "HEAD"])
            or entry.get("platform") != platform()
            or entry.get("started_unix_seconds", 0) < manifest["built_unix_seconds"]
            or entry.get("requested_configuration") is not None
        ):
            raise ValueError(f"missing or failed required evidence: {suite}")
        _passed_tests(evidence / f"tests-{suite}.xml")
    destination = directory(root)
    manifest["files"] = [
        entry for entry in manifest["files"] if not entry["role"].startswith("test:")
    ]
    if (destination / "tests").exists():
        shutil.rmtree(destination / "tests")
    for source in sorted(evidence.iterdir()):
        if source.is_file():
            path = destination / "tests" / source.name
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, path)
            manifest["files"].append(_entry(destination, path, f"test:{source.name}"))
    manifest["validation"] = {
        "test_job_id": job_id("test", platform()),
        "pipeline_id": os.environ.get("GITHUB_RUN_ID"),
        "suites": list(SUITES),
    }
    _write(destination / "manifest.json", manifest)
    verify(root)
    export(root)
    _artifact_output(f"validation-{platform()}-{manifest['validation']['test_job_id']}")


def export(root: Path) -> None:
    package = directory(root)
    target = root / "dist/downloads" / f"dokime-{platform()}.zip"
    target.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(target, "w", ZIP_DEFLATED) as archive:
        for path in sorted(package.rglob("*")):
            if path.is_file():
                archive.write(path, path.relative_to(package).as_posix())


def platform() -> str:
    if sys.platform not in {"linux", "win32"}:
        raise ValueError("native packages require Linux or Windows")
    return "windows" if sys.platform == "win32" else "linux"


# === Private Functions === #


def _report_template(root: Path, manifest: dict) -> None:
    report = root / "crates/dokime-app/dist-report/report.html"
    template = next(
        entry for entry in manifest["files"] if entry["role"] == "report-template"
    )
    if (
        not report.is_file()
        or report.is_symlink()
        or digest(report) != template["sha256"]
    ):
        raise ValueError(
            "canonical report template differs from the release artifact; "
            "run mise run ci:artifacts:restore before testing"
        )


def _artifact_output(name: str) -> None:
    destination = os.environ.get("GITHUB_OUTPUT")
    if destination:
        with Path(destination).open("a", encoding="utf-8") as output:
            output.write(f"artifact_name={name}\n")


def _products(root: Path) -> dict[str, tuple[Path, str]]:
    suffix = ".exe" if platform() == "windows" else ""
    products = {
        "license": (root / "LICENSE-APACHE", "licenses"),
        "host": (root / f"target/release/dokime{suffix}", "runtime"),
        "tool": (root / f"target/release/cargo-dokime{suffix}", "runtime"),
        "desktop": (root / f"target/release/dokime-gui{suffix}", "runtime"),
        "report-template": (
            root / "crates/dokime-app/dist-report/report.html",
            "reports",
        ),
        "plugin": (root / "target/release/plugins/mini-sidecar.dokime", "plugins"),
        "protocol": (root / "target/protocol/dokime-protocol.tar", "protocol"),
        "protocol-checksum": (
            root / "target/protocol/dokime-protocol.tar.sha256",
            "protocol",
        ),
        "wheel": (select_wheel(root / "dist/python", "dokime-*.whl"), "python"),
        "sdist": (_one(root / "dist/python", "dokime-*.tar.gz"), "python"),
    }
    if platform() == "linux":
        products.update(
            deb=(_one(root / "dist/tauri/linux-x86_64", "*.deb"), "installers"),
            rpm=(_one(root / "dist/tauri/linux-x86_64", "*.rpm"), "installers"),
            **{"wheel-build": (root / "dist/python/wheel-build.json", "reports")},
        )
    else:
        source = root / "dist/tauri/windows-x86_64"
        products["installer"] = (_one(source, "*setup.exe"), "installers")
        for role, pattern in (("signature", "*.exe.sig"), ("updater", "latest.json")):
            if list(source.glob(pattern)):
                products[role] = (_one(source, pattern), "installers")
    for source, _ in products.values():
        if not source.is_file() or source.is_symlink():
            raise ValueError(f"missing native build artifact: {source}")
    return products


def _one(root: Path, pattern: str) -> Path:
    paths = list(root.glob(pattern))
    if len(paths) != 1:
        raise ValueError(f"expected exactly one {pattern} in {root}")
    return paths[0]


def _entry(root: Path, path: Path, role: str) -> dict:
    return {
        "role": role,
        "path": path.relative_to(root).as_posix(),
        "bytes": path.stat().st_size,
        "sha256": digest(path),
    }


def _output(root: Path, command: list[str]) -> str:
    executable = shutil.which(command[0])
    if executable is None:
        raise FileNotFoundError(
            f"{command[0]} is not installed; install it through mise"
        )
    return subprocess.run(
        [executable, *command[1:]],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
        encoding="utf-8",
    ).stdout.strip()


def _write(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


# === Entry Point === #
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("collect", "verify", "validate"))
    arguments = parser.parse_args()
    {"collect": collect, "verify": verify, "validate": validate}[arguments.command](
        Path.cwd()
    )
