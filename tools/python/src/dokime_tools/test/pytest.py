# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Pytest Commands for Python Lanes, Coverage and Benchmarks.\n\na Python project keeps its stories under `tests/<category>/` and its benchmarks under\n`benchmarks/<category>/`. Lanes run in the shared root `.venv/` without syncing it: a\nfocused SDK lane builds and installs the wheel first, and a prepared schedule installs the\nstaged wheel before its lanes start. Lane arguments after `--` are appended to pytest.\n"

# === Imports === #

# Standard Library Imports
import os
import sys
from collections.abc import Sequence
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from . import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

STRICT = ["--strict-config", "--strict-markers", "--import-mode=importlib"]

# === Types === #

# === Classes === #

# === Public Functions === #


def environment(directory: Path) -> dict[str, str]:
    """The shared root environment with no ambient selection that could narrow a lane."""
    variables = dict(os.environ)
    variables.pop("UV_PROJECT_ENVIRONMENT", None)
    variables.pop("PYTEST_ADDOPTS", None)
    variables.pop("COVERAGE_PROCESS_START", None)
    return variables


def run(
    project: Path,
    suite: str,
    deadline: int,
    junit: Path,
    arguments: Sequence[str],
) -> list[str]:
    """One pytest lane with a terminating per-story hang guard."""
    if not (project / suite).is_dir():
        raise policy.TestError(f"Run Pytest Tests: missing Python suite {project / suite}")
    # Integration Fixtures Own Ephemeral Sockets And Worker-Local Temporary Roots.
    parallel = []
    if suite == "tests/integration" and not any(
        value in ("-n", "--numprocesses") or value.startswith(("-n", "--numprocesses=")) for value in arguments
    ):
        parallel = ["-n", "auto", "--dist=worksteal", "--max-worker-restart=0"]
    return [
        sys.executable,
        "-m",
        "pytest",
        *STRICT,
        "--timeout-method=thread",
        f"--timeout={deadline}",
        f"--junitxml={junit}",
        suite,
        *parallel,
        *arguments,
    ]


def coverage(project: Path, cobertura: Path) -> list[str]:
    """Coverage arguments whose measured sources the project's own coverage configuration declares.

    The project's `pyproject.toml` names what is measured and maps installed packages back to
    their source tree, so tests that import an installed wheel still report authored modules.
    """
    configuration = project / "pyproject.toml"
    if not configuration.is_file():
        raise policy.TestError(f"Run Pytest Tests: missing coverage configuration {configuration}")
    return [
        "--cov",
        f"--cov-config={configuration}",
        "--cov-report=term",
        f"--cov-report=xml:{cobertura}",
    ]


def bench(project: Path, category: str, output: Path, arguments: Sequence[str]) -> list[str]:
    suite = f"benchmarks/{category}"
    if not (project / suite).is_dir():
        raise policy.TestError(f"Run Pytest Tests: missing Python benchmarks {project / suite}")
    return [
        sys.executable,
        "-m",
        "pytest",
        *STRICT,
        "--benchmark-only",
        f"--benchmark-json={output}",
        suite,
        *arguments,
    ]


# === Private Functions === #


# === Entry Point === #
