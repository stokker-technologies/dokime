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
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.package.staging import clear_output, stage_file
from dokime_tools.test import archive

# Type Checking Imports


# === Public Exports === #

# === Constants === #

EDITABLES = "editables==0.5 --hash=sha256:61e5ffa82629e0d8bfe09bc44a07db3c1ab8ed1ce78a6980732870f19b5e7d4c"

# === Types === #

# === Classes === #

# === Public Functions === #


def cargo(root: Path, producer: dict) -> dict:
    """Export The Locked Source Closure Without Compiling Another Target."""
    root = root.resolve()
    directory = root / "target/dependencies"
    vendor = directory / "vendor"
    config = directory / "cargo-config.toml"
    clear_output(vendor)
    clear_output(config)
    return archive.native(
        root,
        producer["profile"],
        "dependency-export",
        [["cargo", "vendor", "--locked", "--versioned-dirs", str(vendor)]],
        [vendor, config],
        locks=[root / "Cargo.lock"],
        parents=[producer],
        stdout={0: config},
    )


def wheelhouse(root: Path, producer: dict) -> dict:
    """Export The Starter's Actual Locked Backend And Tool Wheels."""
    root = root.resolve()
    directory = root / "target/dependencies"
    tools = directory / "starter-tools"
    templates = root / "crates/cargo-dokime/templates/tools/python"
    for name in ("pyproject.toml", "uv.lock"):
        source = templates / f"{name}.tmpl"
        destination = tools / name
        stage_file(source, destination)
    requirements = directory / "starter-requirements.txt"
    wheels = directory / "wheels"
    clear_output(requirements)
    clear_output(wheels)
    bundled = list((Path(ensurepip.__file__).parent / "_bundled").glob("pip-*.whl"))
    if len(bundled) != 1:
        raise ValueError("the selected Python must supply one bundled ensurepip wheel")
    loader = "import runpy,sys; sys.path.insert(0,sys.argv.pop(1)); runpy.run_module('pip',run_name='__main__')"
    commands = [
        ["uv", "export", "--locked", "--all-groups", "--no-emit-project", "--project", str(tools)],
        [
            sys.executable,
            "-c",
            loader,
            str(bundled[0]),
            "download",
            "--dest",
            str(wheels),
            "--require-hashes",
            "--only-binary=:all:",
            "--requirement",
            str(requirements),
        ],
    ]
    # Hatchling Requests `editables` Only For Editable Installs, So No Lock Names It.
    editables = root / "target/tmp/starter-editables.txt"
    editables.parent.mkdir(parents=True, exist_ok=True)
    editables.write_text(f"{EDITABLES}\n", encoding="utf-8")
    commands.append([*commands[1][:-2], "--no-deps", "--requirement", str(editables)])
    return archive.native(
        root,
        producer["profile"],
        "dependency-export",
        commands,
        [requirements, wheels],
        locks=[tools / "pyproject.toml", tools / "uv.lock"],
        inputs=[bundled[0], editables],
        parents=[producer],
        stdout={0: requirements},
    )


# === Private Functions === #

# === Entry Point === #
