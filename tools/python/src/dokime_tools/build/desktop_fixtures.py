# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare desktop SDK fixtures without a retired host"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import shutil
import subprocess
import sys

# Third Party Imports
from harmos_tools.build import paths

# Local Imports
from dokime_tools.build import configuration
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
    output = paths.output(root, "build", profile) / "fixtures/desktop"
    output.mkdir(parents=True, exist_ok=True)

    # Build the Current CLI Through Its Focused Project Task
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
    shutil.copy2(cli, output / cli.name)

    # Bundle the Prepared Shared SDK Fixture Without Compiling in a Story
    executable = paths.output(root, "build", profile) / "fixtures/host" / f"dokime-rust-thermal{extension}"
    bundle = output / "thermal.dokime"
    bundle.unlink(missing_ok=True)
    result = subprocess.run(
        [str(cli), "plugin", "bundle", "--executable", str(executable), "--out", str(bundle)],
        cwd=root,
        check=False,
        text=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=subprocess.PIPE,
    )
    print(result.stderr, end="", file=sys.stderr, flush=True)
    result.check_returncode()

    # Extend the Tests Fixture Inventory Without Replacing Product Bytes
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
