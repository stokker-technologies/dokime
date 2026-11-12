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


def build(profile: str) -> None:
    root = configuration.ROOT
    extension = ".exe" if os.name == "nt" else ""

    # Build the CLI Through the Project Task.
    subprocess.run(["mise", "run", "build", "--", "--package", "dokime-cli-next"], cwd=root, check=True, creationflags=spawn_flags())
    cli = root / "target/debug" / f"dokime-next{extension}"
    output = paths.output(root, "build", profile) / "fixtures/cli"
    output.mkdir(parents=True, exist_ok=True)
    environment = {**os.environ, "CARGO_TARGET_DIR": str(root / "target")}
    with tempfile.TemporaryDirectory(prefix="dokime-cli-") as scratch:
        project = Path(scratch) / "rust"

        # Generate the User's Rust Project With the Real Command-Line Tool.
        subprocess.run([str(cli), "new", str(project), "--lang", "rust"], cwd=root, check=True, creationflags=spawn_flags())
        subprocess.run(["mise", "trust", str(project / "mise.toml")], cwd=root, check=True, creationflags=spawn_flags())

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
    build(parser.parse_args().profile)
