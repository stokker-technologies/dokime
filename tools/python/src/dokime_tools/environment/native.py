# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare the installed native wheel for cross-language acceptance."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
import argparse
import os
import subprocess
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path

# Third Party Imports

# Local Imports
from dokime_tools.package.wheel import WheelSelectionError, select_wheel

# Type Checking Imports

# === Public Exports === #

# === Constants === #

DEFAULT_WHEEL_DIR = Path("dist/python")
WHEEL_DIR_ENV = "DOKIME_PYTHON_TEST_WHEEL_DIR"


# === Types === #

# === Classes === #

# === Public Functions === #


def main(argv: Sequence[str] | None = None) -> int:
    args = _parse_args(argv)
    wheel_dir = Path(args.wheel_dir)

    wheel = _unique_wheel(wheel_dir)

    # One immutable wheel supplies all cross-language and native categories.
    # Mise prepares the shared workspace before installing this wheel.
    root = Path(__file__).resolve().parents[5]
    native_environment = root / ".venv"
    native_python = native_environment / (
        "Scripts/python.exe" if os.name == "nt" else "bin/python"
    )
    _run(
        [
            "uv",
            "pip",
            "install",
            "--python",
            str(native_python),
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
            raise SystemExit(
                f"No dokime wheel artifact exists in {wheel_dir}."
            ) from error
        raise SystemExit(
            f"Expected one dokime wheel in {wheel_dir}, found: {error}."
        ) from error


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
    parser.add_argument(
        "--wheel-dir",
        default=os.environ.get(WHEEL_DIR_ENV, DEFAULT_WHEEL_DIR),
        help="Directory containing the exact release wheel used for Python tests.",
    )
    return parser.parse_args(argv)


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
