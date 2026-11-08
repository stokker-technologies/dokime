# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Collect Rust, Shell and Locked Python License Notices."""

# === Imports === #

# Standard Library Imports
import argparse
import email.parser
import hashlib
import io
import json
import shutil
import tomllib
import urllib.request
import zipfile
from pathlib import Path

# Third Party Imports
# Local Imports
from . import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# These Published npm Versions Omit Standalone License Files.
SHELL_LICENSE_SOURCES = {
    (
        "@bufbuild/protobuf",
        "2.16.0",
    ): "https://raw.githubusercontent.com/bufbuild/protobuf-es/b3b232f7f466eda27c66e8f63c82f930578b6cc6/LICENSE",
    # The Published gitHead Is Unavailable; Pin the Upstream MIT License Independently.
    (
        "react-remove-scroll-bar",
        "2.3.8",
    ): "https://raw.githubusercontent.com/theKashey/react-remove-scroll-bar/8ca9ba5ea52de03308fe8ced94f7b159a44d28ff/LICENSE",
}

# === Types === #

# === Classes === #

# === Public Functions === #


def python_notices(lock: dict) -> tuple[str, int]:
    """Read License Metadata and Text from Hash-Verified Wheels Named by uv.lock."""
    sections = ["Python Locked Dependency Notices (Including Build and Development Tools)"]
    count = 0
    for package in lock["package"]:
        if "registry" not in package.get("source", {}):
            continue
        wheels = package.get("wheels", [])
        if not wheels:
            raise policy.ToolError(f"no locked wheel available for license inventory: {package['name']}")
        wheel = min(wheels, key=lambda item: item.get("size", 0))
        with urllib.request.urlopen(wheel["url"], timeout=60) as response:
            data = response.read()
        actual = "sha256:" + hashlib.sha256(data).hexdigest()
        if actual != wheel["hash"]:
            raise policy.ToolError(f"locked wheel hash mismatch: {package['name']}")
        sections.append(f"\n{package['name']} {package['version']}")
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            metadata_name = next(name for name in archive.namelist() if name.endswith(".dist-info/METADATA"))
            metadata = email.parser.BytesParser().parsebytes(archive.read(metadata_name))
            license_name = metadata.get("License-Expression") or metadata.get("License")
            if not license_name:
                license_name = "; ".join(value for value in metadata.get_all("Classifier", []) if value.startswith("License ::"))
            if not license_name:
                raise policy.ToolError(f"missing Python license metadata: {package['name']}")
            sections.append(license_name)
            files = [
                name
                for name in archive.namelist()
                if not name.endswith("/")
                and (".dist-info/licenses/" in name or Path(name).name.lower().startswith(("license", "licence", "copying", "notice")))
            ]
            if not files:
                raise policy.ToolError(f"missing Python license text: {package['name']}")
            for name in sorted(files):
                sections.append(archive.read(name).decode("utf-8"))
        count += 1
    return "\n\n".join(sections), count


def shell_notices(document: dict) -> tuple[str, int]:
    """Include Installed License Text for Every Production pnpm Dependency."""
    sections = ["Shell Production Dependency Notices"]
    inventory = set()
    for license_name, packages in sorted(document.items()):
        for package in packages:
            versions = package.get("versions", [package.get("version", "")])
            sections.append(f"\n{package['name']} {', '.join(versions)}\n{license_name}")
            inventory.update((package["name"], version) for version in versions)
            texts = set()
            for path in package["paths"]:
                for source in sorted(Path(path).iterdir()):
                    if source.is_file() and source.name.lower().startswith(("license", "licence", "copying", "notice")):
                        texts.add(source.read_text(encoding="utf-8"))
            if not texts:
                for version in versions:
                    url = SHELL_LICENSE_SOURCES.get((package["name"], version))
                    if url is None:
                        raise policy.ToolError(f"missing shell license text: {package['name']}")
                    with urllib.request.urlopen(url, timeout=60) as response:
                        texts.add(f"License Source: {url}\n\n" + response.read().decode("utf-8"))
            if package["name"] == "@bufbuild/protobuf":
                for path in package["paths"]:
                    for relative in ("dist/esm/index.js", "dist/esm/wire/varint.js"):
                        header = []
                        for line in (Path(path) / relative).read_text(encoding="utf-8").splitlines():
                            if not line.startswith("//"):
                                break
                            header.append(line.removeprefix("//").lstrip())
                        if not header:
                            raise policy.ToolError(f"missing protobuf copyright header: {relative}")
                        texts.add("\n".join(header))
            sections.extend(sorted(texts))
    return "\n\n".join(sections), len(inventory)


def stage(directory: Path, destination: Path) -> None:
    """Place Notices in an Existing Installer Distribution and Update Its Inventory."""
    manifest = destination / "distribution.json"
    document = json.loads(manifest.read_text(encoding="utf-8"))
    for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
        source = directory / name
        shutil.copyfile(source, destination / name)
        document["files"][name] = hashlib.sha256(source.read_bytes()).hexdigest()
    manifest.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stage", type=Path)
    options = parser.parse_args()
    directory = policy.root()
    output = directory / "target/notices"
    output.mkdir(parents=True, exist_ok=True)
    policy.execute(
        directory, ["cargo", "about", "generate", "--locked", "--workspace", "--format", "json", "--output-file", str(output / "rust.json")]
    )
    policy.execute(
        directory, ["cargo", "about", "generate", "--locked", "--workspace", "about.hbs", "--output-file", str(output / "rust.txt")]
    )
    rust = json.loads((output / "rust.json").read_text(encoding="utf-8"))
    crates = {(item["crate"]["name"], item["crate"]["version"]) for license_ in rust["licenses"] for item in license_["used_by"]}
    shell = json.loads(
        policy.execute(directory, ["pnpm", "--dir", "crates/dokime-gui", "licenses", "list", "--prod", "--json"], capture=True)
    )
    shell_text, shell_count = shell_notices(shell)
    python_text, python_count = python_notices(tomllib.loads((directory / "uv.lock").read_text(encoding="utf-8")))
    text = (output / "rust.txt").read_text(encoding="utf-8") + "\n\n" + shell_text + "\n\n" + python_text + "\n"
    (directory / "THIRD-PARTY-NOTICES").write_text(text, encoding="utf-8")
    counts = {"rust": len(crates), "shell": shell_count, "python": python_count}
    (output / "counts.json").write_text(json.dumps(counts, indent=2) + "\n", encoding="utf-8")
    if options.stage:
        stage(directory, options.stage)
    print(json.dumps(counts))
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
