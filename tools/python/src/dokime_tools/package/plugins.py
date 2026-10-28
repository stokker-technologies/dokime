# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Package Product Plugins From Their Existing Compiler Outputs"""

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from urllib.parse import urlsplit

from dokime_tools.package.staging import clear_output

PACKAGES = ("obc-plugin",)


def prepare(root: Path, messages: Path, profile: str, *, tool: Path, output: Path) -> list[Path]:
    """Collect The Product Build's Plugin Executables Without Another Compiler."""
    root = root.resolve()
    metadata = messages.with_name("metadata.json")
    with metadata.open("w", encoding="utf-8") as written:
        subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            cwd=root,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    clear_output(output)
    output.mkdir(parents=True)
    staged: list[Path] = []
    for package in PACKAGES:
        build_json = output / f"{package}.build.json"
        collector = [str(tool), "dokime", "sidecar", "build", "--from-cargo-json", str(messages), "--metadata-json", str(metadata)]
        collector += ["--profile", profile, "--package", package, "--json"]
        with build_json.open("w", encoding="utf-8") as written:
            subprocess.run(
                collector,
                cwd=root,
                stdout=written,
                check=True,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                stdin=sys.stdin,
                stderr=sys.stderr,
            )
        packager = [str(tool), "dokime", "sidecar", "package", "--build-json", str(build_json), "--output", str(output), "--json"]
        packaged = subprocess.run(
            packager,
            cwd=root,
            capture_output=True,
            text=True,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
        ).stdout
        for item in json.loads(packaged):
            path = Path(item["path"])
            staged.append(path if path.is_absolute() else root / path)
    return staged


def release_feed(assets: list[tuple[Path, str]], *, tool: Path, requires_dokime: str) -> dict:
    """Inspect Existing Bundles And Produce One Package's Target-Specific Release Feed."""
    if not assets or len(assets) > 32 or not requires_dokime or len(requires_dokime) > 256:
        raise ValueError("a release needs bounded assets and a host version requirement")
    identity = None
    targets: set[str] = set()
    entries = []
    for path, url in assets:
        parsed = urlsplit(url)
        if parsed.scheme != "https" or not parsed.hostname or parsed.username or parsed.password or parsed.fragment:
            raise ValueError("plugin release assets require credential-free HTTPS URLs")
        inspected = subprocess.run(
            [str(tool), "dokime", "sidecar", "inspect", str(path)],
            capture_output=True,
            text=True,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
        )
        manifest = json.loads(inspected.stdout)["manifest"]
        selected = (manifest["identity"]["name"], manifest["identity"]["version"])
        if identity is not None and identity != selected:
            raise ValueError("all release assets must contain the same package and version")
        identity = selected
        executable_targets = {item["executable"]["target"] for item in manifest["files"] if item.get("executable")}
        if not executable_targets or targets.intersection(executable_targets):
            raise ValueError("release assets require distinct executable targets")
        targets.update(executable_targets)
        with path.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        entries.extend({"target": target, "url": url, "sha256": digest} for target in sorted(executable_targets))
    if len(entries) > 32:
        raise ValueError("plugin release feed exceeds the target limit")
    assert identity is not None
    return {"format_version": 1, "package": identity[0], "version": identity[1], "requires_dokime": requires_dokime, "assets": entries}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate plugins.json from inspected, already built bundles")
    parser.add_argument("--tool", required=True, type=Path)
    parser.add_argument("--asset", required=True, nargs=2, action="append", metavar=("BUNDLE", "HTTPS_URL"))
    parser.add_argument("--requires-dokime", required=True)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    feed = release_feed([(Path(path), url) for path, url in args.asset], tool=args.tool, requires_dokime=args.requires_dokime)
    args.output.write_text(json.dumps(feed, indent=2) + "\n", encoding="utf-8")
