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
import base64
import csv
import email.parser
import hashlib
import io
import json
import shutil
import tarfile
import tempfile
import tomllib
import urllib.request
import zipfile
from pathlib import Path

from ..build import paths
from ..package.staging import stage_file

# Third Party Imports
# Local Imports
from . import policy, sbom

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
    """Ship Notices in the Installer's Installed LICENSE and Portable Layout."""
    manifest = destination / "distribution.json"
    document = json.loads(manifest.read_text(encoding="utf-8"))
    for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
        stage_file(directory / name, destination / name)
        document["files"][name] = hashlib.sha256((destination / name).read_bytes()).hexdigest()
    text = (directory / "LICENSE").read_text(encoding="utf-8")
    for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
        text += "\n\n" + (directory / name).read_text(encoding="utf-8")
    (destination / "LICENSE").write_text(text, encoding="utf-8", newline="\n")
    document["files"]["LICENSE"] = hashlib.sha256((destination / "LICENSE").read_bytes()).hexdigest()
    manifest.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")


def archive(directory: Path, source: Path) -> None:
    """Add Notices Before Release Hashing, Preserving Executable Modes and Wheel RECORD."""
    notices = {name: (directory / name).read_bytes() for name in ("NOTICE", "THIRD-PARTY-NOTICES")}
    with tempfile.TemporaryDirectory(prefix="dokime-notices-") as temporary:
        result = Path(temporary) / source.name
        if source.name.endswith((".zip", ".whl")):
            with zipfile.ZipFile(source) as original, zipfile.ZipFile(result, "w", compression=zipfile.ZIP_DEFLATED) as output:
                wheel = source.suffix == ".whl"
                record = next((name for name in original.namelist() if name.endswith(".dist-info/RECORD")), None) if wheel else None
                if wheel and record is None:
                    raise policy.ToolError("wheel has no RECORD")
                prefix = record.rsplit("/", 1)[0] + "/licenses/" if record else ""
                added = {prefix + name: data for name, data in notices.items()}
                for entry in original.infolist():
                    if entry.filename not in {*added, record}:
                        output.writestr(entry, original.read(entry))
                for name, data in added.items():
                    output.writestr(name, data)
                if record:
                    rows = [row for row in csv.reader(io.StringIO(original.read(record).decode("utf-8"))) if row[0] not in {*added, record}]
                    for name, data in added.items():
                        digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).decode().rstrip("=")
                        rows.append([name, "sha256=" + digest, str(len(data))])
                    rows.append([record, "", ""])
                    contents = io.StringIO(newline="")
                    csv.writer(contents).writerows(rows)
                    output.writestr(record, contents.getvalue())
        elif source.name.endswith(".tar.gz"):
            with tarfile.open(source) as original, tarfile.open(result, "w:gz") as output:
                for entry in original.getmembers():
                    if entry.name not in notices:
                        output.addfile(entry, original.extractfile(entry) if entry.isfile() else None)
                for name, data in notices.items():
                    entry = tarfile.TarInfo(name)
                    entry.size = len(data)
                    entry.mode = 0o644
                    output.addfile(entry, io.BytesIO(data))
        else:
            raise policy.ToolError(f"unsupported notice archive: {source}")
        shutil.copyfile(result, source)


def generate(directory: Path) -> dict:
    """Resolve Approved Licenses, Then Collect Actual Copyright and License Text."""
    output = directory / "target/notices"
    output.mkdir(parents=True, exist_ok=True)
    document = sbom.exceptions(directory)
    metadata = json.loads(policy.execute(directory, ["cargo", "metadata", "--locked", "--format-version", "1"], capture=True))
    packages = {(item["name"], item["version"]) for item in metadata["packages"]}
    licenses = {}
    for entry in document.get("license", []):
        if (entry["package"], entry["version"]) in packages:
            licenses.setdefault(entry["package"], set()).update(entry.get("accepted", [entry["license"]]))
    for name in licenses:
        allowed = {item["version"] for item in document["license"] if item["package"] == name}
        if any(package == name and version not in allowed for package, version in packages):
            raise policy.ToolError(f"license exception does not cover every resolved {name} version")
    license_policy = tomllib.loads((directory / "deny.toml").read_text(encoding="utf-8"))
    config = f"accepted = {json.dumps(license_policy['licenses']['allow'])}\n"
    config += (directory / "about.toml").read_text(encoding="utf-8")
    for name, accepted in sorted(licenses.items()):
        config += f"\n[{json.dumps(name)}]\naccepted = {json.dumps(sorted(accepted))}\n"
    configured = output / "about.toml"
    configured.write_text(config, encoding="utf-8", newline="\n")
    policy.execute(
        directory,
        [
            "cargo",
            "about",
            "generate",
            "--locked",
            "--workspace",
            "--config",
            str(configured),
            "--format",
            "json",
            "--output-file",
            str(output / "rust.json"),
        ],
    )
    policy.execute(
        directory,
        [
            "cargo",
            "about",
            "generate",
            "--locked",
            "--workspace",
            "--config",
            str(configured),
            "about.hbs",
            "--output-file",
            str(output / "rust.txt"),
        ],
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
    print(json.dumps(counts))
    return counts


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stage", type=Path)
    parser.add_argument("--archives", type=Path)
    parser.add_argument("--python", action="store_true")
    parser.add_argument("--profile", choices=("dev", "release"))
    parser.add_argument("--target", default="native")
    options = parser.parse_args()
    directory = policy.root()
    generate(directory)
    destination = options.stage
    if options.profile:
        target = paths.target() if options.target == "native" else options.target
        if options.python:
            options.archives = paths.output(directory, "package", options.profile, target) / "python"
        else:
            destination = paths.output(directory, "build", options.profile, target) / "product/installer-distribution"
    if destination:
        stage(directory, destination)
    if options.archives:
        wheels = sorted(options.archives.glob("*.whl"))
        if not wheels:
            raise policy.ToolError(f"no packaged wheels in {options.archives}")
        for source in wheels:
            archive(directory, source)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
