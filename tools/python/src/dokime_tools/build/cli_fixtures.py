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
import hashlib
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
from dokime_tools.build import configuration, paths
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

    # Build the CLI Through the Project Task.
    subprocess.run(
        ["mise", "run", "build", "--", "--package", "dokime-cli"],
        cwd=root,
        check=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    cli = root / "target/debug" / f"dokime{extension}"
    output = paths.output(root, "build", profile) / "fixtures/cli"
    output.mkdir(parents=True, exist_ok=True)
    # A Freshly Generated Project Has No Tool Lock; Resolve Its Declared Tools Unlocked.
    environment = {**os.environ, "CARGO_TARGET_DIR": str(root / "target"), "MISE_LOCKED": "0"}
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    environment.pop("VIRTUAL_ENV", None)
    environment.pop("PYTHONPATH", None)
    if language == "rust":
        scratch_directory = tempfile.TemporaryDirectory(prefix="dokime-cli-")
        scratch = scratch_directory.name
        try:
            project = Path(scratch) / "rust"
            example = project / "test"

            # Generate the User's Rust Project With the Real Command-Line Tool.
            subprocess.run(
                [str(cli), "new", str(project), "--lang", "rust"],
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
            sdk = (root / "sdk/rust").as_posix()
            result = subprocess.run(
                ["mise", "-C", str(example), "run", "build", "--", "--config", f"patch.crates-io.dokime.path='{sdk}'"],
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
            shutil.copytree(project, output / "rust", dirs_exist_ok=True)
        finally:
            scratch_directory.cleanup()
        shutil.copy2(root / "target/debug" / f"dokime-example-test{extension}", output / f"dokime-example-test{extension}")
    else:
        scratch_directory = tempfile.TemporaryDirectory(prefix="dokime-cli-")
        scratch = scratch_directory.name
        try:
            project = Path(scratch) / language
            example = project if language == "python" else project / ("Test" if language == "csharp" else "test")
            arguments = [str(cli), "new", str(project), "--lang", language]
            if language == "cpp":
                sdk = root / "target/cp/tests/dokime-cpp.tar.gz"
                archive_file = sdk.open("rb")
                try:
                    digest = hashlib.file_digest(archive_file, "sha256").hexdigest()
                finally:
                    archive_file.close()
                arguments += ["--sdk-url", sdk.as_posix(), "--sdk-sha256", digest]
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
                wheel = next((paths.output(root, "build", profile, "any") / "python").glob("*.whl"))
                manifest = (example / "pyproject.toml").open("a", encoding="utf-8")
                try:
                    manifest.write(f"\n[tool.uv.sources]\ndokime-sdk = {{ path = {json.dumps(wheel.as_posix())} }}\n")
                finally:
                    manifest.close()
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
                    ["mise", "-C", "sdk/csharp", "run", "build:starter", "--", "--project", str(example / "Test.csproj")],
                    cwd=root,
                    env=environment,
                    check=True,
                    creationflags=spawn_flags(),
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                )
                environment["DOKIME_STARTER"] = str(example / f"bin/Release/net10.0/dokime-example-test{extension}")
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
                shutil.copytree(example / "bin/Release/net10.0", output / "csharp", dirs_exist_ok=True)
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
                child = dict(environment)
                if os.name == "nt":
                    child["PATH"] = str(root / "target/cp/v/x64-windows/bin") + os.pathsep + child["PATH"]
                for placement in ("managed", "remote"):
                    subprocess.run(
                        [
                            sys.executable,
                            str(root / "sdk/cpp/tests/support/run_plugin.py"),
                            str(root / "target/cp/consumer" / f"dokime-plugin{extension}"),
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
                for artifact in (root / "target/cp/consumer").iterdir():
                    if artifact.suffix in (".exe", ".dll") or artifact.name == "dokime-plugin":
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
