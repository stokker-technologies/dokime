# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Stage Build Output Without Following Links"""

import os
import shutil
import stat
from pathlib import Path


def clear_output(path: Path) -> None:
    """Prepare One Owned Output Without Following A Directory Link."""
    _ensure_directory(path.parent)
    if path.is_symlink():
        raise ValueError(f"owned output must not be a symlink: {path}")
    _remove(path)


def stage_file(source: Path, destination: Path) -> None:
    metadata = source.lstat()
    if not stat.S_ISREG(metadata.st_mode):
        raise ValueError(f"staged resource must be a regular file: {source}")
    _ensure_directory(destination.parent)
    if destination.is_symlink():
        raise ValueError(f"staged destination must not be a symlink: {destination}")
    if destination.exists() and destination.is_file() and source.read_bytes() == destination.read_bytes():
        _permissions(source, destination)
        return
    _remove(destination)
    shutil.copyfile(source, destination)
    _permissions(source, destination)


def sync_output(source: Path, destination: Path) -> None:
    source = source.absolute()
    destination = destination.absolute()
    if source == destination or source in destination.parents or destination in source.parents:
        raise ValueError("build output roots must be separate directories")
    _ensure_directory(source, create=False)
    _ensure_directory(destination)
    sources = _tree(source, allow_links=False)
    destinations = _tree(destination, allow_links=True)
    # Validate The Complete Projection Before Pruning
    for relative in sources.keys() & destinations.keys():
        if destinations[relative] == "symlink":
            raise ValueError(f"build output destination contains an unsafe entry: {destination / relative}")
    for relative in sorted(destinations.keys() - sources.keys(), key=lambda p: len(p.parts), reverse=True):
        _remove(destination / relative)
    for relative in sorted(sources, key=lambda p: (len(p.parts), str(p))):
        src = source / relative
        dst = destination / relative
        if sources[relative] == "directory":
            if dst.exists() and not dst.is_dir():
                _remove(dst)
            _ensure_directory(dst)
            _permissions(src, dst)
        else:
            stage_file(src, dst)


def _ensure_directory(path: Path, *, create: bool = True) -> None:
    for parent in reversed([path, *path.parents]):
        if parent.is_symlink():
            raise ValueError(f"directory must not be a symlink: {parent}")
        if parent.exists():
            if not parent.is_dir():
                raise ValueError(f"must be a directory: {parent}")
        elif create:
            parent.mkdir()
        else:
            raise ValueError(f"directory is missing: {parent}")


def _tree(root: Path, *, allow_links: bool) -> dict[Path, str]:
    entries: dict[Path, str] = {}

    def visit(directory: Path) -> None:
        for path in sorted(directory.iterdir()):
            mode = path.lstat().st_mode
            relative = path.relative_to(root)
            if stat.S_ISLNK(mode):
                if not allow_links:
                    raise ValueError(f"build output source contains a symlink: {path}")
                entries[relative] = "symlink"
            elif stat.S_ISDIR(mode):
                entries[relative] = "directory"
                visit(path)
            elif stat.S_ISREG(mode):
                entries[relative] = "file"
            else:
                raise ValueError(f"build output contains an unsupported entry: {path}")

    visit(root)
    return entries


def _remove(path: Path) -> None:
    if path.is_symlink():
        path.unlink()
    elif path.is_dir():
        shutil.rmtree(path)
    elif path.exists():
        path.unlink()


def _permissions(source: Path, destination: Path) -> None:
    src = stat.S_IMODE(source.stat().st_mode)
    dst = stat.S_IMODE(destination.stat().st_mode)
    same = bool(src & stat.S_IWRITE) == bool(dst & stat.S_IWRITE) if os.name == "nt" else src == dst
    if not same:
        destination.chmod(src)
