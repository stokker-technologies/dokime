# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Export Verified Command-Line Distribution Inputs"""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import stat
import tomllib
import zipfile
from pathlib import Path

# Third Party Imports
from harmos_tools.build import paths

# Local Imports
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def export(root: Path, profile: str) -> Path:
    """Record Shipping Command-Line Bytes And Legal Notices."""
    build = paths.output(root, "build", profile)
    native = (build / "product/target.txt").read_text(encoding="utf-8").strip()
    if native not in {"x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"}:
        raise ValueError(f"unsupported distribution target: {native}")
    suffix = ".exe" if "windows" in native else ""
    config = tomllib.loads((root / "tools/project.toml").read_text(encoding="utf-8"))
    source = {
        "LICENSE": root / "LICENSE",
        "NOTICE": root / "target/notices/NOTICE",
        "THIRD-PARTY-NOTICES": root / "target/notices/THIRD-PARTY-NOTICES",
        f"bin/dokime-next{suffix}": build / "bin" / f"dokime-next{suffix}",
        f"bin/dokime-gui{suffix}": build / "bin" / f"dokime-gui{suffix}",
    }
    organization = root / "organization"
    for path in organization.rglob("*"):
        if path.is_symlink() or path.is_junction():
            raise ValueError(f"distribution appearance input must not contain links: {path}")
        if path.is_file():
            source[path.relative_to(root).as_posix()] = path

    # Validate Inputs Before Replacing the Owned Export
    invalid = [path for path in source.values() if path.is_symlink() or not path.is_file()]
    if invalid:
        raise ValueError(f"distribution input must be a regular file: {invalid[0]}")
    destination = build / "product/distribution"
    clear_output(destination)
    hashes = {}
    for name, path in sorted(source.items()):
        stage_file(path, destination / name)
        hashes[name] = hashlib.sha256((destination / name).read_bytes()).hexdigest()
    manifest = {"schema": 1, "version": config["version"]["value"], "target": native, "files": hashes}
    (destination / "distribution.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return destination


def archive(source: Path, output: Path) -> Path:
    """Publish A Portable Distribution Without Recompiling Or Rewriting Build Bytes."""
    manifest = json.loads((source / "distribution.json").read_text(encoding="utf-8"))
    output.mkdir(parents=True, exist_ok=True)
    destination = output / f"dokime-distribution-{manifest['version']}-{manifest['target']}.zip"
    checksums = manifest["files"]
    names = ["distribution.json", *sorted(checksums)]
    entries = [_entry(source, name, checksums.get(name)) for name in names]
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as bundle:
        _write_entries(bundle, entries)
    return destination


# === Private Functions === #


def _entry(source: Path, name: str, checksum: str | None) -> tuple[zipfile.ZipInfo, bytes]:
    relative = Path(name)
    unsafe = relative.is_absolute() or ".." in relative.parts
    if unsafe or "\\" in name or ":" in name:
        raise ValueError("unsafe installer distribution path")
    path = source / relative
    if path.is_symlink() or not stat.S_ISREG(path.lstat().st_mode):
        raise ValueError("installer distribution must contain regular files")
    data = path.read_bytes()
    if checksum is not None and hashlib.sha256(data).hexdigest() != checksum:
        raise ValueError(f"installer distribution changed after Build: {name}")

    entry = zipfile.ZipInfo(name)
    entry.create_system = 3
    entry.external_attr = (stat.S_IFREG | (0o755 if name.startswith("bin/") else 0o644)) << 16
    entry.compress_type = zipfile.ZIP_DEFLATED
    return entry, data


def _write_entries(bundle: zipfile.ZipFile, entries: list[tuple[zipfile.ZipInfo, bytes]]) -> None:
    for entry, data in entries:
        bundle.writestr(entry, data)


# === Entry Point === #

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default="dev", choices=("dev", "release"))
    options = parser.parse_args()
    directory = Path.cwd()
    archive(export(directory, options.profile), paths.output(directory, "package", options.profile) / "rust")
