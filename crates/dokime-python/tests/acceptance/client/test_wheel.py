# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify the installed unified wheel and its remote CLI."""

# === Imports === #

# Standard Library Imports
import hashlib
import importlib.metadata
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
from zipfile import ZipFile

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_unified_cli() -> None:
    root = Path(__file__).resolve().parents[5]
    wheel_dir = Path(
        os.environ.get("DOKIME_PYTHON_TEST_WHEEL_DIR", root / "dist/python-test")
    )
    wheels = list(wheel_dir.glob("dokime-*.whl"))
    assert len(wheels) == 1, "prepare exactly one native wheel before acceptance"
    distribution = importlib.metadata.distribution("dokime")
    with ZipFile(wheels[0]) as archive:
        names = archive.namelist()
        assert "dokime/client/cli.py" in names
        assert "dokime/core/__init__.py" in names
        assert "dokime/py.typed" in names
        assert "dokime/client/_generated/dokime/v1/execution_pb2.pyi" in names
        assert any(
            name.startswith("dokime/_native") and name.endswith((".so", ".pyd"))
            for name in names
        )
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
        requirement.startswith(("dokime-client", "dokime-python", "grpcio"))
        for requirement in distribution.requires or []
    )
    assert importlib.util.find_spec("dokime_client") is None
    subprocess.run(
        [
            sys.executable,
            "-I",
            "-c",
            "import sys; import dokime; from dokime.client import Capture, Cycle, DokimeClient, RunRequest; assert DokimeClient()._transport.backend is None; assert 'grpc' not in sys.modules",
        ],
        cwd=wheel_dir,
        check=True,
        capture_output=True,
        text=True,
    )
    executable = Path(sys.executable).parent / (
        "dokime-client.exe" if os.name == "nt" else "dokime-client"
    )
    result = subprocess.run(
        [str(executable), "run", "--help"], check=True, capture_output=True, text=True
    )
    assert "--parameters-file" in result.stdout
    assert "--report" in result.stdout


# === Private Functions === #

# === Entry Point === #
