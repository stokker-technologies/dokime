# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate Organization Assets"""

# === Imports === #

# Standard Library Imports
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
BRAND = ROOT / "organization"
GENERATED = ROOT / "crates/dokime-gui/src/generated/organization"

# === Types === #

# === Classes === #

# === Public Functions === #


def generate() -> None:
    """Materialize With The Small Native-Free Owner; Cargo Checks Source Freshness."""
    subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--locked",
            "-p",
            "dokime-organization",
            "--bin",
            "dokime-organization",
            "--",
            str(BRAND),
            str(GENERATED),
        ],
        cwd=ROOT,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )


def generate_icons() -> None:
    """Generate Desktop Icons From The Brand's PNG Icon Master."""
    destination = ROOT / "crates/dokime-gui/src-tauri/icons"
    cli = ROOT / "crates/dokime-gui/node_modules/@tauri-apps/cli/tauri.js"
    with tempfile.TemporaryDirectory(prefix="organization-icons-") as directory:
        command = ["node", str(cli), "icon", str(BRAND / "icons/icon.png"), "--output", directory]
        subprocess.run(
            command,
            cwd=ROOT,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        subprocess.run(
            [*command, "--png", "64"],
            cwd=ROOT,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        outputs = sorted(path for path in Path(directory).iterdir() if path.is_file())
        for path in outputs:
            shutil.copyfile(path, destination / path.name)
        _write_json(
            destination / "icon-integrity.json",
            {
                "algorithm": "sha256",
                "source": _digest(BRAND / "icons/icon.png"),
                "outputs": [_digest(destination / path.name) for path in outputs],
            },
        )


# === Private Functions === #


def _write_json(path: Path, value: object) -> None:
    text = json.dumps(value, indent=2) + "\n"
    _write_text(path, text)


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    encoded = text.encode("utf-8")
    if not path.exists() or path.read_bytes() != encoded:
        path.write_bytes(encoded)


def _digest(path: Path) -> dict[str, str]:
    return {"path": path.relative_to(ROOT).as_posix(), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--icons", action="store_true", help="Also Regenerate Desktop Icons")
    args = parser.parse_args()
    generate()
    if args.icons:
        generate_icons()
