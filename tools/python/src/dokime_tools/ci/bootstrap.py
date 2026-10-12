# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare Native CI Platform Prerequisites"""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
import argparse
import os
import platform
import shutil
import subprocess
import sys
from collections.abc import Callable, Sequence
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Debian Build And Installer Prerequisites
GUI_MARKER_PACKAGE = "libwebkit2gtk-4.1-dev"
GUI_PACKAGES = (
    "git",
    "xz-utils",
    "unzip",
    "build-essential",
    "pkg-config",
    "libssl-dev",
    "patchelf",
    "rpm",
    "file",
    GUI_MARKER_PACKAGE,
    "libgtk-3-dev",
    "libsoup-3.0-dev",
    "libjavascriptcoregtk-4.1-dev",
    "librsvg2-dev",
)
VC_TOOLSET_COMPONENT = "Microsoft.VisualStudio.Component.VC.Tools.x86.x64"
VSWHERE_RELATIVE_PATH = Path("Microsoft Visual Studio/Installer/vswhere.exe")

# === Types === #

# === Classes === #

# === Public Functions === #


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parse_arguments(argv)
    system = platform.system()
    _step(f"host system is {system}")
    _create_temporary_directory()
    for step in platform_steps(system, gui_packages=arguments.gui_packages):
        step()
    for task in arguments.tasks:
        _run_mise_task(task)
    _step("bootstrap complete")
    return 0


def parse_arguments(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="bootstrap.py",
        description="Prepare a CI runner and run the requested mise sync tasks.",
    )
    parser.add_argument(
        "--gui-packages",
        action="store_true",
        help="install the Debian GUI development packages (Linux jobs only).",
    )
    parser.add_argument(
        "tasks",
        nargs="*",
        help="mise tasks to run once the host is prepared, in order.",
    )
    return parser.parse_args(argv)


def platform_steps(system: str, *, gui_packages: bool) -> list[Callable[[], None]]:
    """Name the host-specific steps for a system, in the order they run."""
    if system == "Linux":
        return [install_gui_packages] if gui_packages else []
    if system == "Windows":
        # The GUI packages have no Windows counterpart; the MSVC toolset does,
        # and every Rust job needs it.
        return [verify_visual_cpp_toolset]
    raise SystemExit(f"bootstrap.py supports Linux and Windows CI hosts, not {system}.")


def install_gui_packages() -> None:
    """Install the Debian GUI development packages when they are missing.

    A CI host that is missing them and cannot install them is broken, and the
    linker error it would eventually produce says nothing useful, so only a
    host with no dpkg at all -- a developer machine, never the CI image --
    gets to skip quietly.
    """
    if shutil.which("dpkg") is None:
        _step("skipping GUI packages: no dpkg here, so this is not a Debian host")
        return
    if all(_package_installed(package) for package in GUI_PACKAGES):
        _step("skipping GUI packages: all prerequisites are installed")
        return
    prefix = []
    if (
        os.geteuid() != 0
        and os.environ.get("GITHUB_ACTIONS") == "true"
        and shutil.which("sudo")
    ):
        prefix = ["sudo", "-n"]
    elif os.geteuid() != 0:
        raise SystemExit(
            f"{GUI_MARKER_PACKAGE} is missing and apt-get needs root; install the "
            "GUI development packages by hand or run this as root."
        )
    _run([*prefix, "apt-get", "update"])
    _run(
        [*prefix, "apt-get", "install", "-y", "--no-install-recommends", *GUI_PACKAGES]
    )


def verify_visual_cpp_toolset() -> None:
    """Fail early when the MSVC toolset rustc links through is not installed.

    The Windows setup script initializes the developer environment before
    installing Cargo-based tools, then persists linker inputs for later jobs.
    This check verifies that the requested native toolset is installed.
    """
    program_files = os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)")
    vswhere = Path(program_files) / VSWHERE_RELATIVE_PATH
    if not vswhere.is_file():
        raise SystemExit(
            f"vswhere.exe not found at '{vswhere}'; install Visual Studio with the "
            "'Desktop development with C++' workload on this runner."
        )
    _step(f"resolving the MSVC toolset through {vswhere}")
    completed = subprocess.run(
        [
            str(vswhere),
            "-latest",
            "-products",
            "*",
            "-requires",
            VC_TOOLSET_COMPONENT,
            "-property",
            "installationPath",
        ],
        capture_output=True,
        check=True,
        text=True,
    )
    installation = completed.stdout.strip()
    if not installation:
        raise SystemExit(
            f"no Visual Studio instance provides {VC_TOOLSET_COMPONENT}; install the "
            "'Desktop development with C++' workload on this runner."
        )
    _step(f"MSVC toolset found at {installation}")


# === Private Functions === #


def _create_temporary_directory() -> None:
    temporary = os.environ.get("TMPDIR")
    if not temporary:
        _step("skipping temporary directory: TMPDIR is unset")
        return
    _step(f"creating {temporary}")
    Path(temporary).mkdir(parents=True, exist_ok=True)


def _package_installed(name: str) -> bool:
    return (
        subprocess.run(["dpkg", "-s", name], capture_output=True, text=True).returncode
        == 0
    )


def _run_mise_task(task: str) -> None:
    _run(["mise", "run", task])


def _run(command: Sequence[str]) -> None:
    _step(" ".join(command))
    subprocess.run(command, check=True)


def _step(message: str) -> None:
    print(f"[ci:bootstrap] {message}", flush=True)


# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
