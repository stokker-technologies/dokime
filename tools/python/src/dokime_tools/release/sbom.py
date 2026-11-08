# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Generate Source and Artifact CycloneDX Inventories."""

# === Imports === #

# Standard Library Imports
import argparse
import hashlib
import json
import shutil
import tarfile
import tempfile
import zipfile
from pathlib import Path

# Third Party Imports
# Local Imports
from . import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def workspace(directory: Path, target: str, output: Path) -> list[Path]:
    """Run Cargo CycloneDX in an Isolated Source Snapshot, Preserving the Checkout."""
    output.mkdir(parents=True, exist_ok=True)
    tracked = policy.execute(directory, ["git", "ls-files", "-z"], capture=True).split("\0")
    with tempfile.TemporaryDirectory(prefix="dokime-sbom-") as temporary:
        snapshot = Path(temporary)
        for name in tracked:
            source = directory / name
            if not name or not source.is_file():
                continue
            if source.suffix not in {".rs", ".toml"} and source.name != "Cargo.lock":
                continue
            destination = snapshot / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, destination)
        policy.execute(snapshot, ["cargo", "cyclonedx", "--format", "json", "--spec-version", "1.5", "--all", "--target", target])
        if (snapshot / "Cargo.lock").read_bytes() != (directory / "Cargo.lock").read_bytes():
            raise policy.ToolError("Cargo CycloneDX changed the dependency resolution")
        result = []
        for source in sorted(snapshot.rglob("*.cdx.json")):
            relative = source.relative_to(snapshot)
            name = "-".join(relative.parts).removesuffix(".cdx.json") + f"-{target}.cdx.json"
            destination = output / name
            shutil.copyfile(source, destination)
            result.append(destination)
        if not result:
            raise policy.ToolError("Cargo CycloneDX produced no inventories")
        return result


def shell(directory: Path, output: Path) -> Path:
    """Scan the Locked Shell Dependency Inventory with Syft."""
    with tempfile.TemporaryDirectory(prefix="dokime-shell-sbom-") as temporary:
        snapshot = Path(temporary)
        for name in ("package.json", "pnpm-lock.yaml"):
            shutil.copyfile(directory / "crates/dokime-gui" / name, snapshot / name)
        document = json.loads(policy.execute(directory, ["syft", "scan", f"dir:{snapshot}", "-o", "cyclonedx-json"], capture=True))
    if not document.get("components"):
        raise policy.ToolError("Syft produced an empty shell inventory")
    output.mkdir(parents=True, exist_ok=True)
    destination = output / "dokime-shell.cdx.json"
    destination.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    return destination


def unpack(archive: Path, destination: Path) -> None:
    """Reject Traversal and Links Before Extracting a Distribution."""
    if zipfile.is_zipfile(archive):
        with zipfile.ZipFile(archive) as bundle:
            for member in bundle.infolist():
                name = member.filename
                mode = member.external_attr >> 16
                if "\\" in name or ":" in name or not (destination / name).resolve().is_relative_to(destination.resolve()):
                    raise policy.ToolError(f"unsafe archive member: {name}")
                if mode & 0o170000 == 0o120000:
                    raise policy.ToolError(f"archive link is not supported: {name}")
            bundle.extractall(destination)
        return
    if tarfile.is_tarfile(archive):
        with tarfile.open(archive) as bundle:
            bundle.extractall(destination, filter="data")
        return
    shutil.copyfile(archive, destination / archive.name)


def artifact(directory: Path, source: Path, output: Path, rust: list[Path]) -> Path:
    """Scan Unpacked Bytes and Include the Target's Conservative Rust Source Inventory."""
    with tempfile.TemporaryDirectory(prefix="dokime-artifact-") as temporary:
        unpacked = Path(temporary)
        unpack(source, unpacked)
        document = json.loads(policy.execute(directory, ["syft", "scan", f"dir:{unpacked}", "-o", "cyclonedx-json"], capture=True))
    components = {item.get("purl", item.get("bom-ref", item["name"])): item for item in document.get("components", [])}
    for path in rust:
        for item in json.loads(path.read_text(encoding="utf-8")).get("components", []):
            key = item.get("purl", item.get("bom-ref", item["name"]))
            if key in components:
                continue
            item["bom-ref"] = key
            components[key] = item
    document["components"] = list(components.values())
    root_component = document.setdefault("metadata", {}).setdefault("component", {})
    root_component.update(
        {
            "type": "file",
            "name": source.name,
            "hashes": [{"alg": "SHA-256", "content": hashlib.sha256(source.read_bytes()).hexdigest()}],
            "properties": [
                {
                    "name": "dokime:inventory-scope",
                    "value": "Syft artifact scan plus conservative source inventories supplied for this target and artifact kind",
                }
            ],
        }
    )
    destination = output / (source.name + ".cdx.json")
    output.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    return destination


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True)
    parser.add_argument("--output", type=Path, default=Path("target/sbom"))
    parser.add_argument("--artifacts", type=Path)
    options = parser.parse_args()
    directory = policy.root()
    inventories = workspace(directory, options.target, options.output)
    shell_inventory = shell(directory, options.output)
    if options.artifacts:
        sources = [
            source
            for source in sorted(options.artifacts.rglob("*"))
            if source.is_file() and source.name.endswith((".zip", ".tar.gz", ".whl", ".exe", ".deb", ".rpm"))
        ]
        if not sources:
            raise policy.ToolError("no distribution artifacts found")
        for source in sources:
            desktop = source.name.startswith("dokime-distribution-") or source.name.endswith(("-setup.exe", ".deb", ".rpm"))
            source_inventories = [*inventories, shell_inventory] if desktop else inventories
            artifact(directory, source, options.output, source_inventories)
    print(f"Generated {len(inventories)} Rust inventories for {options.target}")
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
