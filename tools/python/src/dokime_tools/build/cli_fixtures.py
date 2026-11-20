# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build the Command-Line Starter Fixture Through Its Own Mise Task."""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.test.policy
from dokime_tools.build import cargo, configuration, paths
from dokime_tools.process import spawn_flags
from dokime_tools.test import archive

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str, language: str = "rust") -> None:
    root = configuration.ROOT
    extension = ".exe" if os.name == "nt" else ""

    # Use the Command-Line Tool From the Matching Product Build.
    cli = paths.output(root, "build", profile) / "bin" / f"dokime{extension}"
    if not cli.is_file():
        raise ValueError(f"Build CLI Fixtures: Run build:product --profile {profile} or Download Its build-cli Artifact: {cli}")
    output = paths.output(root, "build", profile) / "fixtures/cli"
    output.mkdir(parents=True, exist_ok=True)
    # A Freshly Generated Project Has No Tool Lock; Resolve Its Declared Tools Unlocked.
    # The Rust Starter Builds in Debug Regardless of the Fixture Output Profile.
    environment = {**cargo.compiler_environment("dev"), "CARGO_TARGET_DIR": str(root / "target"), "MISE_LOCKED": "0"}
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    environment.pop("VIRTUAL_ENV", None)
    environment.pop("PYTHONPATH", None)
    if language == "rust":
        scratch_directory = tempfile.TemporaryDirectory(prefix="dokime-cli-")
        scratch = scratch_directory.name
        try:
            project = Path(scratch) / "rust"
            example = project

            # Generate the User's Rust Project With the Real Command-Line Tool.
            subprocess.run(
                [str(cli), "new", str(project), "--lang", "rust", "--from", str(root)],
                cwd=root,
                check=True,
                creationflags=spawn_flags(),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )
            subprocess.run(
                ["mise", "trust", str(example / "mise.toml")],
                cwd=root,
                check=True,
                creationflags=spawn_flags(),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )

            # Build the Generated Source Against the SDK Candidate.
            result = subprocess.run(
                ["mise", "-C", str(example), "run", "build"],
                cwd=root,
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                creationflags=spawn_flags(),
            )
            print(result.stdout, flush=True)
            print(result.stderr, flush=True)
            result.check_returncode()
            # Keep Sources Separate From the Extensionless Native Executable.
            shutil.copytree(project, output / "rust-source", dirs_exist_ok=True)
        finally:
            scratch_directory.cleanup()
        shutil.copy2(root / "target/debug" / f"rust{extension}", output / f"rust{extension}")
    else:
        scratch_directory = tempfile.TemporaryDirectory(prefix="dokime-cli-")
        scratch = scratch_directory.name
        try:
            project = Path(scratch) / language
            example = project
            arguments = [str(cli), "new", str(project), "--lang", language, "--from", str(root)]
            subprocess.run(
                arguments,
                cwd=root,
                env=environment,
                check=True,
                creationflags=spawn_flags(),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )
            subprocess.run(
                ["mise", "trust", str(example / "mise.toml")],
                cwd=root,
                check=True,
                creationflags=spawn_flags(),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )
            if language == "python":
                environment["DOKIME_TEST_HOST"] = str(paths.output(root, "build", profile) / "fixtures/host" / f"dokime-host{extension}")
                subprocess.run(
                    ["mise", "-C", str(example), "run", "build"],
                    cwd=root,
                    env=environment,
                    check=True,
                    creationflags=spawn_flags(),
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                )
                subprocess.run(
                    ["mise", "-C", str(example), "run", "test"],
                    cwd=root,
                    env=environment,
                    check=True,
                    creationflags=spawn_flags(),
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                )
            elif language == "csharp":
                subprocess.run(
                    ["mise", "-C", str(example), "run", "build"],
                    cwd=root,
                    env=environment,
                    check=True,
                    creationflags=spawn_flags(),
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                )
                environment["DOKIME_STARTER"] = str(example / f"assets/plugin/Csharp.Plugin{extension}")
                subprocess.run(
                    ["mise", "-C", "sdk/csharp", "run", "test:starter"],
                    cwd=root,
                    env=environment,
                    check=True,
                    creationflags=spawn_flags(),
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                )
                shutil.copytree(example / "assets/plugin", output / "csharp", dirs_exist_ok=True)
            else:
                subprocess.run(
                    ["mise", "-C", "sdk/cpp", "run", "consumer", "--", "--source", str(example)],
                    cwd=root,
                    env=environment,
                    check=True,
                    creationflags=spawn_flags(),
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                )
                consumer = root / "target/cp/consumer"
                child = dict(environment)
                if os.name == "nt":
                    child["PATH"] = str(root / "target/cp/v/x64-windows/bin") + os.pathsep + child["PATH"]
                for placement in ("managed", "remote"):
                    subprocess.run(
                        [
                            sys.executable,
                            str(root / "sdk/cpp/tests/support/run_plugin.py"),
                            str(consumer / f"dokime-plugin{extension}"),
                            str(root / "target/cp/tests/tests" / f"dokime-plugin-journey{extension}"),
                            placement,
                        ],
                        cwd=root,
                        env=child,
                        check=True,
                        creationflags=spawn_flags(),
                        stdin=sys.stdin,
                        stdout=sys.stdout,
                        stderr=sys.stderr,
                    )
                (output / "cpp").mkdir(exist_ok=True)
                for artifact in consumer.iterdir():
                    if artifact.suffix == ".dll" or artifact.name == f"dokime-plugin{extension}":
                        shutil.copy2(artifact, output / "cpp" / artifact.name)
            shutil.copytree(project, output / language / "source", ignore=shutil.ignore_patterns(".venv", "bin", "obj"), dirs_exist_ok=True)
        finally:
            scratch_directory.cleanup()

    # Extend the Portable Tests Fixture Inventory.
    stamp = paths.output(root, "build", profile) / "fixtures/stamp.json"
    record = json.loads(stamp.read_text(encoding="utf-8")) if stamp.is_file() else {"files": {}, "metadata": {}}
    files = [*record["files"], *(path.relative_to(root).as_posix() for path in output.rglob("*") if path.is_file())]
    archive.stamp(root, profile, files, record["metadata"])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=dokime_tools.test.policy.PROFILES, default="dev")
    parser.add_argument("--language", choices=("rust", "python", "csharp", "cpp"), default="rust")
    options = parser.parse_args()
    build(options.profile, options.language)
