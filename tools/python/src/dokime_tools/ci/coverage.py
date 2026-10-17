# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Bind Required Coverage To The Exact Native Build Without A Percentage Gate."""

# === Imports === #

# Standard Library Imports
import json
import os
import shutil
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.test import run as evidence
from dokime_tools.check.release import digest as digest
from dokime_tools.check.release import inputs as inputs
from dokime_tools.ci.artifacts import job_id as job_id
from dokime_tools.package.artifacts import directory as directory
from dokime_tools.package.artifacts import platform as platform
from dokime_tools.package.artifacts import verify as verify

# Type Checking Imports

# === Public Exports === #

# === Constants === #
SUITES = (
    "integration-rust",
    "acceptance-rust",
    "integration-python",
    "acceptance-python",
)

# === Types === #

# === Classes === #

# === Public Functions === #


def collect(root: Path) -> None:
    build = verify(root)
    source = root / ".stkr/tests"
    evidence.collect(root, [source], coverage_required=True)
    result = json.loads((source / "latest.json").read_text(encoding="utf-8"))
    tests = result["tests"]
    if set(tests) != set(SUITES) or any(
        entry.get("started_unix_seconds", 0) < build["built_unix_seconds"]
        for entry in tests.values()
    ):
        raise ValueError("required coverage is missing or predates the exact build")
    destination = root / "dist/coverage" / platform()
    if destination.exists():
        shutil.rmtree(destination)
    shutil.copytree(source, destination)
    proof = {
        "version": 1,
        "platform": platform(),
        "inputs": inputs(root),
        "build_manifest_sha256": digest(directory(root) / "manifest.json"),
        "coverage_job_id": job_id("ci:coverage", platform()),
        "pipeline_id": os.environ.get("GITHUB_RUN_ID"),
        "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
        "suites": list(SUITES),
        "files": [
            {"path": path.name, "bytes": path.stat().st_size, "sha256": digest(path)}
            for path in sorted(destination.iterdir())
            if path.is_file()
        ],
    }
    (destination / "coverage.json").write_text(
        json.dumps(proof, indent=2) + "\n", encoding="utf-8"
    )
    output = os.environ.get("GITHUB_OUTPUT")
    if output:
        with Path(output).open("a", encoding="utf-8") as stream:
            stream.write(
                f"artifact_name=coverage-{platform()}-{proof['coverage_job_id']}\n"
            )


def verify_proof(root: Path, source: Path, build: dict) -> dict:
    path = source / "coverage.json"
    if path.is_symlink() or not path.is_file():
        raise ValueError("required distinct coverage proof is missing")
    proof = json.loads(path.read_text(encoding="utf-8"))
    if (
        proof.get("version") != 1
        or proof.get("platform") != build["platform"]
        or proof.get("inputs") != build["inputs"]
        or proof.get("build_manifest_sha256")
        != digest(directory(root) / "manifest.json")
        or proof.get("pipeline_id") != build["pipeline_id"]
        or proof.get("run_attempt") != build["run_attempt"]
        or set(proof.get("suites", [])) != set(SUITES)
    ):
        raise ValueError("coverage proof belongs to different native inputs")
    files = proof.get("files")
    if not isinstance(files, list) or not files:
        raise ValueError("coverage proof has no reports")
    names = set()
    for entry in files:
        name = entry.get("path")
        if not isinstance(name, str) or Path(name).name != name or name in names:
            raise ValueError("coverage proof contains an unsafe or duplicate report")
        names.add(name)
        report = source / name
        if (
            report.is_symlink()
            or not report.is_file()
            or report.stat().st_size != entry.get("bytes")
            or digest(report) != entry.get("sha256")
        ):
            raise ValueError("coverage report bytes differ")
    required = {"latest.json", "coverage-rust.xml", "coverage-python.xml"} | {
        f"tests-{suite}.xml" for suite in SUITES
    }
    if not required <= names:
        raise ValueError("coverage proof omits required reports")
    result = json.loads((source / "latest.json").read_text(encoding="utf-8"))
    if set(result.get("tests", {})) != set(SUITES) or any(
        entry.get("outcome") != "passed"
        or entry.get("coverage") is not True
        or entry.get("revision") != build["build_revision"]
        or entry.get("platform") != build["platform"]
        or entry.get("requested_configuration") is not None
        for entry in result["tests"].values()
    ):
        raise ValueError("required coverage did not pass for the exact native build")
    for language in ("rust", "python"):
        evidence.validate_coverage(
            (source / f"coverage-{language}.xml").read_text(encoding="utf-8")
        )
    return proof


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    collect(Path.cwd())
