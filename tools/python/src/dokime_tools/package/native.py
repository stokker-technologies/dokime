# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Invoke the Built CLI on Its Matching Local Distribution Kit."

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import subprocess
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import paths
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def package(root: Path, profile: str) -> None:
    source = paths.output(root, "build", profile) / "product/distribution"
    kit = json.loads((source / "distribution.json").read_text(encoding="utf-8"))
    windows = "windows" in kit["target"]
    signed = windows and bool(os.environ.get("TAURI_SIGNING_PRIVATE_KEY"))
    stable = os.environ.get("DOKIME_RELEASE_CONTEXT") == "true" or os.environ.get("GITHUB_REF", "").startswith("refs/tags/v")
    if windows and stable and not signed:
        raise ValueError("Package Desktop: Stable Publication Requires the Updater Signing Key")
    tag = "nightly" if kit["channel"] == "nightly" else f"v{kit['version']}"
    environment = dict(os.environ)
    if signed:
        environment.setdefault("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "")
    environment.setdefault("TAURI_UPDATER_ARTIFACT_BASE_URL", f"https://github.com/stokker-technologies/dokime/releases/download/{tag}")
    command = [
        str(source / "bin" / ("dokime.exe" if windows else "dokime")),
        "installer",
        "package",
        "--manifest",
        str(root / "installer.toml"),
        "--artifacts",
        str(source),
        "--output",
        str(paths.output(root, "package", profile) / "rust"),
        "--format",
        "nsis" if windows else "deb,rpm",
    ]
    if signed:
        command.append("--sign")
    subprocess.run(command, cwd=root, env=environment, check=True, creationflags=spawn_flags())


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    package(Path.cwd(), parser.parse_args().profile)
