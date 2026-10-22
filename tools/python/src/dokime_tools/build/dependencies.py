# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Capture Offline Inputs For The Generated Project Acceptance Story"""

# === Imports === #

# Standard Library Imports
import ensurepip
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def cargo(root: Path, directory: Path) -> list[Path]:
    """Export The Locked Source Closure Without Compiling Another Target."""
    root = root.resolve()
    vendor = directory / "vendor"
    config = directory / "cargo-config.toml"
    clear_output(vendor)
    clear_output(config)
    with config.open("w", encoding="utf-8") as output:
        subprocess.run(["cargo", "vendor", "--locked", "--versioned-dirs", str(vendor)], cwd=root, stdout=output, check=True)
    # The Configuration Names The Closure; Its Thousands Of Vendored Files Stay Out Of The Stamp.
    return [config]


def wheelhouse(root: Path, directory: Path) -> list[Path]:
    """Export The Starter's Actual Locked Backend And Tool Wheels."""
    root = root.resolve()
    tools = directory / "starter-tools"
    templates = root / "crates/cargo-dokime/templates"
    clear_output(tools)
    for relative in ("pyproject.toml", "uv.lock", "tools/python/pyproject.toml"):
        stage_file(templates / f"{relative}.tmpl", tools / relative)
    requirements = directory / "starter-requirements.txt"
    wheels = directory / "wheels"
    clear_output(requirements)
    clear_output(wheels)
    bundled = list((Path(ensurepip.__file__).parent / "_bundled").glob("pip-*.whl"))
    if len(bundled) != 1:
        raise ValueError("the selected Python must supply one bundled ensurepip wheel")
    with requirements.open("w", encoding="utf-8") as output:
        subprocess.run(
            ["uv", "export", "--locked", "--all-groups", "--no-emit-project", "--project", str(tools / "tools/python")],
            cwd=root,
            stdout=output,
            check=True,
        )
    loader = "import runpy,sys; sys.path.insert(0,sys.argv.pop(1)); runpy.run_module('pip',run_name='__main__')"
    download = [sys.executable, "-c", loader, str(bundled[0]), "download", "--dest", str(wheels), "--require-hashes", "--only-binary=:all:"]
    subprocess.run([*download, "--requirement", str(requirements)], cwd=root, check=True)
    # Hatchling Requests `editables` Only For Editable Installs, So No Lock Names It; The Hash-Checked
    # Requirement Is Tracked So Every Lane Runner Installs The Same Wheel.
    subprocess.run([*download, "--no-deps", "--requirement", str(root / "tools/python/editables.txt")], cwd=root, check=True)
    return [requirements, *sorted(wheels.glob("*.whl"))]


# === Private Functions === #

# === Entry Point === #
