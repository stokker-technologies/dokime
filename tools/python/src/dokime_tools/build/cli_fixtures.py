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

# Third Party Imports
# Local Imports
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
        ["mise", "run", "build", "--", "--package", "dokime-cli-next"],
        cwd=root,
        check=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    cli = root / "target/debug" / f"dokime-next{extension}"
    output = paths.output(root, "build", profile) / "fixtures/cli"
    output.mkdir(parents=True, exist_ok=True)
    # A Freshly Generated Project Has No Tool Lock; Resolve Its Declared Tools Unlocked.
    environment = {**os.environ, "CARGO_TARGET_DIR": str(root / "target"), "MISE_LOCKED": "0"}
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    if language == "rust":
        with tempfile.TemporaryDirectory(prefix="dokime-cli-") as scratch:
            project = Path(scratch) / "rust"

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
                ["mise", "trust", str(project / "mise.toml")],
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
                ["mise", "-C", str(project), "run", "build", "--", "--config", f"patch.crates-io.dokime.path='{sdk}'"],
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
        shutil.copy2(root / "target/debug" / f"dokime-starter{extension}", output / f"dokime-starter{extension}")
    else:
        with tempfile.TemporaryDirectory(prefix="dokime-cli-") as scratch:
            project = Path(scratch) / language
            arguments = [str(cli), "new", str(project), "--lang", language]
            if language == "cpp":
                sdk = root / "target/cp/tests/dokime-cpp.tar.gz"
                with sdk.open("rb") as archive_file:
                    digest = hashlib.file_digest(archive_file, "sha256").hexdigest()
                arguments += ["--sdk-url", sdk.as_uri(), "--sdk-sha256", digest]
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
                ["mise", "trust", str(project / "mise.toml")],
                cwd=root,
                check=True,
                creationflags=spawn_flags(),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )
            if language == "python":
                wheel = next((paths.output(root, "build", profile, "any") / "python").glob("*.whl"))
                config = Path(scratch) / "uv.toml"
                config.write_text(f"[sources]\ndokime-sdk = {{ path = {json.dumps(wheel.as_posix())} }}\n", encoding="utf-8")
                environment = dict(environment, UV_CONFIG_FILE=str(config))
                environment["DOKIME_TEST_HOST"] = str(
                    paths.output(root, "build", profile) / "fixtures/host" / f"dokime-host-next{extension}"
                )
                subprocess.run(
                    ["mise", "-C", str(project), "run", "build"],
                    cwd=root,
                    env=environment,
                    check=True,
                    creationflags=spawn_flags(),
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                )
                subprocess.run(
                    ["mise", "-C", str(project), "run", "test"],
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
                    ["mise", "-C", "sdk/csharp", "run", "build:starter", "--", "--project", str(project / "Starter.csproj")],
                    cwd=root,
                    env=environment,
                    check=True,
                    creationflags=spawn_flags(),
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                )
                environment["DOKIME_STARTER"] = str(project / f"bin/Release/net10.0/dokime-starter{extension}")
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
                shutil.copytree(project / "bin/Release/net10.0", output / "csharp", dirs_exist_ok=True)
            else:
                subprocess.run(
                    ["mise", "-C", "sdk/cpp", "run", "consumer", "--", "--source", str(project)],
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

    # Extend the Portable Tests Fixture Inventory.
    stamp = paths.output(root, "build", profile) / "fixtures/stamp.json"
    record = json.loads(stamp.read_text(encoding="utf-8")) if stamp.is_file() else {"files": {}, "metadata": {}}
    files = [*record["files"], *(path.relative_to(root).as_posix() for path in output.rglob("*") if path.is_file())]
    archive.stamp(root, profile, files, record["metadata"])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=configuration.PROFILES, default="dev")
    parser.add_argument("--language", choices=("rust", "python", "csharp", "cpp"), default="rust")
    options = parser.parse_args()
    build(options.profile, options.language)
