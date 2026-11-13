# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prepare Shared Inputs And Collect Results For The Native Mise Test Graph."""

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
from collections.abc import Sequence
from pathlib import Path

# Third Party Imports
# Local Imports
from ..build import paths
from ..prepare import generated
from ..process import spawn_flags
from . import archive, policy, report

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def command(category: str, language: str, profile: str, workers: int, archive_directory: Path | None = None) -> list[str]:
    """Use The Same Native Lane Runner Without Concurrent Input Reinstallation."""
    arguments = [
        sys.executable,
        "-m",
        "dokime_tools.test.run",
        "test",
        "--prepared",
        "--profile",
        profile,
        "--category",
        category,
        "--language",
        language,
    ]
    extra = []
    if language == "rust":
        if archive_directory is not None:
            arguments += ["--archive-directory", str(archive_directory)]
        extra = ["--test-threads", str(workers)]
    elif language == "python":
        arguments += ["--project", "sdk/python"]
        if category == "integration":
            extra = ["-n", str(workers), "--dist=worksteal", "--max-worker-restart=0"]
        elif category == "acceptance":
            extra = ["--build-profile", profile]
    elif language == "typescript":
        arguments += ["--project", "crates/dokime-gui"]
        if category == "system":
            extra = ["node", "e2e/system.mjs", "--profile", profile, "--junit", "{junit}"]
        elif category in ("unit", "integration"):
            extra = [
                "node",
                "./node_modules/vitest/vitest.mjs",
                "run",
                f"tests/{category}",
                *(
                    [
                        "--exclude",
                        "tests/unit/{settings/preferences,console/nominal,cycle/{nominal,projection,document-stream},history/totals,library/nominal,shell/navigation,plots/{columnar-frame,live-frames,plot-pipeline}}.test.ts",
                        "--testNamePattern",
                        "^(?!.*Native Rejections and Captured Diagnostics Reach Notification History)",
                    ]
                    if category == "unit"
                    else []
                ),
                "--reporter=default",
                "--reporter=junit",
                "--outputFile.junit={junit}",
                "--testTimeout={timeout_ms}",
                f"--maxWorkers={workers}",
            ]
        else:
            raise policy.TestError(f"no native command for {category}-{language}")
    else:
        raise policy.TestError(f"no native command for {category}-{language}")
    return arguments + (["--", *extra] if extra else [])


def prepare(directory: Path, profile: str) -> None:
    """Prepare One Shared Read-Only Input Set Before Mise Starts Any Leaf."""
    started = time.perf_counter()
    lanes = policy.lanes(directory)
    if not lanes:
        raise policy.TestError("no declared test lanes")
    worker_pools = sum(
        language == "rust" or language == "typescript" and category != "system" or language == "python" and category == "integration"
        for category, language in lanes
    )
    desktop = int(("system", "typescript") in lanes)
    workers = max(1, ((os.process_cpu_count() or 1) - desktop) // max(1, worker_pools))
    destination = report.native_directory(directory, Path("target/test-reports") / f"run-{profile}-{paths.target()}")
    for name in ("time-summary.md", "time-summary.json", "prepared.json"):
        (destination / name).unlink(missing_ok=True)
    for category, language in lanes:
        (destination / f"{category}-{language}.json").unlink(missing_ok=True)
    (destination / "integration-python-tooling.json").unlink(missing_ok=True)

    archive.install_wheels(directory, profile)
    generated.restore(profile)
    extracted = Path(tempfile.mkdtemp(prefix="inputs-", dir=destination))
    if any(language == "rust" for _, language in lanes):
        archive.extract(directory, profile, extracted)
    report.dump(
        destination / "prepared.json",
        {
            "profile": profile,
            "worker_pools": worker_pools,
            "workers_per_pool": workers,
            "archive_directory": str(extracted),
            "preparation_seconds": time.perf_counter() - started,
            "execution_started": time.time(),
        },
    )


def lane(directory: Path, profile: str, category: str, language: str, project: str) -> None:
    """Record The Real Exit Status; The Final Aggregate Fails After All Leaves Finish."""
    destination = directory / "target/test-reports" / f"run-{profile}-{paths.target()}"
    context = json.loads((destination / "prepared.json").read_text(encoding="utf-8"))
    extracted = Path(context["archive_directory"]) if language == "rust" else None
    key = f"{category}-{language}"
    if project == "tools/python":
        key += "-tooling"
        arguments = [sys.executable, "-m", "unittest", "discover", "-s", "tools/python/tests/integration"]
    else:
        arguments = command(category, language, profile, context["workers_per_pool"], extracted)
    environment = dict(os.environ, PYTHONUNBUFFERED="1")
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    row = _lane(directory, key, arguments, destination, environment)
    report.dump(destination / f"{key}.json", row)
    print(f"{'PASS' if row['exit_code'] == 0 else 'FAIL'}  {key}: {row['seconds']:.3f} s (exit {row['exit_code']})", flush=True)
    if row["exit_code"]:
        print((directory / row["log"]).read_text(encoding="utf-8", errors="replace"), flush=True)


def finish(directory: Path, profile: str) -> int:
    """Missing Results And Failed Leaves Both Fail The Aggregate."""
    destination = directory / "target/test-reports" / f"run-{profile}-{paths.target()}"
    context = json.loads((destination / "prepared.json").read_text(encoding="utf-8"))
    keys = [f"{category}-{language}" for category, language in policy.lanes(directory)]
    keys.append("integration-python-tooling")
    rows = []
    for key in keys:
        result = destination / f"{key}.json"
        if not result.is_file():
            rows.append({"lane": key, "exit_code": 1, "seconds": 0, "log": ""})
            print(f"FAIL  {key}: missing lane result", file=sys.stderr)
            continue
        rows.append(json.loads(result.read_text(encoding="utf-8")))
    context["lanes"] = sorted(rows, key=lambda row: row["lane"])
    context["execution_seconds"] = time.time() - context.pop("execution_started")
    context["total_seconds"] = context["preparation_seconds"] + context["execution_seconds"]
    extracted = Path(context.pop("archive_directory"))
    if extracted.resolve().parent != destination.resolve() or not extracted.name.startswith("inputs-"):
        raise policy.TestError("prepared archive escapes its owned report directory")
    shutil.rmtree(extracted)
    report.dump(destination / "time-summary.json", context)
    lines = ["# Concurrent Test Execution", "", "| Lane | Exit | Seconds | Log |", "| --- | ---: | ---: | --- |"]
    for row in context["lanes"]:
        lines.append(f"| {row['lane']} | {row['exit_code']} | {row['seconds']:.3f} | [{row['lane']}.log]({row['lane']}.log) |")
    report.write(destination / "time-summary.md", "\n".join(lines) + "\n")
    print(f"Logs And Summary: {destination.relative_to(directory).as_posix()}", flush=True)
    return int(any(row["exit_code"] != 0 for row in rows))


def main(arguments: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=policy.PROFILES, default="dev")
    actions = parser.add_mutually_exclusive_group(required=True)
    actions.add_argument("--prepare", action="store_true")
    actions.add_argument("--lane", action="store_true")
    actions.add_argument("--finish", action="store_true")
    parser.add_argument("--category", choices=policy.CATEGORIES)
    parser.add_argument("--language", choices=policy.LANGUAGES)
    parser.add_argument("--project", default="sdk/python")
    options = parser.parse_args(arguments)
    try:
        directory = policy.root()
        if options.prepare:
            prepare(directory, options.profile)
        elif options.finish:
            return finish(directory, options.profile)
        else:
            if not options.category or not options.language:
                parser.error("--lane requires --category and --language")
            lane(directory, options.profile, options.category, options.language, options.project)
        return 0
    except (policy.TestError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"test: {error}", file=sys.stderr)
        return 1


# === Private Functions === #


def _lane(directory: Path, key: str, arguments: Sequence[str], destination: Path, environment: dict[str, str]) -> dict:
    started = time.perf_counter()
    log = destination / f"{key}.log"
    with log.open("w", encoding="utf-8") as output:
        try:
            status = subprocess.run(
                arguments, cwd=directory, env=environment, stdout=output, stderr=subprocess.STDOUT, creationflags=spawn_flags(), check=False
            ).returncode
        except OSError as error:
            output.write(str(error) + "\n")
            status = 1
    return {
        "lane": key,
        "exit_code": status,
        "seconds": round(time.perf_counter() - started, 3),
        "log": log.relative_to(directory).as_posix(),
    }


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
