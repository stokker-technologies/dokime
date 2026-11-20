# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Own Package Assembly Operations."

# === Imports === #

# Standard Library Imports
import hashlib
import shutil
import stat
import tarfile
import zipfile
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..build import paths
from . import changelog, definition

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def assemble(directory: Path, language: str, profile: str, target: str, staging: Path) -> None:
    build = paths.output(directory, "build", profile, target)
    substitutions = {"profile": profile, "target": target, "version": policy.config(directory).get("version", {}).get("value")}
    if language in {"python", "typescript"}:
        source = build / {"python": "python", "typescript": "web"}[language]
        if target != "any" and not source.is_dir():
            raise policy.ToolError(f"Package Product: missing Product Build input: {source}")
        for entry in source.rglob("*"):
            if entry.is_symlink():
                raise policy.ToolError("Package Product: linked package input")
            if not entry.is_file():
                continue
            relative = entry.relative_to(source)
            original = definition.regular(source, relative.as_posix())
            destination = staging / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(original, destination)
    if language == "rust":
        # Shipped Plugin Bundles Travel Beside the Executables the Product Build Staged
        extension = policy.config(directory).get("release", {}).get("plugin_extension")
        for plugin in policy.config(directory).get("release", {}).get("plugins", []):
            if plugin.get("kind") == "sidecar" and target in plugin.get("targets", {}):
                name = f"{plugin['package']}.{extension}"
                shutil.copy2(definition.regular(build, "bin/" + name), staging / name)
    for item in definition.definitions(directory, profile, target):
        if item["language"] != language:
            continue
        output = staging / item["output"]
        if "triple" in item:
            actual = definition.regular(build, "product/target.txt").read_text().strip()
            if actual != item["triple"]:
                raise policy.ToolError(f"Build Rust target {actual} differs from package target {item['triple']}")
            source = definition.regular(build, "bin/" + item["binary"])
            documentation = [
                (definition.regular(directory, "LICENSE"), "LICENSE"),
                (definition.regular(directory, "README.md"), "README.md"),
            ]
            value = substitutions["version"]
            changelog_file = staging / "CHANGELOG.md"
            changelog_file.write_text(changelog.notes(directory, "v" + value), encoding="utf-8", newline="\n")
            documentation.append((changelog_file, "CHANGELOG.md"))
            contents = [(source, item["binary"]), *documentation]
            if output.name.endswith(".zip"):
                archive = zipfile.ZipFile(output, "x", compression=zipfile.ZIP_DEFLATED)
                try:
                    for original, name in contents:
                        write_file(archive, original, name, 0o755 if original == source else 0o644)
                finally:
                    archive.close()
            else:
                archive = tarfile.open(output, "x:gz")
                try:
                    for original, name in contents:
                        metadata = archive.gettarinfo(str(original), arcname=name)
                        metadata.mode = 0o755 if original == source else 0o644
                        stream = original.open("rb")
                        try:
                            archive.addfile(metadata, stream)
                        finally:
                            stream.close()
                finally:
                    archive.close()
            changelog_file.unlink()
            continue
        destinations = set()
        archive = zipfile.ZipFile(output, "x", compression=zipfile.ZIP_DEFLATED)
        try:
            for entry in item["inputs"]:
                prefix = "target/build/{profile}/{target}/"
                if not entry["path"].startswith(prefix):
                    raise policy.ToolError("Package Product: bundle inputs must come from the matching Product Build")
                relative = entry["path"][len(prefix) :].format(**substitutions)
                source = definition.regular(build, relative)
                name = entry["destination"].format(**substitutions)
                parts = name.split("/")
                unsafe = (
                    not name
                    or any(part in {"", ".", ".."} for part in parts)
                    or "\\" in name
                    or ":" in name
                    or any(ord(char) < 32 for char in name)
                )
                overlaps = any(
                    name.casefold() == old or name.casefold().startswith(old + "/") or old.startswith(name.casefold() + "/")
                    for old in destinations
                )
                if unsafe or overlaps:
                    raise policy.ToolError("Package Product: unsafe or duplicate bundle destination")
                destinations.add(name.casefold())
                mode = entry.get("mode", 0o644)
                if type(mode) is not int or not 0 <= mode <= 0o777:
                    raise policy.ToolError("Package Product: bundle mode must contain only ordinary permission bits")
                write_file(archive, source, name, mode)
        finally:
            archive.close()


def write_file(archive: zipfile.ZipFile, source: Path, name: str, mode: int = 0o644, *, checksum: str | None = None) -> str:
    """Write a Regular ZIP Member With Explicit Unix Permissions."""
    metadata = zipfile.ZipInfo.from_file(source, name)
    metadata.create_system = 3
    metadata.external_attr = (stat.S_IFREG | mode) << 16
    metadata.compress_type = zipfile.ZIP_DEFLATED
    data = source.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if checksum is not None and digest != checksum:
        raise policy.ToolError(f"Package Product: Rebuild the Changed Archive Input: {name}")
    archive.writestr(metadata, data)
    return digest


# === Private Functions === #

# === Entry Point === #
