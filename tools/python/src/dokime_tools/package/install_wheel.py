# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Install exactly one wheel from a distribution directory."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations

import argparse
import os
import subprocess
from collections.abc import Sequence
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.package.wheel import WheelSelectionError as WheelSelectionError
from dokime_tools.package.wheel import select_wheel as select_wheel

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main(argv: Sequence[str] | None = None) -> int:
    args = _parse_args(argv)
    wheel = _resolve_wheel(Path(args.dist_dir), args.pattern)

    command = [
        "uv",
        "pip",
        "install",
        "--python",
        str(
            Path(__file__).resolve().parents[5]
            / ".venv"
            / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        ),
        "--reinstall",
    ]
    if args.no_deps:
        command.append("--no-deps")
    command.append(str(wheel))
    subprocess.run(command, check=True)
    return 0


# === Private Functions === #


def _parse_args(argv: Sequence[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dist_dir", help="Directory containing the built wheel.")
    parser.add_argument(
        "pattern", help="Wheel glob pattern, for example package-*.whl."
    )
    parser.add_argument(
        "--no-deps",
        action="store_true",
        help="Install the wheel without resolving dependencies.",
    )
    return parser.parse_args(argv)


def _resolve_wheel(dist_dir: Path, pattern: str) -> Path:
    try:
        return select_wheel(dist_dir, pattern)
    except WheelSelectionError as error:
        if not error.wheels:
            raise SystemExit(
                f"No wheel matching {pattern!r} exists in {dist_dir}."
            ) from error
        raise SystemExit(f"Multiple wheels match {pattern!r}: {error}.") from error


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
