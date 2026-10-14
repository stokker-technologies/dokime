# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Stamp Prepared Fixtures With The Contracts That Built Them"""

# === Imports === #

# Standard Library Imports
import hashlib
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools import fixtures

# Type Checking Imports


# === Public Exports === #

# === Constants === #
REVISION = "becc491d56a4e7be98ed228ecd5f7cb6c06bf900"

# === Types === #

# === Classes === #

# === Public Functions === #


def test_fixture_stamp(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("GITHUB_SHA", REVISION)
    generated = tmp_path / "packages/dokime/src/dokime"
    generated.mkdir(parents=True)
    (generated / "contracts.json").write_bytes(b'{"contracts":[]}')
    (generated / "descriptors.bin").write_bytes(b"descriptors")
    expected = hashlib.sha256(b'{"contracts":[]}descriptors').hexdigest()

    path = fixtures.write_stamp(tmp_path / "target/fixtures", tmp_path)

    assert path == tmp_path / "target/fixtures" / fixtures.STAMP
    assert path.read_text(encoding="ascii") == (
        f"revision {REVISION}\ncontracts {expected}\n"
    )
    (generated / "descriptors.bin").write_bytes(b"changed descriptors")
    assert fixtures.stamp(tmp_path) != path.read_text(encoding="ascii")


# === Private Functions === #

# === Entry Point === #
