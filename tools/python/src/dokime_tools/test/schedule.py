# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Schedule Independent Test Lanes After Preparing Shared Read-Only Inputs."""

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
import sys
import time
from collections.abc import Mapping, Sequence
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

# Third Party Imports
# Local Imports
from .. import temporary
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
        if category == "integration":
            extra += ["-E", "not binary_id(dokime-macros::compile)"]
    elif language == "python":
        arguments += ["--project", "packages/dokime-client"]
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


def execute(directory: Path, commands: Mapping[str, Sequence[str]], destination: Path) -> list[dict]:
    """Keep Each Lane's Full Output And Finish Every Lane Despite Failures."""
    environment = dict(os.environ)
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    # Children Use The Prepared Interpreter Without uv Sync Rewriting Shared Inputs.
    environment["PYTHONUNBUFFERED"] = "1"
    results = []
    with ThreadPoolExecutor(max_workers=len(commands)) as executor:
        pending = {}
        for key, arguments in commands.items():
            print(f"START {key}", flush=True)
            future = executor.submit(_lane, directory, key, arguments, destination, environment)
            pending[future] = key
        for future in as_completed(pending):
            row = future.result()
            results.append(row)
            print(
                f"{'PASS' if row['exit_code'] == 0 else 'FAIL'}  {row['lane']}: {row['seconds']:.3f} s (exit {row['exit_code']})",
                flush=True,
            )
            if row["exit_code"] != 0:
                log = directory / row["log"]
                print("\n".join(log.read_text(encoding="utf-8", errors="replace").splitlines()[-30:]), flush=True)
    return sorted(results, key=lambda row: row["lane"])


def main(arguments: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=policy.PROFILES, default="dev")
    options = parser.parse_args(arguments)
    started = time.perf_counter()
    try:
        directory = policy.root()
        lanes = policy.lanes(directory)
        if not lanes:
            raise policy.TestError("no declared test lanes")
        # Budget Every Worker Pool And Leave Headroom For The Interactive Desktop.
        rust_lanes = sum(language == "rust" for _, language in lanes)
        worker_pools = sum(
            language == "rust" or language == "typescript" and category != "system" or language == "python" and category == "integration"
            for category, language in lanes
        )
        desktop = int(("system", "typescript") in lanes)
        workers = max(1, ((os.process_cpu_count() or 1) - desktop) // max(1, worker_pools))
        destination = report.native_directory(directory, Path("target/test-reports") / f"run-{options.profile}-{paths.target()}")
        for name in ("time-summary.md", "time-summary.json"):
            (destination / name).unlink(missing_ok=True)

        print("Preparing Shared Test Inputs", flush=True)
        archive.install_wheels(directory, options.profile)
        generated.restore(options.profile)
        with temporary.directory() as extracted:
            if rust_lanes:
                archive.extract(directory, options.profile, extracted)
            commands = {
                f"{category}-{language}": command(category, language, options.profile, workers, extracted if language == "rust" else None)
                for category, language in lanes
            }
            preparation = time.perf_counter() - started
            print(
                f"Running {len(lanes)} Lanes Concurrently ({workers} Workers Per Rust Lane / Python Integration / Vitest Lane)", flush=True
            )
            execution = time.perf_counter()
            rows = execute(directory, commands, destination)
            seconds = time.perf_counter() - execution
        total = time.perf_counter() - started
        result = {
            "profile": options.profile,
            "worker_pools": worker_pools,
            "workers_per_pool": workers,
            "preparation_seconds": preparation,
            "execution_seconds": seconds,
            "total_seconds": total,
            "lanes": rows,
        }
        report.dump(destination / "time-summary.json", result)
        lines = [
            "# Concurrent Test Execution",
            "",
            f"Preparation: {preparation:.3f} s; execution: {seconds:.3f} s; total: {total:.3f} s.",
            "",
            "| Lane | Exit | Seconds | Log | Time Summary |",
            "| --- | ---: | ---: | --- | --- |",
        ]
        for row in rows:
            lines.append(
                f"| {row['lane']} | {row['exit_code']} | {row['seconds']:.3f} | [{row['lane']}.log]({row['lane']}.log) | [Summary](../{row['lane']}/time-summary.md) |"
            )
        report.write(destination / "time-summary.md", "\n".join(lines) + "\n")
        print(f"Execution: {seconds:.3f} s; With Shared Preparation: {total:.3f} s", flush=True)
        print(f"Logs And Summary: {destination.relative_to(directory).as_posix()}", flush=True)
        return int(any(row["exit_code"] != 0 for row in rows))
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
