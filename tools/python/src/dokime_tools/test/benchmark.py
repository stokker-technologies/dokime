# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Test Benchmark Operations."""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import math
import os
import platform
import re
import shutil
import sys
import time
from collections.abc import Sequence
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.policy
import dokime_tools.test.desktop
import dokime_tools.test.scratch

from ..build import paths
from ..release import reports
from . import archive, desktop_bench, execution, lanes, policy, protocol_bench, pytest, report

# Type Checking Imports

# === Public Exports === #

# === Constants === #

CRITERION = report.BENCHMARKS / "criterion"
BASELINE = "dev"

# === Types === #

# === Classes === #

# === Public Functions === #


def run(
    directory: Path,
    category: str,
    language: str,
    project: str,
    refresh: bool,
    extra: Sequence[str],
    archive_arguments: Sequence[str] = (),
    binary: str | None = None,
) -> int:
    """Measure one benchmark layer and compare it with the `dev` baseline."""
    if binary == "system-sdk":
        from . import sdk_bench

        return sdk_bench.run(directory, refresh, extra)
    rules = policy.load(directory)
    output = directory / report.BENCHMARKS / f"benchmarks-{language}-{category}.json"
    if output.is_symlink() or not output.resolve().is_relative_to(directory.resolve()):
        raise policy.TestError("Run Benchmark Tests: keep the native report inside the repository and unlinked")
    output.unlink(missing_ok=True)
    measure = category in ("integration", "system")
    if language == "rust" and category == "system" and binary in (None, "system-desktop"):
        previous = directory / report.BENCHMARKS / "benchmarks-rust-desktop.json"
        if previous.is_symlink() or not previous.resolve().is_relative_to(directory.resolve()):
            raise policy.TestError("Run Benchmark Tests: desktop benchmark report escapes the repository")
        previous.unlink(missing_ok=True)
    lanes.preflight(directory, category, language, "bench")
    identity = {
        "platform": [
            sys.platform,
            platform.machine(),
            platform.platform(),
            platform.processor(),
        ],
        "runner": os.environ.get("RUNNER_NAME", os.environ.get("CI_RUNNER_ID", platform.node())),
        "profile": "bench",
        "metric": "median-nanoseconds",
        "arguments": list(extra),
        "cargo_arguments": list(archive_arguments),
    }
    boundaries = {}
    if language == "rust":
        identity["rustc"] = archive._toolchain(directory)
        rows, status = rust(directory, category, refresh, measure, extra, archive_arguments, identity, binary, boundaries=boundaries)
    elif language == "python":
        identity["python"] = {
            "version": sys.version,
            "executable": archive._digest(Path(sys.executable)),
        }
        rows, status = python(directory, category, project, refresh, measure, extra, identity)
    else:
        raise policy.TestError("Run Benchmark Tests: benchmarks run for rust and python")
    if not rows and "--test" not in extra:
        raise policy.TestError(f"Run Benchmark Tests: no {category} {language} benchmark produced results")
    if "--test" in extra:
        print("Benchmark Diagnostic Mode: No Criterion Regression Estimates")

    revision = os.environ.get("GITHUB_SHA") or dokime_tools.policy.execute(directory, ["git", "rev-parse", "HEAD"], capture=True).strip()
    summary = reports.measurements(rows, f"{language}/{category}", revision, rules.bench_regression_percent, boundaries=boundaries)
    dokime_tools.policy.write_json(output, summary)

    for name, median, baseline in rows:
        if baseline is None:
            if refresh and status == 0:
                print(f"::notice::benchmark baseline seeded: {name}")
            else:
                print(
                    f"benchmark baseline missing: {name}; run mise run bench -- --language {language} --layer {category} --refresh-baseline"
                )
                status = status or 1
            continue
        change = (median - baseline) / baseline * 100
        if math.isfinite(change) and change > rules.bench_regression_percent:
            print(f"benchmark regression: {name}: {change:.1f}%")
    print(f"bench {category}-{language}: {len(rows)} native results; see {report.BENCHMARKS}")
    return 0 if status == 0 else 1


def rust(
    directory: Path,
    category: str,
    refresh: bool,
    measure: bool,
    extra: Sequence[str],
    archive_arguments: Sequence[str],
    identity: dict,
    binary: str | None = None,
    *,
    boundaries: dict[str, str] | None = None,
) -> tuple[list[tuple[str, float, float | None]], int]:
    messages = paths.output(directory, "build", "bench") / "bench/compiler.json"
    if not messages.is_file():
        raise policy.TestError("Run Benchmark Tests: missing benchmark compiler metadata; run build:rust:bench first")
    archive.fixtures(directory, "bench", [messages.relative_to(directory).as_posix()])
    rows = []
    status = 0
    executed = False
    for line in messages.read_text(encoding="utf-8").splitlines():
        message = json.loads(line) if line.startswith("{") else {}
        target = message.get("target", {})
        name = target.get("name", "")
        if name == "system-sdk":
            continue
        if binary is not None and name != binary:
            continue
        artifact = message.get("reason") == "compiler-artifact" and "bench" in target.get("kind", []) and message.get("executable")
        if not artifact:
            continue
        if not (name == category or name.startswith(category + "-") or name == "protocol" and category == "system"):
            continue
        executed = True
        manifest = Path(message["manifest_path"])
        owner = manifest.parent / name
        if manifest.is_absolute() or ".." in owner.parts:
            raise policy.TestError("Run Benchmark Tests: benchmark owner escapes the repository")
        compatibility = baseline_identity(identity | {"compiler_profile": message.get("profile", {})})
        home = report.native_directory(directory, CRITERION / compatibility / owner)
        if any(path.is_symlink() or path.is_junction() for path in home.rglob("*")):
            raise policy.TestError("Run Benchmark Tests: native benchmark files must not be linked")
        baselines = read_baselines(home)
        if name == "protocol":
            (home / "encoded-sizes.json").unlink(missing_ok=True)
            (directory / report.BENCHMARKS / "benchmarks-rust-protocol-sizes.json").unlink(missing_ok=True)
        since = time.time()
        environment = dict(os.environ, CRITERION_HOME=str(home.resolve()))
        environment.pop("TEST_REPORT_DIR", None)
        environment.pop("DOKIME_TEST_TRACE_DIR", None)
        if name == "system-desktop":
            (home / "desktop").mkdir(exist_ok=True)
            for diagnostic in ("observations.json", "completion.json", "host-shutdown.json"):
                (home / "desktop" / diagnostic).unlink(missing_ok=True)
            shutil.copyfile(paths.output(directory, "build", "bench") / "fixtures/stamp.json", home / "desktop/fixture-stamp.json")
        scratch_root = dokime_tools.test.scratch.Root(environment, home / ("desktop" if name == "system-desktop" else "scratch"))
        try:
            child_environment = scratch_root.environment
            desktop_environment = dokime_tools.test.desktop.Environment(directory, child_environment) if name == "system-desktop" else None
            try:
                code, peak = execution.measure(
                    [str(directory / message["executable"]), "--bench", "--baseline-lenient", BASELINE, *extra],
                    directory / manifest.parent,
                    child_environment,
                    output=home / "run.log",
                )
            finally:
                if desktop_environment is not None:
                    desktop_environment.close()
        finally:
            scratch_root.close()
        status = status or code
        if boundaries is not None and (home / "run.log").is_file():
            boundary = None
            for line in (home / "run.log").read_text(encoding="utf-8").splitlines():
                if line.startswith("Boundary | "):
                    boundary = line.split(" | ", 2)[2]
                matched = re.match(r"Benchmarking ([^:]+)(?::|$)", line)
                if matched and boundary is not None:
                    boundaries[f"{owner.as_posix()}/{matched[1]}"] = boundary
        if any(path.is_symlink() or path.is_junction() for path in home.rglob("*")):
            raise policy.TestError("Run Benchmark Tests: native benchmark files must not be linked")
        if name == "system-desktop":
            desktop_bench.collect(
                directory, home / "desktop", identity, policy.load(directory).bench_regression_percent, refresh=refresh, success=code == 0
            )
        if name == "protocol":
            protocol_bench.collect(
                directory, home, identity, policy.load(directory).bench_regression_percent, refresh=refresh, success=code == 0
            )
        for path in sorted(home.rglob("new/estimates.json")) if home.is_dir() else []:
            if path.stat().st_mtime < since:
                continue
            benchmark = json.loads(path.with_name("benchmark.json").read_text(encoding="utf-8"))
            estimates = json.loads(path.read_text(encoding="utf-8"))
            key = path.parent.parent
            case = f"{owner.as_posix()}/{benchmark.get('full_id', key.name)}"
            rows.append((case, estimates["median"]["point_estimate"], baselines.get(key)))
            if code == 0 and refresh:
                shutil.copytree(path.parent, key / BASELINE, dirs_exist_ok=True)
        if measure and peak is not None:
            print(f"benchmark peak memory: {owner.as_posix()}: {peak} bytes")
    if not executed:
        raise policy.TestError("Run Benchmark Tests: no native benchmark executable matched the selection")
    return rows, status


def baseline_identity(identity: dict) -> str:
    """Partition by Platform, Toolchain and Scenario, Independently of Runner Replacement."""
    compatible = {key: value for key, value in identity.items() if key not in {"runner", "platform"}}
    compatible["platform"] = identity.get("platform", [])[:2]
    return hashlib.sha256(json.dumps(compatible, sort_keys=True).encode()).hexdigest()


def read_baselines(home: Path) -> dict[Path, float]:
    medians = {}
    for path in home.rglob(f"{BASELINE}/estimates.json") if home.is_dir() else []:
        try:
            median = json.loads(path.read_text(encoding="utf-8"))["median"]["point_estimate"]
        except (OSError, ValueError, KeyError, TypeError):
            continue
        if not isinstance(median, bool) and isinstance(median, (int, float)) and math.isfinite(median) and median > 0:
            medians[path.parent.parent] = median
    return medians


def python(
    directory: Path,
    category: str,
    project: str,
    refresh: bool,
    measure: bool,
    extra: Sequence[str],
    identity: dict,
) -> tuple[list[tuple[str, float, float | None]], int]:
    owner = Path(project)
    if owner.is_absolute() or ".." in owner.parts:
        raise policy.TestError("Run Benchmark Tests: benchmark project escapes the repository")
    compatibility = baseline_identity(identity)
    home = report.native_directory(directory, report.BENCHMARKS / "python" / compatibility / owner / category)
    output = home / "new.json"
    stored = home / f"{BASELINE}.json"
    if output.is_symlink() or stored.is_symlink():
        raise policy.TestError("Run Benchmark Tests: native benchmark files must not be linked")
    try:
        previous = json.loads(stored.read_text(encoding="utf-8")) if stored.is_file() else {}
        baselines = {entry.get("fullname", entry["name"]): entry["stats"]["median"] * 1e9 for entry in previous.get("benchmarks", [])}
    except (OSError, ValueError, KeyError, TypeError):
        baselines = {}
    baselines_items = {}
    for baselines_name, baselines_value in baselines.items():
        if isinstance(baselines_value, bool):
            continue
        if not isinstance(baselines_value, (int, float)):
            continue
        if not math.isfinite(baselines_value):
            continue
        if not baselines_value > 0:
            continue
        baselines_items[baselines_name] = baselines_value
    baselines = baselines_items
    output.unlink(missing_ok=True)
    cwd = directory / project
    environment = pytest.environment(directory)
    environment.pop("TEST_REPORT_DIR", None)
    environment.pop("DOKIME_TEST_TRACE_DIR", None)
    status, peak = execution.measure(pytest.bench(cwd, category, output, extra), cwd, environment)
    if not output.is_file():
        raise policy.TestError("Run Benchmark Tests: pytest-benchmark wrote no results")
    rows = []
    for entry in json.loads(output.read_text(encoding="utf-8"))["benchmarks"]:
        name = entry.get("fullname", entry["name"])
        rows.append((f"{owner.as_posix()}/{name}", entry["stats"]["median"] * 1e9, baselines.get(name)))
    if status == 0 and refresh:
        shutil.copyfile(output, stored)
    if measure and peak is not None:
        print(f"benchmark peak memory: {project}/{category}: {peak} bytes")
    return rows, status


# === Private Functions === #

# === Entry Point === #
