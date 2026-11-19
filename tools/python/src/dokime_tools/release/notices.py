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
import http.client
import io
import json
import os
import random
import re
import shutil
import sys
import tarfile
import tempfile
import time
import tomllib
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from collections.abc import Callable
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..build import paths
from ..package.staging import stage_file
from . import sbom

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Six Attempts Absorb Brief Registry Outages Without Blocking Packaging Indefinitely.
DOWNLOAD_ATTEMPTS = 6
# Start at Two Seconds to Let Dropped Connections Recover Before Retrying.
DOWNLOAD_BASE_DELAY_S = 2
DOWNLOAD_MAX_DELAY_S = 30

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

# These Published Python Wheels Omit Standalone License Files.
PYTHON_LICENSE_SOURCES = {
    ("grpcio-tools", "1.85.1"): (
        "https://raw.githubusercontent.com/grpc/grpc/558b7e3b09e42e2160018732acbbf56c9094c84d/LICENSE",
        "58c86a3f2b5cca0ebdf9ed3e7f4e6dc38e3b38796d8c3dc92064c62a4e39d0dd",
    ),
}

# === Types === #

# === Classes === #

# === Public Functions === #


def download(
    url: str | urllib.request.Request,
    *,
    operation: str = "download",
    timeout_s: float = 60,
    opener: Callable[..., http.client.HTTPResponse] | None = None,
    before_retry: Callable[[], None] | None = None,
) -> bytes:
    """Fetch Complete Bytes with Bounded Retries for Transient Network Failures."""
    address = url.full_url if isinstance(url, urllib.request.Request) else url
    open_request = opener or urllib.request.urlopen
    for attempt in range(1, DOWNLOAD_ATTEMPTS + 1):
        try:
            response = open_request(url, timeout=timeout_s)
            try:
                return response.read()
            finally:
                response.close()
        except (OSError, urllib.error.URLError, http.client.HTTPException) as error:
            retry = not isinstance(error, urllib.error.HTTPError) or 500 <= error.code < 600
            if isinstance(error, urllib.error.HTTPError):
                error.close()
            if not retry or attempt == DOWNLOAD_ATTEMPTS:
                raise policy.ToolError(
                    f"Collect Third-Party Notices: {operation} failed: {address} after {attempt} attempts: {error}"
                ) from error
            delay_s = min(DOWNLOAD_MAX_DELAY_S, DOWNLOAD_BASE_DELAY_S * 2 ** (attempt - 1))
            time.sleep(random.uniform(delay_s / 2, delay_s))
            if before_retry is not None:
                before_retry()
    raise AssertionError("unreachable download attempt")


def python_notices(lock: dict) -> tuple[str, int]:
    """Read Locked Wheel Notices, Using Pinned License Sources for Incomplete Wheels."""
    sections = ["Python Locked Dependency Notices (Including Build and Development Tools)"]
    count = 0
    cache_hits = 0
    downloads = 0
    license_downloads = 0
    cache = Path(os.environ.get("UV_CACHE_DIR") or policy.execute(Path.cwd(), ["uv", "cache", "dir"], capture=True).strip())
    hashes = set()
    for hashes_package in lock["package"]:
        for hashes_wheel in hashes_package.get("wheels", []):
            hashes.add(hashes_wheel["hash"].removeprefix("sha256:"))

    cached = {}
    # uv Usually Stores Unpacked Wheels; Only Original Archives Can Be Hash-Verified.
    for directory, _, files in os.walk(cache):
        for name in files:
            if name.endswith(".whl") or name in hashes:
                cached.setdefault(name, []).append(Path(directory) / name)
    for package in lock["package"]:
        if "registry" not in package.get("source", {}):
            continue
        wheels = package.get("wheels", [])
        if not wheels:
            raise policy.ToolError(f"Collect Third-Party Notices: no locked wheel available for license inventory: {package['name']}")

        def sort_key(item: dict) -> int:
            return item.get("size", 0)

        wheel = min(wheels, key=sort_key)
        digest = wheel["hash"].removeprefix("sha256:")
        filename = Path(urllib.parse.unquote(urllib.parse.urlsplit(wheel["url"]).path)).name
        candidates = cached.get(digest, []) + cached.get(filename, [])
        data = None
        for candidate in candidates:
            contents = candidate.read_bytes()
            if hashlib.sha256(contents).hexdigest() == digest:
                data = contents
                break
        if data is None:
            data = download(wheel["url"])
            downloads += 1
        else:
            cache_hits += 1
        actual = "sha256:" + hashlib.sha256(data).hexdigest()
        if actual != wheel["hash"]:
            raise policy.ToolError(f"Collect Third-Party Notices: locked wheel hash mismatch: {package['name']} ({wheel['url']})")
        sections.append(f"\n{package['name']} {package['version']}")
        archive = zipfile.ZipFile(io.BytesIO(data))
        try:
            metadata_name = next(name for name in archive.namelist() if name.endswith(".dist-info/METADATA"))
            metadata = email.parser.BytesParser().parsebytes(archive.read(metadata_name))
            license_name = metadata.get("License-Expression") or metadata.get("License")
            if not license_name:
                license_name = "; ".join(value for value in metadata.get_all("Classifier", []) if value.startswith("License ::"))
            if not license_name:
                raise policy.ToolError(f"Collect Third-Party Notices: missing Python license metadata: {package['name']}")
            sections.append(license_name)
            files = []
            for files_name in archive.namelist():
                if files_name.endswith("/"):
                    continue
                if not (
                    ".dist-info/licenses/" in files_name
                    or Path(files_name).name.lower().startswith(("license", "licence", "copying", "notice"))
                ):
                    continue
                files.append(files_name)

            if not files:
                license_source = PYTHON_LICENSE_SOURCES.get((package["name"], package["version"]))
                if license_source is None:
                    raise policy.ToolError(f"Collect Python License for {package['name']}: Missing License Text")
                url, expected = license_source
                license_data = download(url, operation=f"Collect Python License for {package['name']}")
                actual = hashlib.sha256(license_data).hexdigest()
                if actual != expected:
                    raise policy.ToolError(f"Collect Python License for {package['name']}: Source Hash Mismatch ({url})")
                sections.append(f"License Source: {url}\n\n" + license_data.decode("utf-8"))
                license_downloads += 1
            for name in sorted(files):
                sections.append(archive.read(name).decode("utf-8"))
        finally:
            archive.close()
        count += 1
    print(f"Python wheels: {cache_hits} cache hits, {downloads} downloads; {license_downloads} pinned license sources", file=sys.stderr)
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
                if not Path(path).is_dir():
                    raise policy.ToolError(
                        f"Collect Third-Party Notices: missing shell dependency: {path}; run pnpm install in crates/dokime-gui"
                    )
                for source in sorted(Path(path).iterdir()):
                    if source.is_file() and source.name.lower().startswith(("license", "licence", "copying", "notice")):
                        texts.add(source.read_text(encoding="utf-8"))
            if not texts:
                for version in versions:
                    url = SHELL_LICENSE_SOURCES.get((package["name"], version))
                    if url is None:
                        raise policy.ToolError(f"Collect Third-Party Notices: missing shell license text: {package['name']}")
                    texts.add(f"License Source: {url}\n\n" + download(url).decode("utf-8"))
            if package["name"] == "@bufbuild/protobuf":
                for path in package["paths"]:
                    for relative in ("dist/esm/index.js", "dist/esm/wire/varint.js"):
                        header = []
                        for line in (Path(path) / relative).read_text(encoding="utf-8").splitlines():
                            if not line.startswith("//"):
                                break
                            header.append(line.removeprefix("//").lstrip())
                        if not header:
                            raise policy.ToolError(f"Collect Third-Party Notices: missing protobuf copyright header: {relative}")
                        texts.add("\n".join(header))
            sections.extend(sorted(texts))
    return "\n\n".join(sections), len(inventory)


def packaged(sources: list[Path]) -> dict[str, bytes]:
    """Read Matching Product Notices From Every Packaged Distribution."""
    result = {}
    for source in sources:
        if not source.is_file():
            raise policy.ToolError(
                f"Collect Third-Party Notices: missing packaged notices artifact: {source}; restore the matching Rust Package artifact"
            )
        bundle = zipfile.ZipFile(source)
        try:
            for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                if bundle.namelist().count(name) != 1:
                    raise policy.ToolError(
                        f"Collect Third-Party Notices: missing or duplicate {name} in package artifact {source}; rerun its Package job"
                    )
                content = bundle.read(name)
                if name in result and result[name] != content:
                    raise policy.ToolError(
                        f"Collect Third-Party Notices: packaged {name} differs between {sources[0]} and {source}; rebuild matching Package artifacts"
                    )
                result[name] = content
        finally:
            bundle.close()
    if not result:
        raise policy.ToolError("Collect Third-Party Notices: packaged notices require at least one target distribution")
    return result


def stage(directory: Path, destination: Path) -> None:
    """Ship Notices in the Installer's Installed LICENSE and Portable Layout."""
    manifest = destination / "distribution.json"
    document = json.loads(manifest.read_text(encoding="utf-8"))
    for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
        stage_file(directory / "target/notices" / name, destination / name)
        document["files"][name] = hashlib.sha256((destination / name).read_bytes()).hexdigest()
    text = (directory / "LICENSE").read_text(encoding="utf-8")
    for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
        text += "\n\n" + (directory / "target/notices" / name).read_text(encoding="utf-8")
    (destination / "LICENSE").write_text(text, encoding="utf-8", newline="\n")
    document["files"]["LICENSE"] = hashlib.sha256((destination / "LICENSE").read_bytes()).hexdigest()
    manifest.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")


def archive(directory: Path, source: Path) -> None:
    """Add Notices Before Release Hashing, Preserving Executable Modes."""
    notices = {name: (directory / name).read_bytes() for name in ("NOTICE", "THIRD-PARTY-NOTICES")}
    temporary_directory = tempfile.TemporaryDirectory(prefix="dokime-notices-")
    temporary = temporary_directory.name
    try:
        result = Path(temporary) / source.name
        if source.name.endswith(".zip"):
            original = zipfile.ZipFile(source)
            try:
                output = zipfile.ZipFile(result, "w", compression=zipfile.ZIP_DEFLATED)
                try:
                    for entry in original.infolist():
                        if entry.filename not in notices:
                            output.writestr(entry, original.read(entry))
                    for name, data in notices.items():
                        output.writestr(name, data)
                finally:
                    output.close()
            finally:
                original.close()
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
            raise policy.ToolError(f"Collect Third-Party Notices: unsupported notice archive: {source}")
        shutil.copyfile(result, source)
    finally:
        temporary_directory.cleanup()


def generate(directory: Path, *, strict: bool = True) -> dict:
    """Collect Complete License Text, Reporting Unaccepted Licenses on Nightly."""
    output = directory / "target/notices"
    output.mkdir(parents=True, exist_ok=True)
    try:
        document = sbom.exceptions(directory)
    except policy.ToolError as error:
        if strict:
            raise
        _warning(f"{error}; repair the exception register before a stable release")
        document = {}
    metadata = json.loads(policy.execute(directory, ["cargo", "metadata", "--locked", "--format-version", "1"], capture=True))
    packages = {(item["name"], item["version"]) for item in metadata["packages"]}
    licenses = {}
    for entry in document.get("license", []):
        if (entry["package"], entry["version"]) in packages:
            licenses.setdefault(entry["package"], set()).update(entry.get("accepted", [entry["license"]]))
    for name in list(licenses):
        allowed = set()
        for allowed_item in document["license"]:
            if allowed_item["package"] != name:
                continue
            allowed.add(allowed_item["version"])

        if any(package == name and version not in allowed for package, version in packages):
            message = f"Collect Third-Party Notices: license exception does not cover every resolved {name} version"
            if strict:
                raise policy.ToolError(message)
            _warning(message + "; review every resolved version before a stable release")
            del licenses[name]
    license_policy = tomllib.loads((directory / "tools/config/deny.toml").read_text(encoding="utf-8"))
    accepted = license_policy["licenses"]["allow"]
    configured = output / "about.toml"
    _configure(configured, accepted, licenses)
    command = [
        "cargo",
        "about",
        "--color",
        "never",
        "generate",
        "--locked",
        "--workspace",
        "--config",
        str(configured),
        "--format",
        "json",
        "--output-file",
        str(output / "rust.json"),
    ]
    try:
        diagnostics = policy.execute(directory, command, capture=not strict)
    except policy.ToolError as error:
        if strict:
            raise
        findings = _findings(directory, str(error), metadata["packages"])
        if not findings:
            raise
        for name, version, license_name in findings:
            _warning(f"Unaccepted Rust License: {name} {version}: {license_name}; review the license policy before a stable release")
            licenses.setdefault(name, set()).add(license_name)
        _configure(configured, accepted, licenses)
        diagnostics = policy.execute(directory, command, capture=True)
    if diagnostics:
        print(diagnostics, file=sys.stderr)
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
            "tools/config/about.hbs",
            "--output-file",
            str(output / "rust.txt"),
        ],
    )
    rust = json.loads((output / "rust.json").read_text(encoding="utf-8"))
    crates = set()
    for crates_license_ in rust["licenses"]:
        for crates_item in crates_license_["used_by"]:
            crates.add((crates_item["crate"]["name"], crates_item["crate"]["version"]))

    if not (directory / "crates/dokime-gui/node_modules").is_dir():
        raise policy.ToolError("Collect Third-Party Notices: missing shell dependencies; run pnpm install in crates/dokime-gui")
    shell = json.loads(
        policy.execute(directory, ["pnpm", "--dir", "crates/dokime-gui", "licenses", "list", "--prod", "--json"], capture=True)
    )
    shell_text, shell_count = shell_notices(shell)
    python_text, python_count = python_notices(tomllib.loads((directory / "uv.lock").read_text(encoding="utf-8")))
    text = (output / "rust.txt").read_text(encoding="utf-8") + "\n\n" + shell_text + "\n\n" + python_text + "\n"
    shutil.copyfile(directory / "NOTICE", output / "NOTICE")
    (output / "THIRD-PARTY-NOTICES").write_text(text, encoding="utf-8", newline="\n")
    counts = {"rust": len(crates), "shell": shell_count, "python": python_count}
    (output / "counts.json").write_text(json.dumps(counts, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(counts))
    return counts


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stage", type=Path)
    parser.add_argument("--profile", choices=("dev", "release"))
    parser.add_argument("--target", default="native")
    options = parser.parse_args()
    directory = policy.root()
    generate(directory, strict=os.environ.get("RELEASE_CHANNEL") != "nightly")
    destination = options.stage
    if options.profile:
        target = paths.target() if options.target == "native" else options.target
        destination = paths.output(directory, "build", options.profile, target) / "product/installer-distribution"
    if destination:
        stage(directory, destination)
    return 0


# === Private Functions === #


def _configure(destination: Path, accepted: list[str], licenses: dict[str, set[str]]) -> None:
    config = f"accepted = {json.dumps(accepted)}\n"
    for name, allowed in sorted(licenses.items()):
        config += f"\n[{json.dumps(name)}]\naccepted = {json.dumps(sorted(allowed))}\n"
    destination.write_text(config, encoding="utf-8", newline="\n")


def _findings(directory: Path, diagnostics: str, packages: list[dict]) -> list[tuple[str, str, str]]:
    """Read Cargo About's Rejected Requirement Spans, Without Interpreting SPDX Expressions."""
    manifests = {Path(item["manifest_path"]).resolve(): item for item in packages}
    findings = set()
    for block in re.split(r"(?m)^error: ", diagnostics)[1:]:
        if not block.startswith("failed to satisfy license requirements\n"):
            continue
        location = re.search(r"(?m)^\s*┌─ (.+):\d+:\d+\s*$", block)
        if location is None:
            raise policy.ToolError("Collect Third-Party Notices: cannot identify rejected crate; inspect Cargo About diagnostics\n" + block)
        manifest = (directory / location[1]).resolve()
        package = manifests.get(manifest)
        if package is None:
            raise policy.ToolError(f"Collect Third-Party Notices: rejected crate is absent from Cargo metadata; inspect {manifest}")
        requirements = set()
        for source in re.finditer(r"(?m)^\s*\d+ │ (.*)\n\s*│ ([- ^]+)$", block):
            for span in re.finditer(r"[-^]+", source[2]):
                requirements.add(source[1][span.start() : span.end()])
        if not requirements or "" in requirements:
            raise policy.ToolError(
                "Collect Third-Party Notices: cannot identify rejected license; inspect Cargo About diagnostics\n" + block
            )
        findings.update((package["name"], package["version"], requirement) for requirement in requirements)
    return sorted(findings)


def _warning(message: str) -> None:
    annotation = message.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
    print("::warning::" + annotation, file=sys.stderr)
    if destination := os.environ.get("GITHUB_STEP_SUMMARY"):
        with Path(destination).open("a", encoding="utf-8") as stream:
            stream.write("\n### Nightly Supply-Chain Warning\n\n" + message + "\n")


# === Entry Point === #

if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except policy.ToolError as error:
        print(f"notices: {error}", file=sys.stderr)
        raise SystemExit(1) from None
