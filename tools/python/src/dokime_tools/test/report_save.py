# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Measure Large-Cycle Report Saves From Verified Bench Artifacts"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

# Third Party Imports

# Local Imports
from ..build import paths
from . import archive, policy

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--copies", type=int, choices=range(1, 129), default=32)
    options = parser.parse_args()
    root = policy.root()
    output = paths.output(root, "build", "bench")
    messages = output / "bench/compiler.json"
    if not messages.is_file():
        raise policy.TestError("missing benchmark compiler metadata; run build:rust:bench first")

    binary = report_binary(messages)
    plugin = output / "fixtures/plugins/obc-plugin.dokime"
    required = [messages.relative_to(root).as_posix(), binary.as_posix(), plugin.relative_to(root).as_posix()]
    archive.fixtures(root, "bench", required)
    environment = dict(os.environ, DOKIME_REPORT_MEASURE_COPIES=str(options.copies))
    environment.pop("STKR_TEST_REPORT_DIR", None)
    result = subprocess.run(
        [str(root / binary), "--bench"],
        cwd=root / "crates/dokime-runtime",
        env=environment,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
        check=False,
    )
    return result.returncode


# === Private Functions === #


def report_binary(messages: Path) -> Path:
    binaries = set()
    for line in messages.read_text(encoding="utf-8").splitlines():
        event = json.loads(line)
        if event.get("reason") != "compiler-artifact":
            continue
        target = event.get("target", {})
        selected = target.get("name") == "system-runtime" and "bench" in target.get("kind", [])
        if not selected or event.get("manifest_path") != "crates/dokime-runtime/Cargo.toml":
            continue
        binary = Path(event["executable"])
        if binary.is_absolute() or ".." in binary.parts:
            raise policy.TestError("benchmark executable escapes the repository")
        binaries.add(binary)
    if len(binaries) != 1:
        raise policy.TestError("expected one built system-runtime benchmark")
    return binaries.pop()


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
