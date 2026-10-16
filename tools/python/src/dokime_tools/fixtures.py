# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compile Native Test Fixtures With The Packaged Tool"""

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
from pathlib import Path

# Third Party Imports

# Local Imports
# isort: split
from .bindings import fingerprint

# Type Checking Imports


# === Public Exports === #

# === Constants === #
ROOT = Path(__file__).resolve().parents[4]
# The Test Supports Refuse Bundles Whose Stamp Disagrees With This Tree.
STAMP = "fixtures.stamp"
FIXTURES = (
    ("mini-sidecar", "mini-sidecar", None),
    ("dokime-route-plugin", "dokime-route-plugin", "dokime-route-plugin"),
    ("fixture-sidecar-protocol", "dokime-route-plugin", "fixture-sidecar-protocol"),
    ("fixture-suites", "dokime-route-plugin", "dokime-app-suite-sidecar-fixture"),
    (
        "fixture-telemetry",
        "dokime-route-plugin",
        "dokime-app-telemetry-sidecar-fixture",
    ),
    (
        "fixture-dependent",
        "dokime-route-plugin",
        "dokime-app-dependent-sidecar-fixture",
    ),
)


# === Types === #

# === Classes === #

# === Public Functions === #


def prepare(
    *, tool: Path | None = None, output: Path = Path("target/fixtures")
) -> None:
    profile = os.environ.get("STKR_TEST_PROFILE", "dev")
    if profile not in {"dev", "release"}:
        raise ValueError("fixture profile must be dev or release")
    directory = "debug" if profile == "dev" else "release"
    binary_name = "cargo-dokime.exe" if os.name == "nt" else "cargo-dokime"
    tool = tool or Path(
        os.environ.get("DOKIME_TEST_TOOL", ROOT / "target" / directory / binary_name)
    )
    if not tool.is_file():
        raise ValueError(f"run mise run build:cli:{profile} before preparing fixtures")
    for name, package, binary in FIXTURES:
        bundle = output / name / f"{name}.dokime"
        command = [
            str(tool),
            "dokime",
            "build",
            "--package",
            package,
            "--profile",
            profile,
            "--output",
            str(bundle),
        ]
        if binary:
            command += ["--bin", binary]
        subprocess.run(command, check=True)
        if not bundle.is_file() or bundle.is_symlink():
            raise ValueError(f"missing native test fixture: {bundle}")
    write_stamp(output)


def stamp(root: Path = ROOT) -> str:
    """The provenance line pair identifying bundles compiled from this tree."""
    revision = (
        os.environ.get("GITHUB_SHA")
        or subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=root, text=True
        ).strip()
    )
    return f"revision {revision}\ncontracts {fingerprint(root)}\n"


def write_stamp(directory: Path, root: Path = ROOT) -> Path:
    """Record beside freshly compiled bundles which tree they belong to."""
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / STAMP
    path.write_text(stamp(root), encoding="ascii")
    return path


# === Private Functions === #

# === Entry Point === #
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tool", type=Path)
    parser.add_argument("--output", type=Path, default=Path("target/fixtures"))
    parser.add_argument(
        "--stamp-only",
        action="store_true",
        help="stamp bundles another task compiled into --output",
    )
    options = parser.parse_args()
    if options.stamp_only:
        write_stamp(options.output)
    else:
        prepare(tool=options.tool, output=options.output)
