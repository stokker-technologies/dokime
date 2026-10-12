# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare A Locked Source Distribution And Build Its Native Wheel"""

# === Imports === #

# Standard Library Imports
import gzip
import io
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import tomllib

# Third Party Imports

# Local Imports
from dokime_tools.package.wheel import select_wheel

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def prepare(archive: Path, source_root: Path | None = None) -> None:
    # Maturin Keeps Only SDK Workspace Members But Copies The Full Cargo Lock.
    with tempfile.TemporaryDirectory() as temporary:
        staging = Path(temporary)
        with tarfile.open(archive) as source:
            entries = source.getmembers()
            roots = {Path(entry.name).parts[0] for entry in entries}
            if len(roots) != 1:
                raise ValueError("source distribution must have one package root")
            source.extractall(staging, filter="data")
        package = staging / roots.pop()
        lock = package / "Cargo.lock"
        original = _packages(lock)
        subprocess.run(
            ["cargo", "metadata", "--offline", "--format-version", "1"],
            cwd=package,
            stdout=subprocess.DEVNULL,
            check=True,
        )
        resolved = _packages(lock)
        if not resolved or not resolved <= original:
            raise ValueError("sdist resolution changed a locked version or checksum")
        subprocess.run(
            ["cargo", "metadata", "--offline", "--locked", "--format-version", "1"],
            cwd=package,
            stdout=subprocess.DEVNULL,
            check=True,
        )
        replacement = lock.read_bytes()
        branding: list[tuple[str, Path]] = []
        if source_root is not None:
            bundle = source_root / "branding"
            paths = [bundle / "brand.toml"]
            for folder in ("theme", "fonts", "logos"):
                directory = bundle / folder
                if directory.is_symlink() or not directory.is_dir():
                    raise ValueError(
                        "sdist branding directories must be native source inputs"
                    )
                paths.extend(
                    sorted(path for path in directory.iterdir() if path.is_file())
                )
            if bundle.is_symlink() or any(
                path.is_symlink() or not path.is_file() for path in paths
            ):
                raise ValueError("sdist branding files must be regular source inputs")
            branding = [(path.relative_to(bundle).as_posix(), path) for path in paths]
        # Preserve Sources And Include Workspace Branding Required By Build.rs.
        with (
            tarfile.open(archive) as source,
            (staging / "prepared.tar.gz").open("wb") as output,
            gzip.GzipFile(
                fileobj=output, filename="", mode="wb", mtime=0
            ) as compressed,
            tarfile.open(fileobj=compressed, mode="w") as target,
        ):
            for entry in entries:
                if entry.name == f"{package.name}/Cargo.lock":
                    entry.size = len(replacement)
                    target.addfile(entry, io.BytesIO(replacement))
                else:
                    target.addfile(
                        entry, source.extractfile(entry) if entry.isfile() else None
                    )
            names = {entry.name for entry in entries}
            for relative, path in branding:
                name = f"{package.name}/branding/{relative}"
                if name in names:
                    existing = source.extractfile(name)
                    if existing is None or existing.read() != path.read_bytes():
                        raise ValueError("sdist branding differs from declared source")
                    continue
                target.add(path, arcname=name, recursive=False)
        archive.write_bytes((staging / "prepared.tar.gz").read_bytes())


# === Private Functions === #


def _packages(lock: Path) -> set[tuple[str, str, str, str]]:
    contents = tomllib.loads(lock.read_text(encoding="utf-8"))
    return {
        (
            package["name"],
            package["version"],
            package.get("source", ""),
            package.get("checksum", ""),
        )
        for package in contents["package"]
    }


# === Entry Point === #

if __name__ == "__main__":
    directory = Path(sys.argv[1])
    archive = select_wheel(directory, "dokime-*.tar.gz")
    prepare(archive, Path.cwd())
    if sys.platform == "linux":
        from dokime_tools.package.linux import build

        build(archive, directory, Path.cwd())
        raise SystemExit(0)
    subprocess.run(
        [
            "uv",
            "build",
            str(archive),
            "--wheel",
            "--out-dir",
            str(directory),
            "--config-setting",
            "maturin.build-args=--locked",
        ],
        check=True,
    )
