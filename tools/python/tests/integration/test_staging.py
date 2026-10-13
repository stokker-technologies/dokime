# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Preserve Safe Frontend Staging"""

# === Imports === #

# Standard Library Imports
import os
import stat
from collections.abc import Callable
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.staging import stage_file, sync_output

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def case_staging_projection(tmp_path: Path) -> None:
    source = tmp_path / "source"
    destination = tmp_path / "destination"
    for root in [source, destination]:
        (root / "assets").mkdir(parents=True)
        (root / "index.html").write_text("same")
        (root / "assets/app.js").write_text("app")
    before = {
        p: p.stat()
        for p in [destination, destination / "assets", destination / "index.html"]
    }
    sync_output(source, destination)
    for path, metadata in before.items():
        assert path.stat().st_ino == metadata.st_ino
        assert path.stat().st_mtime_ns == metadata.st_mtime_ns
    (destination / "stale.js").write_text("stale")
    (source / "assets/app.js").write_text("changed")
    if os.name != "nt":
        (source / "assets/app.js").chmod(0o600)
    sync_output(source, destination)
    assert not (destination / "stale.js").exists()
    assert (destination / "assets/app.js").read_text() == "changed"
    if os.name != "nt":
        assert stat.S_IMODE((destination / "assets/app.js").stat().st_mode) == 0o600
    # File/Directory Replacements Keep The Projection Exact
    (source / "index.html").unlink()
    (source / "index.html").mkdir()
    (source / "index.html/new").write_text("new")
    sync_output(source, destination)
    assert (destination / "index.html/new").read_text() == "new"
    (source / "index.html/new").unlink()
    (source / "index.html").rmdir()
    (source / "index.html").write_text("file")
    sync_output(source, destination)
    assert (destination / "index.html").read_text() == "file"


def case_staging_links(tmp_path: Path) -> None:
    source = tmp_path / "source"
    destination = tmp_path / "destination"
    source.mkdir()
    destination.mkdir()
    outside = tmp_path / "outside"
    outside.write_text("outside")
    sentinel = destination / "stale.js"
    sentinel.write_text("keep until validation")
    link = source / "asset.js"
    try:
        link.symlink_to(outside)
    except OSError as error:
        if os.name == "nt":
            pytest.skip(f"Windows symlink privilege unavailable: {error}")
        raise
    with pytest.raises(ValueError, match="symlink"):
        sync_output(source, destination)
    assert sentinel.read_text() == "keep until validation"
    link.unlink()
    link.write_text("new")
    (destination / "asset.js").symlink_to(outside)
    with pytest.raises(ValueError, match="unsafe entry"):
        sync_output(source, destination)
    assert sentinel.read_text() == "keep until validation"
    assert outside.read_text() == "outside"
    (destination / "asset.js").unlink()
    (destination / "obsolete.js").symlink_to(outside)
    sync_output(source, destination)
    assert not (destination / "obsolete.js").is_symlink()
    assert outside.read_text() == "outside"
    ancestor = tmp_path / "linked"
    ancestor.symlink_to(destination, target_is_directory=True)
    with pytest.raises(ValueError, match="symlink"):
        stage_file(link, ancestor / "new.js")


def case_staging_special(tmp_path: Path) -> None:
    if os.name == "nt":
        pytest.skip("POSIX FIFOs are unavailable on Windows")
    source = tmp_path / "source"
    destination = tmp_path / "destination"
    source.mkdir()
    destination.mkdir()
    sentinel = destination / "stale.js"
    sentinel.write_text("keep")
    os.mkfifo(source / "special")
    with pytest.raises(ValueError, match="unsupported entry"):
        sync_output(source, destination)
    assert sentinel.read_text() == "keep"
    with pytest.raises(ValueError, match="separate directories"):
        sync_output(destination, destination)


# === Private Functions === #


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_staging_projection, id="staging_projection"),
        pytest.param(case_staging_links, id="staging_links"),
        pytest.param(case_staging_special, id="staging_special"),
    ],
)
def test_contract(scenario: Callable[..., None], tmp_path: Path) -> None:
    scenario(tmp_path)
