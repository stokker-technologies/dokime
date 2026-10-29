# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Export Verified Native Installer Inputs"""

import hashlib
import json
import stat
import zipfile
from pathlib import Path

from dokime_tools.build import paths
from dokime_tools.package.staging import clear_output, stage_file


def export(root: Path, profile: str) -> Path:
    """Record Shipping Bytes, Their Organization Defaults And Compiled Identity."""
    build = paths.output(root, "build", profile)
    native = (build / "product/target.txt").read_text(encoding="utf-8").strip()
    if native not in {"x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"}:
        raise ValueError(f"unsupported installer target: {native}")
    suffix = ".exe" if "windows" in native else ""
    tauri = root / "crates/dokime-gui/src-tauri"
    config = json.loads((tauri / "tauri.conf.json").read_text(encoding="utf-8"))
    source = {
        "LICENSE": root / "LICENSE",
        **{f"bin/{name}{suffix}": build / "bin" / f"{name}{suffix}" for name in ("dokime", "dokime-gui")},
    }
    organization = root / "organization"
    if organization.is_symlink() or organization.is_junction() or not organization.is_dir():
        raise ValueError("installer organization input must be a regular directory")
    source["organization/organization.toml"] = organization / "organization.toml"
    pending = [organization]
    while pending:
        directory = pending.pop()
        for path in sorted(directory.iterdir()):
            if path.is_symlink() or path.is_junction():
                raise ValueError(f"installer organization input must not contain links: {path}")
            if path.is_dir():
                pending.append(path)
            elif path.is_file():
                source[path.relative_to(root).as_posix()] = path
            else:
                raise ValueError(f"installer organization input must be a regular file or directory: {path}")

    # Validate Inputs Before Replacing The Owned Export
    invalid = [path for path in source.values() if path.is_symlink() or not path.is_file()]
    if invalid:
        raise ValueError(f"installer input must be a regular file: {invalid[0]}")
    identity = {
        "identifier": config["identifier"],
        "slug": "dokime",
        "updater_public_key": config["plugins"]["updater"]["pubkey"],
    }
    destination = build / "product/installer-payload"
    clear_output(destination)
    hashes = {}
    for name, path in sorted(source.items()):
        stage_file(path, destination / name)
        hashes[name] = hashlib.sha256((destination / name).read_bytes()).hexdigest()
    manifest = {"schema": 1, "version": config["version"], "target": native, "identity": identity, "files": hashes}
    (destination / "payload.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return destination


def archive(source: Path, output: Path) -> Path:
    """Publish A Portable Payload Without Recompiling Or Rewriting Build Bytes."""
    manifest = json.loads((source / "payload.json").read_text(encoding="utf-8"))
    output.mkdir(parents=True, exist_ok=True)
    destination = output / f"dokime-payload-{manifest['version']}-{manifest['target']}.zip"
    checksums = manifest["files"]
    names = ["payload.json", *sorted(checksums)]
    entries = [_entry(source, name, checksums.get(name)) for name in names]
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as bundle:
        _write_entries(bundle, entries)
    return destination


def _entry(source: Path, name: str, checksum: str | None) -> tuple[zipfile.ZipInfo, bytes]:
    relative = Path(name)
    unsafe = relative.is_absolute() or ".." in relative.parts
    if unsafe or "\\" in name or ":" in name:
        raise ValueError("unsafe installer payload path")
    path = source / relative
    if path.is_symlink() or not stat.S_ISREG(path.lstat().st_mode):
        raise ValueError("installer payload must contain regular files")
    data = path.read_bytes()
    if checksum is not None and hashlib.sha256(data).hexdigest() != checksum:
        raise ValueError(f"installer payload changed after Build: {name}")

    entry = zipfile.ZipInfo(name)
    entry.create_system = 3
    entry.external_attr = (stat.S_IFREG | (0o755 if name.startswith("bin/") else 0o644)) << 16
    entry.compress_type = zipfile.ZIP_DEFLATED
    return entry, data


def _write_entries(bundle: zipfile.ZipFile, entries: list[tuple[zipfile.ZipInfo, bytes]]) -> None:
    for entry, data in entries:
        bundle.writestr(entry, data)
