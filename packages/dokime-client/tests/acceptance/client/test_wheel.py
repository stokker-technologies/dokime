# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify the exact installed pure Python wheel."""

# === Imports === #

# Standard Library Imports
import hashlib
import importlib.metadata
import importlib.util
import subprocess
import sys
from pathlib import Path
from zipfile import ZipFile

# Third Party Imports
# Local Imports
from dokime_tools.build import paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_pure_wheel(workspace: Path, request) -> None:
    # The build writes the wheel where every Python lane installs it from.
    wheel_dir = paths.output(workspace, "build", request.config.getoption("--build-profile"), "any") / "python"
    wheels = list(wheel_dir.glob("dokime_client-*.whl"))
    assert len(wheels) == 1, "build exactly one wheel before acceptance"
    assert wheels[0].name.endswith("-py3-none-any.whl")
    distribution = importlib.metadata.distribution("dokime-client")
    with ZipFile(wheels[0]) as archive:
        names = archive.namelist()
        assert "dokime_client/client/client.py" in names
        assert "dokime_client/py.typed" in names
        assert "dokime_client/__init__.pyi" in names
        assert "dokime_client/operation.py" in names
        assert "dokime_client/dynamic.py" in names
        assert "dokime_client/contracts.json" in names
        assert "dokime_client/client-settings.json" not in names
        assert "dokime_client/descriptors.bin" in names
        assert any(name.startswith("dokime_client/_wire/") and name.endswith(".pyi") for name in names)
        assert not any(name.startswith("dokime_client/client/_generated/") for name in names)
        assert not any(name.endswith((".so", ".pyd", ".dll", ".dylib")) for name in names)
        assert "dokime_client/client/cli.py" not in names
        assert "dokime_client/client/_console.py" not in names
        assert not any(name.startswith("dokime/") for name in names)
        for name in names:
            if not name.startswith("dokime_client/") or name.endswith("/"):
                continue
            installed = Path(str(distribution.locate_file(name)))
            assert installed.is_file(), f"Missing installed wheel payload: {name}"
            with archive.open(name) as expected, installed.open("rb") as actual:
                expected_digest = hashlib.sha256()
                for block in iter(lambda: expected.read(1024 * 1024), b""):
                    expected_digest.update(block)
                assert hashlib.file_digest(actual, "sha256").digest() == expected_digest.digest(), (
                    f"Installed payload differs from selected wheel: {name}"
                )

    assert not any(
        requirement.startswith(("dokime-client", "dokime-python", "dokime ", "rich", "typer"))
        for requirement in distribution.requires or []
    )
    assert importlib.util.find_spec("dokime") is None
    assert any(requirement.startswith("grpcio") for requirement in distribution.requires or [])
    assert any(requirement.startswith("protobuf") for requirement in distribution.requires or [])
    assert not any(entry.group == "console_scripts" for entry in distribution.entry_points)
    subprocess.run(
        [
            sys.executable,
            "-I",
            "-c",
            "import dokime_client; from dokime_client.client import Cycle, DokimeClient, Execution, Template, TestCase; assert DokimeClient()._transport.backend is None",
        ],
        cwd=wheel_dir,
        check=True,
        capture_output=True,
        text=True,
    )


# === Private Functions === #

# === Entry Point === #
