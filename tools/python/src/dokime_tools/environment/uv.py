# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run uv With The Repository Environment And Native Python Linker Inputs."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

REPO_ROOT = Path(__file__).resolve().parents[5]

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> int:
    env = os.environ.copy()
    _configure_root_venv(env)
    _configure_windows_python_lib(env)
    return subprocess.call(["uv", *sys.argv[1:]], env=env)


# === Private Functions === #


def _configure_root_venv(env: dict[str, str]) -> None:
    env["UV_PROJECT_ENVIRONMENT"] = str(REPO_ROOT / ".venv")


def _configure_windows_python_lib(env: dict[str, str]) -> None:
    if os.name != "nt":
        return

    lib_dir = _python_lib_dir(env)
    if lib_dir is None:
        return

    existing = env.get("LIB", "")
    lib_entry = str(lib_dir)
    entries = [entry for entry in existing.split(";") if entry]
    if lib_entry not in entries:
        env["LIB"] = ";".join([lib_entry, *entries])


def _python_lib_dir(env: dict[str, str]) -> Path | None:
    python = _project_python(env)
    if python is None:
        python = _uv_python(env)
    if python is None:
        return None

    command = [
        str(python),
        "-c",
        "import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')",
    ]
    result = subprocess.run(
        command, check=False, capture_output=True, env=env, text=True, encoding="utf-8"
    )
    if result.returncode != 0:
        return None

    value = result.stdout.strip()
    if not value:
        return None

    lib_dir = Path(value)
    return lib_dir if lib_dir.exists() else None


def _project_python(env: dict[str, str]) -> Path | None:
    project_environment = Path(env["UV_PROJECT_ENVIRONMENT"])
    scripts_dir = "Scripts" if os.name == "nt" else "bin"
    executable = "python.exe" if os.name == "nt" else "python"
    python = project_environment / scripts_dir / executable
    return python if python.exists() else None


def _uv_python(env: dict[str, str]) -> Path | None:
    command = ["uv", "python", "find"]
    if uv_python := env.get("UV_PYTHON"):
        command.append(uv_python)

    result = subprocess.run(
        command, check=False, capture_output=True, env=env, text=True, encoding="utf-8"
    )
    if result.returncode != 0:
        return None

    python = Path(result.stdout.strip())
    return python if python.exists() else None


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
