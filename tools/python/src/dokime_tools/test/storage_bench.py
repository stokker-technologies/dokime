# ***
#
# Dokime
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
from typing import TypedDict, cast

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.test.credentials
import dokime_tools.test.desktop
import dokime_tools.test.desktop_bench
import dokime_tools.test.scratch

from ..build import paths
from ..process import spawn_flags
from . import archive

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CompilerTarget(TypedDict):
    name: str
    kind: list[str]


class CompilerEvent(TypedDict):
    target: CompilerTarget
    executable: str | None


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
    events = [cast(CompilerEvent, json.loads(line)) for line in metadata.read_text(encoding="utf-8").splitlines()]
    executables = []
    for executables_event in events:
        if not isinstance((executable := executables_event.get("executable")), str):
            continue
        if executables_event["target"]["name"] != "storage":
            continue
        if "bench" not in executables_event["target"]["kind"]:
            continue
        executables.append(executable)

    if len(executables) != 1:
        raise ValueError("Build the storage benchmark once with mise run build -- --artifact bench --binary storage")
    archive.fixtures(directory, "bench", executables)
    environment = dict(os.environ, CRITERION_HOME=str(directory / "target/benchmarks/storage"))
    if options.once:
        environment["DOKIME_BENCH_ONCE"] = "1"
    environment.pop("DOKIME_TRACE_DIR", None)
    report = directory / "target/benchmarks/storage/run.log"
    report.parent.mkdir(parents=True, exist_ok=True)
    scratch_root = dokime_tools.test.scratch.Root(environment, report.parent)
    try:
        scratch_environment = scratch_root.environment
        credential_store = dokime_tools.test.credentials.Store(scratch_environment)
        try:
            child_environment = credential_store.environment
            written = report.open("w", encoding="utf-8")
            try:
                process = subprocess.Popen(
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
                )
                try:
                    output_stream = process.stdout
                    if output_stream is None:
                        raise RuntimeError("Run Storage Bench Tests: Measure Storage Benchmarks: Open the Output Stream")
                    try:
                        for line in output_stream:
                            print(line, end="", flush=True)
                            written.write(line)
                            written.flush()
                    finally:
                        output_stream.close()
                finally:
                    status = process.wait()
            finally:
                written.close()
            if status:
                raise RuntimeError(f"Run Storage Bench Tests: Measure Storage Benchmarks: Executable Exited {status}; See {report}")
        finally:
            credential_store.close()
    finally:
        scratch_root.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
