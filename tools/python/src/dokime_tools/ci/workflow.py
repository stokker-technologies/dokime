# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Own Provider Operations and Release Ordering Behind Project Tasks."

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..process import spawn_flags
from ..release import guard
from . import bindings, mirror, nightly_freshness, nightly_report, release_mode, required, review, transport
from .matrices import SDK_LANGUAGES, SDKS

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run Provider Workflow Operations")
    parser.add_argument(
        "operation",
        choices=(
            "nightly-freshness",
            "nightly-report",
            "release-mode",
            "review",
            "required",
            "mirror",
            "release-prepare",
            "release-tag",
            "release-verify",
            "rehearse",
            "bindings",
            "send",
            "receive",
            "summary",
            "sdk-check",
            "sdk-test",
        ),
    )
    parser.add_argument("--profile", choices=("dev", "release"), default="release")
    parser.add_argument("--target")
    parser.add_argument("--language", choices=("csharp", "cpp"))
    parser.add_argument("--stage-only", action="store_true")
    parser.add_argument("--channel", choices=("stable", "nightly"), default="stable")
    options = parser.parse_args(arguments)
    operation = options.operation
    if operation == "send":
        transport.send(options.profile)
        return 0
    if operation == "receive":
        transport.receive()
        return 0
    if operation == "summary":
        destination = os.environ.get("GITHUB_STEP_SUMMARY")
        if destination:
            summary = Path(destination)
            text = f"Execution Host Equivalence on Linux and Windows: {', '.join(SDKS[language] for language in SDK_LANGUAGES)}\n"
            previous = summary.read_text(encoding="utf-8") if summary.exists() else ""
            summary.write_text(previous + text, encoding="utf-8", newline="\n")
        return 0
    if operation in {"sdk-check", "sdk-test"}:
        if options.language is None:
            parser.error("Select SDK: Add --language csharp or cpp")
        language = "C#" if options.language == "csharp" else "C++"
        tools = ["dotnet"] if options.language == "csharp" else ["cmake", "ninja", "pipx:clang-format"]
        print(f"Install {language} SDK Tools", flush=True)
        result = subprocess.run(
            ["mise", "-C", f"sdk/{options.language}", "install", *tools],
            check=False,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        if result.returncode:
            print(
                f"Install {language} SDK Tools Failed (Exit {result.returncode}): Fix the Tool Installation Error Above and Rerun mise run ci:{operation} -- --language {options.language} --profile {options.profile}",
                file=sys.stderr,
                flush=True,
            )
            return result.returncode
        commands = ["check"] if operation == "sdk-check" else ["build", "test"]
        if options.language == "csharp" and operation == "sdk-check":
            commands = ["restore", "check"]
        for command in commands:
            invocation = ["mise", "-C", f"sdk/{options.language}", "run", "--output", "interleave", command]
            if options.language == "cpp":
                preset = "release" if options.profile == "release" else "tests"
                invocation += ["--", "--preset", preset]
                if operation == "sdk-test":
                    packages = sorted(Path("target/sdk-cpp").glob("dokime-cpp-*.tar.gz"))
                    if len(packages) != 1:
                        raise RuntimeError("Test C++ SDK: Download the Matching Build SDK C++ Artifact Into target/sdk-cpp")
                    invocation += ["--sdk-package", str(packages[0].resolve())]
                if command == "test":
                    invocation.append("--verbose")
            print(f"{command.title()} {language} SDK", flush=True)
            result = subprocess.run(
                invocation,
                check=False,
                creationflags=spawn_flags(),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )
            if result.returncode:
                print(
                    f"{command.title()} {language} SDK Failed (Exit {result.returncode}): Fix the Reported Error Above and Rerun {subprocess.list2cmdline(invocation)}",
                    file=sys.stderr,
                    flush=True,
                )
                return result.returncode
        return 0
    transport.receive()
    if operation == "nightly-freshness":
        nightly_freshness.run()
    elif operation == "nightly-report":
        nightly_report.run()
    elif operation == "release-mode":
        release_mode.run()
    elif operation == "review":
        review.main()
    elif operation == "required":
        required.run()
    elif operation == "mirror":
        mirror.run()
    elif operation == "bindings":
        return bindings.run(options)
    else:
        from ..release import run

        if operation == "release-prepare":
            if run.main(["prepare"]):
                return 1
            from ..test import runners

            token = os.environ.get("GH_TOKEN", "")
            os.environ["GH_TOKEN"] = os.environ["RUNNER_GROUP_READ_TOKEN"]
            try:
                runners.main()
            finally:
                os.environ["GH_TOKEN"] = token
            return run.main(["review"])
        if operation == "release-tag":
            guard.check()
            return run.main(["tag"])
        if operation == "release-verify":
            if options.channel == "nightly":
                from ..release import nightly

                nightly.verify(policy.root())
                return 0
            return run.main(["verify"])
        args = ["rehearse", "--profile", options.profile, "--channel", options.channel]
        if options.target:
            args += ["--target", options.target]
        if options.stage_only:
            args += ["--stage-only"]
        return run.main(args)
    transport.send(options.profile)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
