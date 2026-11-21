# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Collect SDK Benchmark Measurements"""

# === Imports === #

# Standard Library Imports
import importlib.util
import json
import math
import os
import platform
import re
import shutil
import sys
from collections.abc import Sequence
from pathlib import Path

# isort: split

# Third Party Imports
# isort: split

# Local Imports
import dokime_tools.test.policy

from .. import policy
from ..build import paths
from ..release import reports
from . import archive, benchmark, credentials, execution, pytest, report, scratch

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def collect(log: str, identity: dict, baseline: dict, revision: str, threshold: float) -> dict:
    previous = {(row["name"], row["unit"]): row["value"] for row in baseline.get("benchmarks", []) if row.get("status") != "not_measured"}
    rows = []
    for line in log.splitlines():
        if not line.startswith("SDK\t"):
            continue
        _, name, unit, value, label, reason = line.split("\t", 5)
        row = {
            "name": name,
            **reports.identity(
                "rust/sdk",
                paths.target(),
                name,
                "Host admission and operation routing; retained host receipt timestamps in the steady window; plugin resident memory",
            ),
            "label": label,
            "unit": unit,
            "revision": revision,
            "baseline_value": previous.get((name, unit)),
            "threshold_percent": threshold,
            "higher_is_better": unit in {"calls/s", "samples/s"},
        }
        if value == "not_measured":
            if not reason:
                raise ValueError(f"Collect SDK Bench: Explain the Unavailable Measure {name}")
            row.update(value=None, status="not_measured", reason=reason, baseline_value=None)
        else:
            measured = float(value)
            if not math.isfinite(measured) or measured < 0:
                raise ValueError(f"Collect SDK Bench: Supply a Finite Nonnegative Measure for {name}")
            row["value"] = measured
        if any(existing["name"] == name for existing in rows):
            raise ValueError(f"Collect SDK Bench: Record {name} Only Once")
        rows.append(row)
    return {"benchmarks": rows, "identity": identity}


def run(directory: Path, refresh: bool, extra: Sequence[str]) -> int:
    published = report.native_directory(directory, report.BENCHMARKS)
    for previous in sorted(published.glob("benchmarks-sdk-*.json")):
        if previous.is_symlink():
            raise ValueError("Run SDK Bench: Keep Native Reports Unlinked")
        previous.unlink()
    messages = paths.output(directory, "build", "bench") / "bench/compiler.json"
    if not messages.is_file():
        raise ValueError("Run SDK Bench: Build system-sdk Through mise run build -- --artifact bench --binary system-sdk")
    archive.fixtures(directory, "bench", [messages.relative_to(directory).as_posix()])
    artifacts = [json.loads(line) for line in messages.read_text(encoding="utf-8").splitlines() if line.startswith("{")]
    artifact = next(
        (
            item
            for item in artifacts
            if item.get("reason") == "compiler-artifact" and item.get("target", {}).get("name") == "system-sdk" and item.get("executable")
        ),
        None,
    )
    if artifact is None:
        raise ValueError("Run SDK Bench: Build the system-sdk Target")
    executable = (directory / artifact["executable"]).resolve()
    if not executable.is_relative_to(directory.resolve()):
        raise ValueError("Run SDK Bench: Keep the Executable Inside the Worktree")
    selected = [value for value in extra if not value.startswith("--")]
    suffix = ".exe" if os.name == "nt" else ""
    reference = paths.output(directory, "build", "bench") / "fixtures/host" / f"dokime-rust-thermal{suffix}"
    fixture_profile = os.environ.get("TEST_PROFILE", os.environ.get("CI_PROFILE", "dev"))
    external = paths.output(directory, "build", fixture_profile) / "fixtures/host"
    commands = {
        "rust": reference,
        "python": Path(sys.executable),
        "csharp": external / "csharp" / f"dokime-system-story{suffix}",
        "cpp": external / f"dokime-system-suite{suffix}",
    }
    available = [
        language
        for language, command in commands.items()
        if command.is_file() and (language != "python" or importlib.util.find_spec("thermal_example") is not None)
    ]
    languages = (
        [os.environ["DOKIME_BENCH_LANGUAGE"]] if "DOKIME_BENCH_LANGUAGE" in os.environ else list(commands) if selected else available
    )
    languages = [language for language in languages if not selected or any(re.search(pattern, f"sdk/{language}/") for pattern in selected)]
    if not languages:
        raise ValueError("Run SDK Bench: Select at Least One SDK Language")
    status = 0
    revision = os.environ.get("GITHUB_SHA") or policy.execute(directory, ["git", "rev-parse", "HEAD"], capture=True).strip()
    for language in languages:
        if language not in {"rust", "python", "csharp", "cpp"}:
            raise ValueError("Run SDK Bench: Select rust, python, csharp or cpp")
        if not commands[language].is_file():
            raise ValueError(
                f"Run SDK Bench: Build the {language} Host Fixture Through mise run build:host-fixtures -- --language {language}"
            )
        archive.fixtures(directory, "bench", [reference.relative_to(directory).as_posix()])
        identity = {
            "platform": [
                sys.platform,
                platform.machine(),
            ],
            "rustc": archive._toolchain(directory),
            "compiler_profile": artifact.get("profile", {}),
            "python": sys.version if language == "python" else None,
            "language": language,
            "scenario": "thermal-interval-v1",
        }
        home = report.native_directory(directory, report.BENCHMARKS / "sdk" / benchmark.baseline_identity(identity))
        stored = home / "dev.json"
        if stored.is_symlink():
            raise ValueError("Run SDK Bench: Keep Native Baselines Unlinked")
        previous = json.loads(stored.read_text(encoding="utf-8")) if stored.is_file() else {}
        root = scratch.Root(pytest.environment(directory), home / "scratch")
        code = 1
        try:
            environment = dict(
                root.environment,
                DOKIME_BENCH_LANGUAGE=language,
                DOKIME_BENCH_ROOT=str(root.path / "host"),
                DOKIME_BENCH_REFERENCE=str(reference),
                DOKIME_BENCH_EXECUTABLE=str(commands[language]),
            )
            store = credentials.Store(environment)
            try:
                code, _ = execution.measure([str(executable)], directory, store.environment, output=home / "run.log")
            finally:
                store.close()
        finally:
            if code != 0:
                root.environment["DOKIME_KEEP_EVIDENCE"] = "1"
            root.close()
        status = status or code
        summary = collect(
            (home / "run.log").read_text(encoding="utf-8"),
            identity,
            previous,
            revision,
            dokime_tools.test.policy.load(directory).bench_regression_percent,
        )
        if code == 0 and not summary["benchmarks"]:
            raise ValueError(f"Run SDK Bench: Record the Scenario Measures for {language}")
        policy.write_json(home / "new.json", summary)
        policy.write_json(directory / report.BENCHMARKS / f"benchmarks-sdk-{language}.json", summary)
        if code == 0 and refresh:
            shutil.copyfile(home / "new.json", stored)
    return status


# === Private Functions === #

# === Entry Point === #
