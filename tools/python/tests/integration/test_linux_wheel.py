# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Portable Wheel Build Inputs And Provenance"""

# === Imports === #

# Standard Library Imports
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.package import linux

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize(
    "field",
    [
        None,
        "image",
        "wheel_sha256",
        "sdist_sha256",
        "python",
        "uv",
        "maturin",
        "rustc",
        "packages",
    ],
)
def test_build_provenance(tmp_path: Path, field: str | None) -> None:
    (tmp_path / "mise.toml").write_text('[tools]\npython="3.13.15"\nuv="0.11.33"\n')
    _backend(tmp_path)
    receipt = {
        "version": 1,
        "image": linux.IMAGE,
        "compatibility": "manylinux_2_28",
        "wheel_sha256": "wheel",
        "sdist_sha256": "sdist",
        "python": "Python 3.13.15",
        "uv": "uv 0.11.33",
        "maturin": "maturin 1.15.0",
        "rustc": "rustc pinned",
        "packages": list(linux.PACKAGES),
    }
    if field is None:
        linux.verify(receipt, "wheel", "sdist", "rustc pinned", tmp_path)
    else:
        receipt[field] = "different"
        with pytest.raises(ValueError, match="provenance"):
            linux.verify(receipt, "wheel", "sdist", "rustc pinned", tmp_path)


def test_build_mounts(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(linux.os, "getuid", lambda: 1000, raising=False)
    monkeypatch.setattr(linux.os, "getgid", lambda: 1000, raising=False)
    (tmp_path / "mise.toml").write_text(
        '[tools]\npython="3.13.15"\nuv="0.11.33"\nrust={version="1.99.0"}\n'
    )
    _backend(tmp_path)
    cargo = tmp_path / "cargo"
    for name in ("git", "registry"):
        (cargo / name).mkdir(parents=True)
    monkeypatch.setenv("CARGO_HOME", str(cargo))
    archive = tmp_path / "dokime-0.1.0.tar.gz"
    archive.write_bytes(b"prepared archive")
    output = tmp_path / "dist"
    output.mkdir()
    with (
        patch.object(linux.platform, "machine", return_value="x86_64"),
        patch.object(linux.shutil, "which", return_value="/usr/bin/uv"),
        patch.object(linux, "_output", return_value=str(tmp_path / "rust")),
        patch.object(linux.subprocess, "run") as run,
    ):
        linux.build(archive, output, tmp_path)
    command = run.call_args.args[0]
    assert linux.IMAGE in command
    assert "linux/amd64" in command
    mounts = [
        command[index + 1] for index, value in enumerate(command) if value == "--mount"
    ]
    assert f"type=bind,source={archive},target=/input/{archive.name},readonly" in mounts
    assert (
        f"type=bind,source={cargo / 'registry'},target=/cache/cargo/registry,readonly"
        in mounts
    )
    assert (
        f"type=bind,source={cargo / 'git'},target=/cache/cargo/git,readonly" in mounts
    )
    assert not any("target/release" in mount or ".git," in mount for mount in mounts)
    assert command[-4:] == ["3.13.15", "0.11.33", "1.99.0", "maturin==1.15.0"]


# === Private Functions === #


def _backend(root: Path) -> None:
    sdk = root / "crates/dokime-python"
    sdk.mkdir(parents=True)
    (sdk / "pyproject.toml").write_text(
        '[build-system]\nrequires=["maturin==1.15.0"]\n'
    )


# === Entry Point === #
