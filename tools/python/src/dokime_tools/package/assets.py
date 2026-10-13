# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Publish Durable Native Release Assets"""

# === Imports === #

# Standard Library Imports
import json
import os
import re
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.check.release import (
    digest,
    release_tag,
    verify_release,
    verify_source_identity,
)
from dokime_tools.package import publish

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #


# === Classes === #
class AssetRegistry(publish.Registry):
    """Original Native Packages And Installer Assets On GitHub Releases."""

    max_files = 32

    def __init__(self, api: str, project: str, token: str, version: str):
        super().__init__(api, project, token)
        if not re.fullmatch(r"\d+\.\d+\.\d+", version):
            raise ValueError("release assets require a stable version")
        self.version = version

    def owns(self, name: str) -> bool:
        return not super().owns(name)

    def asset_url(self, name: str) -> str:
        if not re.fullmatch(r"[A-Za-z0-9_.-]{1,255}", name) or name in {".", ".."}:
            raise ValueError("release asset name is invalid")
        return f"https://github.com/{self.project}/releases/download/v{self.version}/{name}"


# === Public Functions === #
def main() -> int:
    root = Path.cwd()
    receipt = root / ".stkr/state/release/assets-published.json"
    receipt.unlink(missing_ok=True)
    try:
        commit, tag = os.environ["GITHUB_SHA"], release_tag(root)
        verify_source_identity(commit, tag)
        directory = root / "dist/build"
        manifests = verify_release(directory, commit, tag, root)
        connection = (
            os.environ["GITHUB_API_URL"],
            os.environ["GITHUB_REPOSITORY"],
            os.environ["GITHUB_TOKEN"],
        )
        python = publish.Registry(*connection)
        python_artifacts = publish.artifacts(directory, manifests)
        publish.verify_inventory(python_artifacts, python.files(tag[1:]))
        registry = AssetRegistry(*connection, tag[1:])
        expected = artifacts(directory, manifests)
        for name in expected:
            registry.asset_url(name)
        publish.reconcile(expected, registry, tag[1:])
        verify_source_identity(commit, tag)
        if verify_release(directory, commit, tag, root) != manifests:
            raise ValueError("validated release manifests changed during publication")
        publish.verify_inventory(python_artifacts, python.files(tag[1:]))
        receipt.parent.mkdir(parents=True, exist_ok=True)
        receipt.write_text(
            json.dumps(
                {
                    "commit": commit,
                    "tag": tag,
                    "files": {
                        name: {
                            "sha256": item.sha256,
                            "bytes": item.size,
                            "url": registry.asset_url(name),
                        }
                        for name, item in expected.items()
                    },
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
    except (KeyError, ValueError, OSError, subprocess.SubprocessError):
        print(
            "Release asset publication incomplete; verify inputs and registry state before retrying.",
            file=sys.stderr,
        )
        return 1
    return 0


def artifacts(directory: Path, manifests: list[dict]) -> dict[str, publish.Artifact]:
    """Keep platform provenance in every durable filename, including manifests."""
    expected: dict[str, publish.Artifact] = {}
    for manifest in manifests:
        platform = manifest["platform"]
        root = directory / platform
        manifest_path = root / "manifest.json"
        entries = [
            entry
            for entry in manifest["files"]
            if entry["role"] in {"deb", "rpm", "installer", "signature", "updater"}
        ]
        entries.append(
            {
                "path": "manifest.json",
                "sha256": digest(manifest_path),
                "bytes": manifest_path.stat().st_size,
            }
        )
        catalog = json.loads(
            (directory.parent / "validation/catalog.json").read_text(encoding="utf-8")
        )
        archive = directory.parent / "downloads" / f"dokime-{platform}.zip"
        record = catalog["platforms"][platform]
        expected_archive = record["archive"]
        if (
            record["manifest_sha256"] != digest(manifest_path)
            or expected_archive["path"] != f"dist/downloads/{archive.name}"
            or archive.is_symlink()
            or archive.stat().st_size != expected_archive["bytes"]
            or digest(archive) != expected_archive["sha256"]
        ):
            raise ValueError("original native download archive changed")
        expected[archive.name] = publish.Artifact(
            archive, expected_archive["sha256"], expected_archive["bytes"]
        )
        for entry in entries:
            path = root / entry["path"]
            name = (
                f"{platform}--{path.name}"
                if path.name == "manifest.json"
                else path.name
            )
            if name in expected:
                raise ValueError("release asset filenames collide")
            expected[name] = publish.Artifact(path, entry["sha256"], entry["bytes"])
    catalog_path = directory.parent / "validation/catalog.json"
    catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
    if (
        catalog.get("inputs") != manifests[0]["inputs"]
        or catalog.get("inputs") != manifests[1]["inputs"]
    ):
        raise ValueError("release catalog differs from the validated final SHA")
    records = expected | publish.artifacts(directory, manifests)
    release_manifest = {
        "version": 1,
        "commit": manifests[0]["build_revision"],
        "tag": "v" + manifests[0]["package_version"],
        "validation": catalog,
        "files": {
            name: {"sha256": item.sha256, "bytes": item.size}
            for name, item in sorted(records.items())
        },
    }
    output = directory.parent / "release"
    output.mkdir(parents=True, exist_ok=True)
    manifest_path = output / "release-manifest.json"
    manifest_path.write_text(
        json.dumps(release_manifest, indent=2) + "\n", encoding="utf-8"
    )
    expected[manifest_path.name] = publish.Artifact(
        manifest_path, digest(manifest_path), manifest_path.stat().st_size
    )
    checksums = output / "SHA256SUMS"
    records = records | {manifest_path.name: expected[manifest_path.name]}
    checksums.write_text(
        "".join(f"{item.sha256}  {name}\n" for name, item in sorted(records.items())),
        encoding="utf-8",
    )
    expected[checksums.name] = publish.Artifact(
        checksums, digest(checksums), checksums.stat().st_size
    )
    if not expected or len(expected) > AssetRegistry.max_files:
        raise ValueError("release asset inventory exceeds its bounds")
    return expected


# === Private Functions === #

# === Entry Point === #
if __name__ == "__main__":
    raise SystemExit(main())
