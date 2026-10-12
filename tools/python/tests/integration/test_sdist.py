# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Reduced Workspace Source Archives Remain Locked And Preserve Sources"""

# === Imports === #

# Standard Library Imports
from pathlib import Path
import subprocess
import tarfile
from unittest.mock import patch

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.package import sdist
from dokime_tools.check.package import _verify_sdist_build_inputs

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("change", ["valid", "missing_brand", "stale_brand"])
def test_sdist_locked(tmp_path: Path, change: str) -> None:
    archive = _archive(tmp_path)
    source_root = tmp_path / "sdk-0.1.0"
    bundle = source_root / "branding"
    for folder in ("theme", "fonts", "logos"):
        (bundle / folder).mkdir(parents=True)
        (bundle / folder / "asset").write_bytes(folder.encode())
    (bundle / "brand.toml").write_text('name = "Dokime"\n')
    sdist.prepare(archive, source_root)
    extracted = tmp_path / "extracted"
    extracted.mkdir()
    with tarfile.open(archive) as contents:
        contents.extractall(extracted, filter="data")
    package = extracted / "sdk-0.1.0"
    subprocess.run(
        ["cargo", "metadata", "--offline", "--locked", "--format-version", "1"],
        cwd=package,
        stdout=subprocess.DEVNULL,
        check=True,
    )
    assert 'name = "unused"' not in (package / "Cargo.lock").read_text()
    for path in bundle.rglob("*"):
        if path.is_file():
            assert (
                package / "branding" / path.relative_to(bundle)
            ).read_bytes() == path.read_bytes()
    assert (package / "sdk/src/lib.rs").read_text() == "pub fn answer() -> u32 { 42 }\n"
    if change == "valid":
        _verify_sdist_build_inputs(archive, source_root)
    elif change == "missing_brand":
        (bundle / "brand.toml").unlink()
        with pytest.raises(SystemExit, match="Missing source build input"):
            _verify_sdist_build_inputs(archive, source_root)
    else:
        (bundle / "theme/asset").write_bytes(b"new branding")
        with pytest.raises(SystemExit, match="Stale source distribution build input"):
            _verify_sdist_build_inputs(archive, source_root)


def test_sdist_drift(tmp_path: Path) -> None:
    archive = _archive(tmp_path)
    original = archive.read_bytes()

    def change_resolution(*args, cwd, **kwargs):
        lock = cwd / "Cargo.lock"
        lock.write_text(
            lock.read_text().replace('version = "0.1.0"', 'version = "0.2.0"')
        )

    with patch.object(sdist.subprocess, "run", side_effect=change_resolution):
        with pytest.raises(ValueError, match="locked version or checksum"):
            sdist.prepare(archive)
    assert archive.read_bytes() == original


# === Private Functions === #


def _archive(root: Path) -> Path:
    package = root / "sdk-0.1.0"
    package.mkdir()
    manifest = package / "Cargo.toml"
    manifest.write_text('[workspace]\nmembers = ["sdk", "unused"]\nresolver = "2"\n')
    for name in ("sdk", "unused"):
        directory = package / name
        (directory / "src").mkdir(parents=True)
        (directory / "Cargo.toml").write_text(
            f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2024"\n'
        )
        (directory / "src/lib.rs").write_text("pub fn answer() -> u32 { 42 }\n")
    subprocess.run(
        ["cargo", "generate-lockfile", "--offline"],
        cwd=package,
        check=True,
    )
    manifest.write_text('[workspace]\nmembers = ["sdk"]\nresolver = "2"\n')
    for relative in (
        "crates/dokime-protocol/proto/dokime/v1/document.proto",
        "crates/dokime-app/dist-report/report.html",
        "crates/dokime-python/python/dokime/__init__.py",
    ):
        path = package / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("build input fixture")
    archive = root / "sdk-0.1.0.tar.gz"
    with tarfile.open(archive, "w:gz") as contents:
        contents.add(package, arcname=package.name)
        contents.add(
            package / "crates/dokime-python/python", arcname=package.name + "/python"
        )
    return archive


# === Entry Point === #
