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
import subprocess
import tomllib
import zipfile
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import paths
from dokime_tools.package.assembly import write_file
from dokime_tools.package.staging import clear_output, stage_file
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def export(root: Path, profile: str) -> Path:
    """Record Shipping Command-Line Bytes and Legal Notices."""
    build = paths.output(root, "build", profile)
    native = (build / "product/target.txt").read_text(encoding="utf-8").strip()
    if native not in {"x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"}:
        raise ValueError(f"Package Product: unsupported distribution target: {native}")
    suffix = ".exe" if "windows" in native else ""
    # Read the Build Identity Embedded by DOKIME_SOURCE_REVISION and DOKIME_BUILD_CHANNEL.
    result = subprocess.run(
        [str(build / "bin" / f"dokime{suffix}"), "--version", "--json"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=True,
        creationflags=spawn_flags(),
    )
    identity = json.loads(result.stdout)
    config = tomllib.loads((root / "tools/project.toml").read_text(encoding="utf-8"))
    if identity["version"] != config["version"]["value"]:
        raise ValueError("Package Product: Rebuild the CLI for the Configured Product Version")
    plugins = []
    bundles = []
    for plugin in config.get("release", {}).get("plugins", []):
        if native not in plugin.get("targets", {}).values():
            continue
        path = build / "bin" / f"{plugin['package']}.dokime"
        result = subprocess.run(
            [str(build / "bin" / f"dokime{suffix}"), "plugin", "inspect", "--bundle", str(path), "--json"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=True,
            creationflags=spawn_flags(),
        )
        name = json.loads(result.stdout)["identity"]["name"]
        if name in plugins:
            raise ValueError(f"Package Product: Declare Each Default Plugin Identity Once: {name}")
        plugins.append(name)
        bundles.append(path)
    source = {
        "LICENSE": root / "LICENSE",
        "NOTICE": root / "target/notices/NOTICE",
        "THIRD-PARTY-NOTICES": root / "target/notices/THIRD-PARTY-NOTICES",
        f"bin/dokime{suffix}": build / "bin" / f"dokime{suffix}",
        f"bin/dokime-gui{suffix}": build / "bin" / f"dokime-gui{suffix}",
    }
    organization = root / "organization"
    for path in organization.rglob("*"):
        if path.is_symlink() or path.is_junction():
            raise ValueError(f"Package Product: distribution appearance input must not contain links: {path}")
        if path.is_file():
            source[path.relative_to(root).as_posix()] = path

    # Validate Inputs Before Replacing the Owned Export
    invalid = []
    for invalid_path in source.values():
        if not (invalid_path.is_symlink() or not invalid_path.is_file()):
            continue
        invalid.append(invalid_path)

    if invalid:
        raise ValueError(f"Package Product: distribution input must be a regular file: {invalid[0]}")
    destination = build / "product/distribution"
    clear_output(destination)
    hashes = {}
    for name, path in sorted(source.items()):
        stage_file(path, destination / name)
        hashes[name] = hashlib.sha256((destination / name).read_bytes()).hexdigest()
    for path in bundles:
        stage_file(path, destination / "plugins" / path.name)
    manifest = {
        "schema": 1,
        "version": identity["version"],
        "target": native,
        "source_revision": identity["revision"] or "unknown",
        "channel": identity["channel"] or "local",
        "plugins": plugins,
        "files": hashes,
    }
    (destination / "distribution.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return destination


def archive(source: Path, output: Path) -> Path:
    """Publish a Portable Distribution Without Recompiling or Rewriting Build Bytes."""
    manifest = json.loads((source / "distribution.json").read_text(encoding="utf-8"))
    output.mkdir(parents=True, exist_ok=True)
    destination = output / f"dokime-distribution-{manifest['version']}-{manifest['target']}.zip"
    checksums = manifest["files"]
    names = ["distribution.json", *sorted(checksums)]
    entries = [_entry(source, name) for name in names]
    bundle = zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED)
    try:
        for name, path in zip(names, entries, strict=True):
            write_file(bundle, path, name, 0o755 if name.startswith("bin/") else 0o644, checksum=checksums.get(name))
    finally:
        bundle.close()
    return destination


# === Private Functions === #


def _entry(source: Path, name: str) -> Path:
    relative = Path(name)
    unsafe = relative.is_absolute() or ".." in relative.parts
    if unsafe or "\\" in name or ":" in name:
        raise ValueError("Package Product: unsafe installer distribution path")
    path = source / relative
    if path.is_symlink() or not stat.S_ISREG(path.lstat().st_mode):
        raise ValueError("Package Product: installer distribution must contain regular files")
    return path


# === Entry Point === #

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default="dev", choices=("dev", "release"))
    options = parser.parse_args()
    directory = Path.cwd()
    archive(export(directory, options.profile), paths.output(directory, "package", options.profile) / "rust")
