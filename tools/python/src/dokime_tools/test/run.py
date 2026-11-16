# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Parse Test, Coverage and Benchmark Arguments."

# === Imports === #

# Standard Library Imports
import argparse
import sys
from collections.abc import Sequence
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from . import benchmark, coverage, lanes, policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main(arguments: Sequence[str] | None = None) -> int:
    values = list(sys.argv[1:] if arguments is None else arguments)
    extra: list[str] = []
    if "--" in values:
        split = values.index("--")
        values, extra = values[:split], values[split + 1 :]

    parser = argparse.ArgumentParser(description="Test Lanes and Evidence")
    commands = parser.add_subparsers(dest="command", required=True)
    lane = commands.add_parser("test", help="run one test lane")
    lane.add_argument("--category", choices=policy.CATEGORIES, required=True)
    lane.add_argument("--language", choices=policy.LANGUAGES, required=True)
    lane.add_argument("--coverage", action="store_true")
    lane.add_argument(
        "--prepared", action="store_true", help="Shared wheel installation already completed before concurrent readers started"
    )
    lane.add_argument("--archive-directory", type=Path, help="Read-Only Nextest Extraction Owned by the Concurrent Entry")
    bench_lane = commands.add_parser("bench", help="run one benchmark layer")
    bench_lane.add_argument("--category", choices=policy.BENCH_CATEGORIES, required=True)
    bench_lane.add_argument("--language", choices=("rust", "python"), required=True)
    bench_lane.add_argument("--refresh-baseline", action="store_true")
    bench_lane.add_argument("--archive-argument", action="append", default=[])
    bench_lane.add_argument("--binary", help="Measure One Native Benchmark Executable")
    measured = commands.add_parser("coverage", help="instrumented run of every layer")
    measured.add_argument("--language", choices=("rust", "python"), required=True)
    for command in (lane, bench_lane, measured):
        command.add_argument("--project", default=".")
    for command in (lane, measured):
        command.add_argument("--profile", choices=policy.PROFILES, default="dev")
    options = parser.parse_args(values)

    try:
        directory = policy.root()
        if options.command == "test" and options.coverage:
            return coverage.run(
                directory,
                options.language,
                options.category,
                options.project,
                options.profile,
                extra,
            )
        if options.command == "test":
            return lanes.run(
                directory,
                options.category,
                options.language,
                options.project,
                options.profile,
                extra,
                prepared=options.prepared,
                archive_directory=options.archive_directory,
            )
        if options.command == "coverage":
            return coverage.run(
                directory,
                options.language,
                None,
                options.project,
                options.profile,
                extra,
            )
        if options.command == "bench":
            return benchmark.run(
                directory,
                options.category,
                options.language,
                options.project,
                options.refresh_baseline,
                extra,
                options.archive_argument,
                options.binary,
            )
        return 0
    except (policy.TestError, OSError, ValueError, KeyError) as error:
        print(f"test: {error}", file=sys.stderr)
        return 1


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
