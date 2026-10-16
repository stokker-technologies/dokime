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
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
from pathlib import Path, PurePosixPath
from zipfile import ZIP_DEFLATED, ZipFile

# Third Party Imports
# Local Imports
from dokime_tools import contracts
from dokime_tools.check.release import (
    AUDITS,
    BINARIES,
    COVERAGE_EVIDENCE,
    EXAMPLE_REPORT,
    EXAMPLES,
    FIXTURE_BUNDLES,
    PROTO,
    REPORT_TEMPLATE,
    TARGETS,
    TEST_ARCHIVES,
    TEST_EVIDENCE,
    VALIDATION,
    _passed_tests,
    build_profile,
    digest,
    inputs,
    required_suites,
    verify_platform,
    verify_test_archives,
)
from dokime_tools.check.wheel import verify_wheel
from dokime_tools.ci.artifacts import job_id
from dokime_tools.fixtures import FIXTURES, STAMP
from dokime_tools.package.wheel import select_wheel

# Type Checking Imports


# === Public Exports === #

# === Constants === #
PLATFORMS = ("linux", "windows")
CONTRACT_ARCHIVE = Path("target/contracts/dokime-contract.tar")
ASSETS = Path("dist/assets")
INSTALLER_ROLES = frozenset({"deb", "rpm", "installer", "signature", "updater"})
PYTHON_ROLES = frozenset({"wheel", "sdist"})


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
    audits = verify_wheel(products["wheel"][0])
    protocol = _protocol(root)
    build_job_id = job_id(f"build:{build_profile()}", platform())
    archives = _test_archives(root, build_job_id)
    with tempfile.TemporaryDirectory(dir=destination.parent) as temporary:
        staging = Path(temporary) / platform()
        staging.mkdir()
        files = []
        for role, (source, relative) in products.items():
            target = staging / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
            files.append(_entry(staging, target, role))
        for name, data in protocol.items():
            path = staging / PROTO / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            files.append(_entry(staging, path, f"proto:{name}"))
        for role, contents in audits.items():
            path = staging / AUDITS / f"{role}.json"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents, encoding="utf-8")
            files.append(_entry(staging, path, role))
        for source in sorted((root / "target/cargo-timings").glob("cargo-timing.html")):
            path = staging / VALIDATION / source.name
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, path)
            files.append(_entry(staging, path, f"timing:{source.name}"))
        manifest = {
            "version": 1,
            "platform": platform(),
            "architecture": "x86_64",
            "profile": build_profile(),
            "target": TARGETS[platform()],
            "python": "py3",
            "abi": "none",
            "package_version": tomllib.loads(
                (root / "Cargo.toml").read_text(encoding="utf-8")
            )["workspace"]["package"]["version"],
            "toolchain": _output(root, ["rustc", "--version", "--verbose"]),
            "build_revision": _output(root, ["git", "rev-parse", "HEAD"]),
            "build_job_id": build_job_id,
            "built_unix_seconds": int(time.time()),
            "pipeline_id": os.environ.get("GITHUB_RUN_ID"),
            "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
            "inputs": identity,
            "contracts": {"archive_sha256": digest(root / CONTRACT_ARCHIVE)},
            "test_archives": archives,
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
    _artifact_output(archives["artifact"], key="archives_artifact_name")
    if build_profile() == "release":
        stage_assets(
            root, manifest, INSTALLER_ROLES, ASSETS / platform() / "installers"
        )
        _artifact_output(
            f"installers-{platform()}-{build_job_id}", key="installers_artifact_name"
        )
        if platform() == "linux":
            stage_assets(root, manifest, PYTHON_ROLES, ASSETS / "python")
            _artifact_output(f"python-{build_job_id}", key="python_artifact_name")


def verify(root: Path) -> dict:
    manifest = verify_platform(
        directory(root), platform(), root, expected_profile=build_profile()
    )
    _report_template(root, manifest)
    return manifest


def verify_archives(root: Path) -> None:
    """Admit only the candidate's own downloaded Nextest archives into a lane."""
    verify_test_archives(root, verify(root))


def prepare_archives(root: Path) -> None:
    from dokime_tools import test_archive

    destination = root / TEST_ARCHIVES
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.is_symlink():
        raise ValueError("test archive destination must not be a symlink")
    with tempfile.TemporaryDirectory(
        prefix="test-archives-", dir=destination.parent
    ) as temporary:
        staged = Path(temporary) / "archives"
        test_archive.build(root, staged, all_features=False, coverage=False)
        if destination.exists():
            shutil.rmtree(destination)
        staged.replace(destination)


def collect_tests(root: Path) -> None:
    from dokime_tools import evidence

    inputs_directory = root / "dist/evidence-downloads"
    lanes = sorted(inputs_directory.iterdir()) if inputs_directory.is_dir() else []
    expected = {
        f"evidence-{platform()}-{suite}" for suite in required_suites(platform())
    }
    if {path.name for path in lanes} != expected or any(
        not path.is_dir() or path.is_symlink() for path in lanes
    ):
        raise ValueError("required native correctness lanes are missing or unexpected")
    evidence.collect(root, lanes)


def validate(root: Path) -> None:
    manifest = verify(root)
    original_manifest = digest(directory(root) / "manifest.json")
    coverage = None
    coverage_directory = root / "dist/coverage" / platform()
    if os.environ.get("GITHUB_ACTIONS") == "true" or coverage_directory.is_dir():
        from dokime_tools.ci.coverage import verify_proof

        coverage = verify_proof(root, coverage_directory, manifest)
    evidence = root / ".stkr/tests"
    results = json.loads((evidence / "latest.json").read_text(encoding="utf-8"))
    for suite in required_suites(platform()):
        entry = results.get("tests", {}).get(suite, {})
        if (
            entry.get("outcome") != "passed"
            or entry.get("coverage") is not False
            or entry.get("revision") != _output(root, ["git", "rev-parse", "HEAD"])
            or entry.get("platform") != platform()
            or (
                suite != "integration-javascript"
                and entry.get("started_unix_seconds", 0)
                < manifest["built_unix_seconds"]
            )
            or entry.get("requested_configuration") is not None
        ):
            raise ValueError(f"missing or failed required evidence: {suite}")
        _passed_tests(evidence / f"tests-{suite}.xml")
    destination = directory(root)
    manifest["files"] = [
        entry for entry in manifest["files"] if not entry["role"].startswith("test:")
    ]
    if (destination / TEST_EVIDENCE).exists():
        shutil.rmtree(destination / TEST_EVIDENCE)
    for source in sorted(evidence.iterdir()):
        if source.is_file():
            path = destination / TEST_EVIDENCE / source.name
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, path)
            manifest["files"].append(_entry(destination, path, f"test:{source.name}"))
    manifest["validation"] = {
        "test_job_id": job_id("test", platform()),
        "pipeline_id": os.environ.get("GITHUB_RUN_ID"),
        "suites": list(required_suites(platform())),
        "build_manifest_sha256": original_manifest,
        "contract_job_id": job_id(
            f"contracts:prepare:{build_profile()}", platform(), completed=True
        ),
        "lane_job_ids": {
            suite: job_id("test:" + suite.replace("-", ":"), platform(), completed=True)
            for suite in required_suites(platform())
        },
    }
    example = root / "dist/examples" / platform() / EXAMPLE_REPORT
    if not example.is_file() or example.is_symlink():
        raise ValueError(f"missing acceptance example report: {example}")
    if not example.read_text(encoding="utf-8").strip():
        raise ValueError("acceptance example report is empty")
    if (destination / EXAMPLES).exists():
        shutil.rmtree(destination / EXAMPLES)
    path = destination / EXAMPLES / EXAMPLE_REPORT
    path.parent.mkdir(parents=True)
    shutil.copy2(example, path)
    manifest["files"] = [
        entry for entry in manifest["files"] if not entry["role"].startswith("example:")
    ]
    manifest["files"].append(_entry(destination, path, f"example:{EXAMPLE_REPORT}"))
    if coverage is not None:
        manifest["validation"]["coverage_job_id"] = coverage["coverage_job_id"]
        for source in sorted(coverage_directory.iterdir()):
            if source.is_file():
                path = destination / COVERAGE_EVIDENCE / source.name
                path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, path)
                manifest["files"].append(
                    _entry(destination, path, f"coverage:{source.name}")
                )
    _write(destination / "manifest.json", manifest)
    verify(root)
    export(root)
    test_job_id = manifest["validation"]["test_job_id"]
    _artifact_output(f"validation-{platform()}-{test_job_id}")
    stage_assets(
        root, manifest, INSTALLER_ROLES, ASSETS / platform() / "installers-validated"
    )
    _artifact_output(
        f"installers-validated-{platform()}-{test_job_id}",
        key="installers_validated_artifact_name",
    )


def export(root: Path) -> None:
    package = directory(root)
    target = root / "dist/downloads" / f"dokime-{platform()}.zip"
    target.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(target, "w", ZIP_DEFLATED) as archive:
        for path in sorted(package.rglob("*")):
            if path.is_file():
                archive.write(path, path.relative_to(package).as_posix())


def stage_assets(
    root: Path, manifest: dict, roles: frozenset[str], destination: Path
) -> Path:
    """Flat copies of the selected package roles, each byte-identical to the manifest."""
    package = directory(root)
    staging = root / destination
    if staging.is_symlink():
        raise ValueError("release asset staging must not be a symlink")
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir(parents=True)
    selected = [entry for entry in manifest["files"] if entry["role"] in roles]
    if not selected:
        raise ValueError("release asset staging selected no package files")
    for entry in selected:
        source = package / entry["path"]
        target = staging / PurePosixPath(entry["path"]).name
        if target.exists():
            raise ValueError("release asset filenames collide")
        if source.is_symlink() or not source.is_file():
            raise ValueError(f"missing release asset: {entry['path']}")
        shutil.copy2(source, target)
        if target.stat().st_size != entry["bytes"] or digest(target) != entry["sha256"]:
            raise ValueError(
                f"release asset differs from the validated manifest: {entry['path']}"
            )
    return staging


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


def _artifact_output(name: str, *, key: str = "artifact_name") -> None:
    destination = os.environ.get("GITHUB_OUTPUT")
    if destination:
        with Path(destination).open("a", encoding="utf-8") as output:
            output.write(f"{key}={name}\n")


def _test_archives(root: Path, build_job_id: str | None) -> dict:
    """Inventory of the Nextest archives `ci:archives:prepare` staged."""
    directory = root / TEST_ARCHIVES
    files = []
    for path in sorted(directory.glob("*")) if directory.is_dir() else []:
        if path.suffix not in {".json", ".zst"} or path.is_symlink():
            raise ValueError(f"unexpected test archive artifact: {path}")
        files.append(
            {"name": path.name, "bytes": path.stat().st_size, "sha256": digest(path)}
        )
    if not files:
        raise ValueError("release preparation omitted Nextest archives")
    return {"artifact": f"test-archives-{platform()}-{build_job_id}", "files": files}


def _products(root: Path) -> dict[str, tuple[Path, str]]:
    """Native build outputs keyed by role, with their package-relative paths."""
    suffix = ".exe" if platform() == "windows" else ""
    profile = build_profile()
    cargo_directory = "debug" if profile == "dev" else "release"
    python_directory = "python-dev" if profile == "dev" else "python"
    products = {
        "license": (root / "LICENSE-APACHE", "licenses/LICENSE-APACHE"),
        "report-template": (
            root / "crates/dokime-app/dist-report/report.html",
            REPORT_TEMPLATE,
        ),
        "plugin": (
            root / "target/fixtures/mini-sidecar/mini-sidecar.dokime",
            "plugins/mini-sidecar.dokime",
        ),
    }
    for role, name in (
        ("host", "dokime-server"),
        ("client", "dokime"),
        ("tool", "cargo-dokime"),
        ("desktop", "dokime-gui"),
    ):
        products[role] = (
            root / f"target/{cargo_directory}/{name}{suffix}",
            f"{BINARIES}/{name}{suffix}",
        )
    distributions = [("wheel", "dokime-*.whl")]
    if profile == "release":
        distributions.append(("sdist", "dokime-*.tar.gz"))
    for role, pattern in distributions:
        source = (
            select_wheel(root / "dist" / python_directory, pattern)
            if role == "wheel"
            else _one(root / "dist" / python_directory, pattern)
        )
        products[role] = (source, f"python/{source.name}")
    installers = root / "dist/tauri" / f"{platform()}-x86_64"
    patterns = (
        (("deb", "*.deb"), ("rpm", "*.rpm"))
        if platform() == "linux"
        else (("installer", "*setup.exe"),)
    )
    for role, pattern in patterns if profile == "release" else ():
        source = _one(installers, pattern)
        products[role] = (source, f"installers/{source.name}")
    if profile == "release" and platform() == "windows":
        for role, pattern in (("signature", "*.exe.sig"), ("updater", "latest.json")):
            if list(installers.glob(pattern)):
                source = _one(installers, pattern)
                products[role] = (source, f"installers/{source.name}")
    for name, _, _ in FIXTURES:
        products[f"fixture:{name}"] = (
            root / "target/fixtures" / name / f"{name}.dokime",
            f"{FIXTURE_BUNDLES}/{name}/{name}.dokime",
        )
    # Restored bundles keep the stamp that tells tests which contracts built them.
    products[f"fixture:{STAMP}"] = (
        root / "target/fixtures" / STAMP,
        f"{FIXTURE_BUNDLES}/{STAMP}",
    )
    for source, _ in products.values():
        if not source.is_file() or source.is_symlink():
            raise ValueError(f"missing native build artifact: {source}")
    return products


def _protocol(root: Path) -> dict[str, bytes]:
    """Unpacked portable contract files whose archive the contracts job published."""
    archive = root / CONTRACT_ARCHIVE
    checksum = archive.with_suffix(".tar.sha256")
    if not archive.is_file() or archive.is_symlink() or checksum.is_symlink():
        raise ValueError(f"missing native build artifact: {archive}")
    if checksum.read_text(encoding="ascii") != f"{digest(archive)}  {archive.name}\n":
        raise ValueError("portable contract archive checksum differs")
    members = contracts.members(archive)
    revision = _output(root, ["git", "rev-parse", "HEAD"])
    if members["revision.txt"] != (revision + "\n").encode("ascii"):
        raise ValueError("portable contracts belong to another source commit")
    return members


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
    parser.add_argument(
        "command",
        choices=(
            "collect",
            "verify",
            "verify-archives",
            "validate",
            "collect-tests",
            "prepare-archives",
        ),
    )
    arguments = parser.parse_args()
    {
        "collect": collect,
        "verify": verify,
        "verify-archives": verify_archives,
        "validate": validate,
        "collect-tests": collect_tests,
        "prepare-archives": prepare_archives,
    }[arguments.command](Path.cwd())
