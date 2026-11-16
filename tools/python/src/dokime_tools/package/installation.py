# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Own Package Installation Operations."

# === Imports === #

# Standard Library Imports
import hashlib
import os
import subprocess
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy, temporary
from ..process import spawn_flags
from . import definition, verification

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def installed(directory: Path, settings: dict, root: Path, target: str) -> None:
    """Install From Published Assets and Prove No Source or Third-Party Fallback."""
    for item in definition.definitions(directory, "release", target):
        if not item.get("installers"):
            continue
        version = settings["tag"].removeprefix("v")
        provider = settings["provider"]
        repository = settings.get("repository") if provider == "github" else settings["project_id"]
        for installer in item["installers"]:
            if settings["tag"] == "nightly" and installer != "mise":
                continue
            install_directory = temporary.Directory()
            install = install_directory.path
            try:
                if installer == "mise":
                    tool = f"{provider}:{repository}[asset_pattern={item['output']},bin={item['binary']}]@{version}"
                    environment = os.environ.copy()
                    environment.pop("MISE_ENABLE_TOOLS", None)
                    subprocess.run(
                        ["mise", "install-into", tool, str(install)],
                        stdin=sys.stdin,
                        stdout=sys.stdout,
                        stderr=sys.stderr,
                        creationflags=spawn_flags(),
                        cwd=install.parent,
                        env=environment,
                        check=True,
                    )
                    matches = list(install.rglob(item["binary"]))
                else:
                    command = [
                        "cargo",
                        "binstall",
                        item["package"],
                        "--version",
                        "=" + version,
                        "--target",
                        item["triple"],
                        "--root",
                        str(install),
                        "--no-confirm",
                        "--disable-strategies",
                        "quick-install,compile",
                        "--strategies",
                        "crate-meta-data",
                    ]
                    subprocess.run(
                        command,
                        stdin=sys.stdin,
                        stdout=sys.stdout,
                        stderr=sys.stderr,
                        creationflags=spawn_flags(),
                        cwd=install,
                        check=True,
                    )
                    matches = [install / "bin" / item["binary"]]
                if len(matches) != 1 or not matches[0].is_file():
                    raise policy.ToolError(f"Package Product: {installer} did not install the declared executable")
                expected = hashlib.sha256(verification.binary_bytes(root / item["output"], item["binary"])).digest()
                if hashlib.sha256(matches[0].read_bytes()).digest() != expected:
                    raise policy.ToolError(f"Package Product: {installer} did not install the exact published asset")
                subprocess.run(
                    [str(matches[0]), *item["verify"]],
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                    creationflags=spawn_flags(),
                    cwd=install,
                    check=True,
                )
            finally:
                install_directory.close()


# === Private Functions === #

# === Entry Point === #
