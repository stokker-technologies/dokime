# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"""Select Repository Tasks And Dispatch Focused Cargo Commands Directly."""

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from ..build import cargo
from ..process import spawn_flags
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
    parser.add_argument("--artifact", choices=("product", "tests", "bench", "fixtures"), default="product")
    parser.add_argument("--package")
    parser.add_argument("--features")
    parser.add_argument("--target")
    parser.add_argument("--exact")
    parser.add_argument("--story", help="Run One Rust Story Module in a Focused Package")
    parser.add_argument("--changed", nargs="+", help="Run Rust Tests Affected by These Repository Paths")
    parser.add_argument("--plan", action="store_true", help="Print Focused Cargo Commands Without Compiling")
    parser.add_argument("--no-run", action="store_true", help="Compile the Selected Rust Tests Without Running Them")
    parser.add_argument("--timings", action="store_true", help="Retain Native Cargo Build Timings")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--coverage", action="store_true")
    parser.add_argument("--repository", action="store_true")
    parser.add_argument("--bindings", action="store_true")
    parser.add_argument("--supply", action="store_true")
    parser.add_argument("--wasm", action="store_true")
    parser.add_argument("--wheel", action="store_true")
    parser.add_argument("--starter", action="store_true")
    parser.add_argument("--dry", action="store_true")
    parser.add_argument("--stage", choices=("assets", "python", "finalize"))
    parser.add_argument("--channel", choices=("stable", "nightly"), default="stable")
    parser.add_argument("--project", choices=("sdk/python", "tools/python"))
    parser.add_argument("--scene", choices=("cycle", "plots", "report", "history", "library", "cycles", "touch"))
    parser.add_argument("--select")
    parser.add_argument("--artifacts")
    parser.add_argument("--binary")
    parser.add_argument("--refresh-baseline", action="store_true")
    parser.add_argument("--cargo", nargs=argparse.REMAINDER, help="Run A Focused Cargo Build, Check Or Rustc With Native Arguments")
    args = parser.parse_args(arguments)
    if args.cargo is not None:
        if args.operation != "build" or args.package or not args.cargo or args.cargo[0] not in ("build", "check", "rustc"):
            parser.error("--cargo requires build and a native build, check or rustc command")
        if not any(
            argument in ("-p", "--package") or argument.startswith("--package=") or argument.startswith("-p") and len(argument) > 2
            for argument in args.cargo[1:]
        ):
            parser.error("--cargo requires an explicit Cargo package")
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
    transport.receive()
    if args.operation == "screenshots":
        from ..screenshots.run import main as screenshots

        options = ["--check"] if args.check else []
        if args.scene:
            options += ["--scene", args.scene]
        return screenshots(options)
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
            (args.package, args.features, args.exact, args.story, args.target, args.select, args.coverage, args.wheel, args.starter)
        ):
            parser.error("--changed selects test packages, targets and required features from paths")
        if args.language not in (None, "rust") or args.profile != "dev":
            parser.error("--changed selects Rust tests in the Cargo test profile")
        try:
            return focused.changed(Path.cwd(), args.changed, args.layer, plan=args.plan, no_run=args.no_run, timings=args.timings)
        except ValueError as error:
            parser.error(str(error))
    if args.story and (args.operation != "test" or not args.package or args.exact or args.select):
        parser.error("--story requires a focused test package and cannot combine with --exact or --select")
    if args.package and args.operation == "test" and args.exact and args.select:
        parser.error("choose either --exact or --select")
    if (args.plan or args.no_run) and not args.package:
        parser.error("--plan and --no-run require a focused package or --changed")
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
            from ..test import run

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
            if args.exact:
                selection += ["--", "--exact", args.exact]
            elif args.story or args.select:
                selection += [args.story or args.select]
            if category == "system":
                from ..test import archive

                archive.install_wheels(Path.cwd(), args.profile)
            return run.test(Path.cwd(), category, "rust", ".", args.profile, selection)
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
    if args.operation == "test" and not any(
        (args.language, args.layer, args.coverage, args.wheel, args.starter, args.project, args.select, args.artifacts)
    ):
        return subprocess.run(
            ["mise", "run", "ci:test", "--", "--profile", args.profile],
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            check=False,
            creationflags=spawn_flags(),
        ).returncode
    tasks: list[tuple[str, list[str]]] = []
    languages = [args.language] if args.language else ["rust", "python", "typescript"]
    profile = ["--profile", args.profile]
    if args.operation in ("format", "lint"):
        if args.operation == "format" and not args.language:
            languages.append("toml")
        tasks = [(f"{args.operation}:{language}", ["--check"] if args.check else []) for language in languages]
    elif args.operation == "check":
        if args.repository:
            tasks = [("check:repository", [])]
        elif args.bindings:
            tasks = [("check:python:bindings", profile)]
        elif args.supply:
            tasks = [("check:rust:supply", [])]
        elif args.wasm:
            return cargo.focused(Path.cwd(), "check", ["--locked", "-p", "dokime-domain", "--target", "wasm32-unknown-unknown"])
        else:
            tasks = [("check:repository", [])]
            tasks += [(f"check:{language}", []) for language in languages if language != "toml"]
            tasks += [("check:typescript:generated", profile)] if "typescript" in languages else []
    elif args.operation == "build":
        if args.artifact == "fixtures":
            parser.error("native Python client fixtures were removed; use the pure SDK's focused stories")
        elif args.artifact == "bench":
            tasks = [("build:rust:bench", ["--binary", args.binary] if args.binary else [])]
        elif args.artifact == "tests":
            tasks = [("build:rust:tests", profile + (["--timings"] if args.timings else []))]
        elif args.language:
            tasks = [(f"build:{args.language}", profile)]
        else:
            tasks = [("build:product", profile)]
    elif args.operation == "test":
        if args.wheel:
            tasks = [("test:python:wheel", profile + (["--stubs"] if args.bindings else []))]
        elif args.starter:
            tasks = [("test:rust:starter", profile)]
        elif args.coverage:
            tasks = [(f"test:{language}:coverage", profile) for language in languages if language in ("rust", "python")]
        else:
            layers = [args.layer] if args.layer else ["unit", "integration", "acceptance", "system"]
            if "python" in languages and "integration" in layers:
                tasks.append(
                    (
                        "test:python:integration",
                        profile + ["--project", "tools/python"] + (["--select", args.select] if args.select else []),
                    )
                )
            if args.project != "tools/python":
                for language in languages:
                    for layer in layers:
                        if language == "typescript" and layer == "acceptance":
                            continue
                        if language == "python" and not (Path("sdk/python/tests") / layer).is_dir():
                            continue
                        options = profile.copy()
                        if args.select:
                            options += ["--select", args.select]
                        if args.artifacts and language == "python" and layer in ("acceptance", "system"):
                            options += ["--artifacts", args.artifacts]
                        tasks.append((f"test:{language}:{layer}", options))
    elif args.operation == "bench":
        layers = [args.layer] if args.layer else ["unit", "integration", "system"]
        if "acceptance" in layers:
            parser.error("bench layers are unit, integration and system")
        for language in [args.language] if args.language else ["rust"]:
            for layer in layers:
                if language != "rust" and (language != "python" or layer != "system"):
                    continue
                options = ["--refresh-baseline"] if args.refresh_baseline else []
                if args.binary:
                    options += ["--binary", args.binary]
                if args.select:
                    options += ["--select", args.select]
                tasks.append((f"bench:{language}:{layer}", options))
    elif args.operation == "package":
        tasks = [
            (f"package:{language}", profile + (["--target", args.target] if args.target else []))
            for language in languages
            if language != "typescript"
        ]
    elif args.operation == "publish":
        if args.channel == "nightly" and not args.dry:
            if args.stage:
                parser.error("nightly publishes the complete package inventory")
            from .. import policy
            from ..release import nightly

            nightly.publish(policy.root())
            return 0
        if args.dry:
            tasks = [("release:publish:dry", [])]
        else:
            # Python Registry Publication Waits For The Package Name Decision; Wheels Still Have Dry-Run Proof.
            tasks = [(f"release:publish:{stage}", []) for stage in ([args.stage] if args.stage else ["assets", "finalize"])]
    if not tasks:
        parser.error("the selected operation has no implementation for this language")
    if args.operation == "publish" and not args.dry:
        from .workflow import publication_guard

        publication_guard()
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
