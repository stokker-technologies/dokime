# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Select Repository Tasks and Dispatch Focused Cargo Commands Directly."

# === Imports === #

# Standard Library Imports
import argparse
import os
import shlex
import subprocess
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.test import lanes

from ..build import cargo
from ..process import spawn_flags
from ..task import bench, build, check, format, lint, package, publish, test
from ..test import focused
from . import transport

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("format", "lint", "check", "build", "test", "bench", "package", "publish", "screenshots"))
    parser.add_argument("--language", choices=("rust", "python", "typescript", "toml"))
    parser.add_argument("--layer", choices=("unit", "integration", "acceptance", "system"))
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    parser.add_argument("--artifact", choices=("product", "tests", "bench"), default="product")
    parser.add_argument("--package")
    parser.add_argument("--features")
    parser.add_argument("--target")
    parser.add_argument("--exact")
    parser.add_argument("--story", help="Run One Rust Story Module in a Focused Package")
    parser.add_argument("--changed", nargs="+", help="Run Rust Tests Affected by These Repository Paths")
    parser.add_argument("--plan", action="store_true", help="Print Focused Cargo Commands Without Compiling")
    parser.add_argument("--no-capture", action="store_true", help="Show Live Output from Focused Rust Stories")
    parser.add_argument("--no-run", action="store_true", help="Compile the Selected Rust Tests Without Running Them")
    parser.add_argument("--timings", action="store_true", help="Retain Native Cargo Build Timings")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--coverage", action="store_true")
    parser.add_argument("--repository", action="store_true")
    parser.add_argument("--bindings", action="store_true")
    parser.add_argument("--supply", action="store_true")
    parser.add_argument("--wheel", action="store_true")
    parser.add_argument("--dry", action="store_true")
    parser.add_argument("--stage", choices=("assets", "finalize"))
    parser.add_argument("--registry", choices=("pypi",), help="Publish One Registry Package from Its Own Trusted Publisher Job")
    parser.add_argument("--channel", choices=("stable", "nightly"), default="stable")
    parser.add_argument("--project", choices=("sdk/python", "tools/python"))
    parser.add_argument("--scene", choices=("cycle", "plots", "report", "history", "library", "cycles", "touch"))
    parser.add_argument("--select")
    parser.add_argument("--filterset", help="Select Focused Rust Stories with a Nextest Filterset")
    parser.add_argument("--binary")
    parser.add_argument("--refresh-baseline", action="store_true")
    parser.add_argument("--once", action="store_true", help="Run Each Selected Benchmark Case Once")
    parser.add_argument("--cargo", nargs=argparse.REMAINDER, help="Run a Focused Cargo Build, Check or Rustc with Native Arguments")
    args = parser.parse_args(arguments)
    if args.no_capture and (args.operation != "test" or not args.package or args.artifact == "tests"):
        parser.error("--no-capture requires a focused source Rust test")
    if args.once and args.operation != "bench":
        parser.error("--once requires bench")
    if args.once and args.refresh_baseline:
        parser.error("--once cannot promote a benchmark baseline")
    if args.cargo is not None:
        if args.operation != "build" or args.package or not args.cargo or args.cargo[0] not in ("build", "check", "rustc"):
            parser.error("--cargo requires build and a native build, check or rustc command")
        workspace_check = args.cargo[0] == "check" and "--workspace" in args.cargo[1:]
        package_selection = any(
            argument in ("-p", "--package") or argument.startswith("--package=") or argument.startswith("-p") and len(argument) > 2
            for argument in args.cargo[1:]
        )
        if not package_selection and not workspace_check:
            parser.error("--cargo requires an explicit Cargo package or an explicit workspace check")
        if (
            args.profile != "dev"
            or args.check
            or args.features
            or args.target
            or args.exact
            or args.story
            or args.changed
            or args.plan
            or args.no_run
            or args.artifact != "product"
            or args.language not in (None, "rust")
        ):
            parser.error("pass compiler selectors after --cargo")
        transport.receive()
        command = args.cargo[1:]
        if "--locked" not in command:
            command = ["--locked", *command]
        if args.timings and not any(argument == "--timings" or argument.startswith("--timings=") for argument in command):
            command = ["--timings", *command]
        return cargo.focused(Path.cwd(), args.cargo[0], command)
    if args.timings and not (
        args.changed
        and args.operation == "test"
        or args.package
        and args.operation in ("build", "test")
        or args.operation == "build"
        and args.artifact == "tests"
    ):
        parser.error("--timings requires a focused build/test or the Tests artifact")
    if not args.plan:
        transport.receive()
    if args.operation == "screenshots":
        import dokime_tools.screenshots.run

        options = ["--check"] if args.check else []
        if args.scene:
            options += ["--scene", args.scene]
        return dokime_tools.screenshots.run.main(options)
    supported = {
        "lint": {"rust", "python", "typescript"},
        "build": {"rust", "python", "typescript"},
        "test": {"rust", "python", "typescript"},
        "bench": {"rust", "python"},
        "package": {"rust", "python"},
    }
    if args.language and args.operation in supported and args.language not in supported[args.operation]:
        parser.error("the selected operation has no implementation for this language")
    if args.operation == "build" and args.check and not args.package:
        parser.error("build --check requires --package")
    if args.changed:
        if args.operation != "test" or any(
            (args.package, args.features, args.exact, args.story, args.target, args.select, args.coverage, args.wheel)
        ):
            parser.error("--changed selects test packages, targets and required features from paths")
        if args.language not in (None, "rust") or args.profile != "dev":
            parser.error("--changed selects Rust tests in the Cargo test profile")
        try:
            return focused.changed(Path.cwd(), args.changed, args.layer, plan=args.plan, no_run=args.no_run, timings=args.timings)
        except ValueError as error:
            parser.error(str(error))
    if args.filterset and (args.operation != "test" or not args.package):
        parser.error("--filterset requires a focused Rust test package")
    if args.story and (args.operation != "test" or not args.package or args.exact or args.select):
        parser.error("--story requires a focused test package and cannot combine with --exact or --select")
    if args.package and args.operation == "test" and args.exact and args.select:
        parser.error("choose either --exact or --select")
    if args.no_run and not args.package:
        parser.error("--no-run requires a focused package or --changed")
    if (args.plan or args.timings) and args.operation not in ("build", "test"):
        parser.error("--plan and --timings require build or test")
    if args.no_run and args.operation != "test":
        parser.error("--no-run requires test")
    if args.features and not args.package:
        parser.error("--features requires --package")
    if args.package and args.operation == "check":
        if args.language != "rust":
            parser.error("check --package requires --language rust")
        for command in (
            ["mise", "run", "format:rust", "--", "--check", "--package", args.package],
            ["mise", "run", "lint:rust", "--", "--package", args.package],
        ):
            status = subprocess.run(
                command,
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
                check=False,
                creationflags=spawn_flags(),
            ).returncode
            if status:
                return status
        return 0
    if args.package and args.operation == "format":
        if args.language != "rust":
            parser.error("format --package requires --language rust")
        command = ["mise", "run", "format:rust", "--", "--package", args.package]
        if args.check:
            command.append("--check")
        return subprocess.run(
            command,
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            check=False,
            creationflags=spawn_flags(),
        ).returncode
    if args.package:
        if args.operation not in ("build", "test"):
            parser.error("--package requires build or test")
        if args.language not in (None, "rust"):
            parser.error("--package selects Cargo packages")
        if args.operation == "test" and args.artifact == "tests":
            if any((args.features, args.timings, args.no_run, args.plan, args.coverage)):
                parser.error("archived tests consume existing bytes; compiler and coverage options require a source test")
            target = args.target or ("lib" if args.layer == "unit" else args.layer) or ("integration" if args.story else None)
            if not target:
                parser.error("archived package tests require --target or --layer")
            category = (
                "unit"
                if target == "lib"
                else next((value for value in ("acceptance", "system") if target == value or target.startswith(f"{value}-")), "integration")
            )
            identity = args.package if target == "lib" else f"{args.package}::{target}"
            selection = ["-E", f"binary_id(={identity})"]
            if args.filterset:
                selection += ["-E", args.filterset]
            if args.exact:
                selection += ["--", "--exact", args.exact]
            elif args.story or args.select:
                selection += [args.story or args.select]
            if category == "system":
                from ..test import archive

                archive.install_wheels(Path.cwd(), args.profile)
            return lanes.run(Path.cwd(), category, "rust", ".", args.profile, selection)
        operation = "check" if args.operation == "build" and args.check else args.operation
        command = ["--locked", "--package", args.package]
        if args.timings:
            command.insert(0, "--timings")
        if args.features:
            command += ["--features", args.features]
        if operation == "test":
            if args.profile != "dev":
                parser.error("focused tests use the Cargo test profile")
            target = args.target or ("lib" if args.layer == "unit" else args.layer) or ("integration" if args.story else None)
            command += ["--lib"] if target == "lib" else ["--test", target] if target else ["--tests"]
            if args.filterset:
                command += ["-E", args.filterset]
            if args.no_capture:
                command += ["--no-capture"]
            if args.no_run:
                command += ["--no-run"]
            if args.exact:
                command += [args.exact, "--", "--exact"]
            elif args.story or args.select:
                command += [args.story or args.select]
        else:
            command += ["--profile", args.profile]
        if args.plan:
            if operation == "test":
                return focused.run(Path.cwd(), command, plan=True)
            print(focused.display(operation, command))
            return 0
        return cargo.focused(Path.cwd(), operation, command)
    if args.features:
        parser.error("--features requires a focused package build or test")
    if args.exact or args.target and args.operation != "package":
        parser.error("--exact and --target require a focused test (or --target for package)")
    if args.operation == "test" and not any((args.language, args.layer, args.coverage, args.wheel, args.project, args.select)):
        return subprocess.run(
            ["mise", "run", "ci:test", "--", "--profile", args.profile],
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            check=False,
            creationflags=spawn_flags(),
        ).returncode
    tasks: list[tuple[str, list[str]]] | None = []
    languages = [args.language] if args.language else ["rust", "python", "typescript"]
    profile = ["--profile", args.profile]
    if args.operation == "format":
        tasks = format.select(args, languages)
    elif args.operation == "lint":
        tasks = lint.select(args, languages)
    elif args.operation == "check":
        tasks = check.select(args, languages, profile)
    elif args.operation == "build":
        tasks = build.select(args, profile)
    elif args.operation == "test":
        tasks = test.select(args, languages, profile)
    elif args.operation == "bench":
        tasks = bench.select(args, parser)
    elif args.operation == "package":
        tasks = package.select(args, languages, profile)
    elif args.operation == "publish":
        tasks = publish.select(args, parser)
    else:
        parser.error("Select Operation: Choose a Supported Public Task")
    if tasks is None:
        return 0

    if not tasks:
        parser.error("the selected operation has no implementation for this language")
    if args.plan:
        for task, options in tasks:
            print(shlex.join(["mise", "run", task, "--", *options]))
        return 0
    if args.operation == "publish" and not args.dry:
        from ..release import guard

        guard.check()
    for task, options in tasks:
        if os.environ.get("COMPILER_CACHE_BACKEND"):
            from .build import run_command

            status = run_command(["mise", "run", task, "--", *options], task)
            if status:
                return status
            continue
        result = subprocess.run(
            ["mise", "run", task, "--", *options],
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            check=False,
            creationflags=spawn_flags(),
        )
        if result.returncode:
            return result.returncode
    transport.send("bench" if args.operation == "build" and args.artifact == "bench" else args.profile)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
