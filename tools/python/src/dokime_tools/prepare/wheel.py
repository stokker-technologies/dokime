# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare the exact installed client wheel for cross-language acceptance."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path

from dokime_tools.package.wheel import WheelSelectionError, select_wheel

# Third Party Imports
# Local Imports
from dokime_tools.test import archive, policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def main(argv: Sequence[str] | None = None) -> int:
    args = _parse_args(argv)
    root = Path(__file__).resolve().parents[5]
    platform = "windows" if sys.platform == "win32" else "linux"
    wheel_dir = root / "dist/build" / platform / "python"
    archive.fixtures(root, archive.tree(root), args.profile, [wheel_dir.relative_to(root).as_posix()])
    wheel = _unique_wheel(wheel_dir)

    # One immutable client wheel supplies all cross-language categories.
    # Mise prepares the shared workspace before installing this wheel.
    native_environment = root / ".venv"
    native_python = native_environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    _run(
        [
            "uv",
            "pip",
            "install",
            "--python",
            str(native_python),
            "--offline",
            "--reinstall",
            "--no-deps",
            str(wheel.resolve()),
        ]
    )

    return 0


# === Private Functions === #


def _unique_wheel(wheel_dir: Path) -> Path:
    try:
        return select_wheel(wheel_dir, "dokime-*.whl")
    except WheelSelectionError as error:
        if not error.wheels:
            raise SystemExit(f"No dokime wheel artifact exists in {wheel_dir}.") from error
        raise SystemExit(f"Expected one dokime wheel in {wheel_dir}, found: {error}.") from error


def _run(
    command: Sequence[str],
    *,
    check: bool = True,
    env: Mapping[str, str] | None = None,
) -> int:
    result = subprocess.run(command, check=check, env=env)
    return result.returncode


def _parse_args(argv: Sequence[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=policy.PROFILES, default="dev")
    return parser.parse_args(argv)


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
