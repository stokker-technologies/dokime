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
from collections.abc import Callable
from pathlib import Path
from zipfile import ZipFile

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_pure_wheel(artifact: Callable[[str], Path]) -> None:
    wheel_dir = artifact("python")
    wheels = list(wheel_dir.glob("dokime-*.whl"))
    assert len(wheels) == 1, "prepare exactly one wheel before acceptance"
    assert wheels[0].name.endswith("-py3-none-any.whl")
    distribution = importlib.metadata.distribution("dokime")
    with ZipFile(wheels[0]) as archive:
        names = archive.namelist()
        assert "dokime/client/client.py" in names
        assert "dokime/py.typed" in names
        assert "dokime/__init__.pyi" in names
        assert "dokime/operation.py" in names
        assert "dokime/dynamic.py" in names
        assert "dokime/contracts.json" in names
        assert "dokime/descriptors.bin" in names
        assert any(
            name.startswith("dokime/_wire/") and name.endswith(".pyi") for name in names
        )
        assert not any(name.startswith("dokime/client/_generated/") for name in names)
        assert not any(
            name.endswith((".so", ".pyd", ".dll", ".dylib")) for name in names
        )
        assert "dokime/client/cli.py" not in names
        assert "dokime/client/_console.py" not in names
        assert not any(name.startswith("dokime_client/") for name in names)
        for name in names:
            if not name.startswith("dokime/") or name.endswith("/"):
                continue
            installed = Path(str(distribution.locate_file(name)))
            assert installed.is_file(), f"Missing installed wheel payload: {name}"
            with archive.open(name) as expected, installed.open("rb") as actual:
                expected_digest = hashlib.sha256()
                for block in iter(lambda: expected.read(1024 * 1024), b""):
                    expected_digest.update(block)
                assert (
                    hashlib.file_digest(actual, "sha256").digest()
                    == expected_digest.digest()
                ), f"Installed payload differs from selected wheel: {name}"

    assert not any(
        requirement.startswith(("dokime-client", "dokime-python", "rich", "typer"))
        for requirement in distribution.requires or []
    )
    assert importlib.util.find_spec("dokime_client") is None
    assert any(
        requirement.startswith("grpcio") for requirement in distribution.requires or []
    )
    assert any(
        requirement.startswith("protobuf")
        for requirement in distribution.requires or []
    )
    assert not any(
        entry.group == "console_scripts" for entry in distribution.entry_points
    )
    subprocess.run(
        [
            sys.executable,
            "-I",
            "-c",
            "import dokime; from dokime.client import Capture, Cycle, DokimeClient, RunRequest; assert DokimeClient()._transport.backend is None",
        ],
        cwd=wheel_dir,
        check=True,
        capture_output=True,
        text=True,
    )


# === Private Functions === #

# === Entry Point === #
