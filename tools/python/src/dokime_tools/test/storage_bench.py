# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Run Prepared Storage Benchmarks"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import subprocess
from pathlib import Path

# Third Party Imports
# Local Imports
from ..build import paths
from ..process import spawn_flags
from . import archive
from .desktop_bench import credentials

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    parser = argparse.ArgumentParser(description="Measure the Prepared Arrow Storage Workloads")
    parser.add_argument("--select")
    parser.add_argument("--once", action="store_true", help="Measure Each Selected Case Once Without Statistical Estimates")
    options = parser.parse_args()
    directory = Path.cwd()
    output = paths.output(directory, "build", "bench")
    metadata = output / "bench/compiler.json"
    archive.fixtures(directory, "bench", [metadata.relative_to(directory).as_posix()])
    events = [json.loads(line) for line in metadata.read_text(encoding="utf-8").splitlines()]
    executables = [
        event["executable"]
        for event in events
        if event.get("executable")
        and event.get("target", {}).get("name") == "storage"
        and "bench" in event.get("target", {}).get("kind", [])
    ]
    if len(executables) != 1:
        raise ValueError("Build the storage benchmark once with mise run build -- --artifact bench --binary storage")
    archive.fixtures(directory, "bench", executables)
    environment = dict(os.environ, CRITERION_HOME=str(directory / "target/benchmarks/storage"))
    if options.once:
        environment["DOKIME_BENCH_ONCE"] = "1"
    environment.pop("DOKIME_TRACE_DIR", None)
    report = directory / "target/benchmarks/storage/run.log"
    report.parent.mkdir(parents=True, exist_ok=True)
    with (
        credentials(environment) as child_environment,
        report.open("w", encoding="utf-8") as written,
        subprocess.Popen(
            [
                str(directory / executables[0]),
                "--bench",
                *(["--test"] if options.once else []),
                *([options.select] if options.select else []),
            ],
            cwd=directory,
            env=child_environment,
            creationflags=spawn_flags(),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
        ) as process,
    ):
        for line in process.stdout:
            print(line, end="", flush=True)
            written.write(line)
            written.flush()
        status = process.wait()
        if status:
            raise RuntimeError(f"Measure Storage Benchmarks: Executable Exited {status}; See {report}")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
