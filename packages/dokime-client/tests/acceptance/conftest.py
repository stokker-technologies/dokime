# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Locate Built Product Artifacts From The Workspace Root"""

# === Imports === #

# Standard Library Imports
import os
from collections.abc import Callable
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.build import paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption("--build-profile", choices=("dev", "release"), default="dev")


@pytest.fixture(scope="session")
def workspace() -> Path:
    """The Workspace Root Is The Nearest Ancestor Holding Cargo.lock"""
    for directory in Path(__file__).resolve().parents:
        if (directory / "Cargo.lock").is_file():
            return directory
    raise AssertionError("no Cargo.lock above the acceptance stories")


@pytest.fixture(scope="session")
def artifact(workspace: Path, request: pytest.FixtureRequest) -> Callable[[str], Path]:
    """Resolve A Matching Profile/Target Build Artifact, Failing When Absent"""
    build = paths.output(workspace, "build", request.config.getoption("--build-profile"))

    def resolve(relative: str) -> Path:
        path = build / relative
        if not path.exists():
            raise AssertionError(f"missing built artifact {path}; run `mise run build`")
        return path

    return resolve


@pytest.fixture(scope="session")
def host(artifact: Callable[[str], Path]) -> Path:
    """The Built dokime Executable Hosting The Server"""
    suffix = ".exe" if os.name == "nt" else ""
    return artifact(f"bin/dokime{suffix}")


# === Private Functions === #

# === Entry Point === #
