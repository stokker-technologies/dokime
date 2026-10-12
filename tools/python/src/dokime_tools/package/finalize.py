# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Finalize Verified Native Releases"""

# === Imports === #

# Standard Library Imports
import argparse
import base64
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from urllib.error import HTTPError, URLError

# Third Party Imports

# Local Imports
from dokime_tools.check.release import (
    release_tag,
    verify_release,
    verify_source_identity,
)
from dokime_tools.package import assets, publish

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #


# === Classes === #
class ReleaseRegistry(assets.AssetRegistry):
    def finalize(
        self,
        tag: str,
        commit: str,
        description: str,
        expected: dict[str, publish.Artifact],
        *,
        all_artifacts: dict[str, publish.Artifact] | None = None,
    ) -> None:
        self.verify_tag(tag, commit)
        if not self.version:
            raise ValueError("release version is missing")
        version = self.version
        remote = self.release(version)
        if remote is None:
            raise ValueError("verified release assets have no native release")
        payload = {
            "tag_name": tag,
            "name": f"Dokime {tag}",
            "body": description,
            "draft": False,
            "prerelease": False,
        }
        all_artifacts = expected if all_artifacts is None else all_artifacts
        verify_downloads(self, remote, all_artifacts, Path.cwd())
        if remote.get("draft") is True:
            try:
                self.request("PATCH", f"/releases/{remote['id']}", payload)
            except (HTTPError, URLError, TimeoutError):
                pass  # Reconcile Uncertain Publication Through The Native API.
            remote = self.release(version)
        confirm(remote, payload, expected, self)
        if remote is None:
            raise ValueError("published release is missing")
        verify_downloads(self, remote, all_artifacts, Path.cwd())
        self.verify_tag(tag, commit)


# === Public Functions === #
def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify-only", action="store_true")
    args = parser.parse_args()
    root = Path.cwd()
    receipt = root / ".stkr/state/release/release-published.json"
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
        registry = ReleaseRegistry(*connection, tag[1:])
        python_artifacts = publish.artifacts(directory, manifests)
        expected = assets.artifacts(directory, manifests)
        publish.verify_inventory(python_artifacts, python.files(tag[1:]))
        publish.verify_inventory(expected, registry.files(tag[1:]))
        description = release_notes(tag)
        verify_source_identity(commit, tag)
        if verify_release(directory, commit, tag, root) != manifests:
            raise ValueError("validated release manifests changed during publication")
        if args.verify_only:
            remote = registry.release(tag[1:])
            confirm(
                remote,
                {"tag_name": tag, "draft": False, "prerelease": False},
                expected,
                registry,
            )
            if remote is None:
                raise ValueError("published release is missing")
            registry.verify_tag(tag, commit)
            verify_downloads(registry, remote, expected | python_artifacts, root)
        else:
            registry.finalize(
                tag,
                commit,
                description,
                expected,
                all_artifacts=expected | python_artifacts,
            )
        publish.verify_inventory(python_artifacts, python.files(tag[1:]))
        publish.verify_inventory(expected, registry.files(tag[1:]))
        verify_source_identity(commit, tag)
        if verify_release(directory, commit, tag, root) != manifests:
            raise ValueError("validated release manifests changed during publication")
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
                        for name, item in (expected | python_artifacts).items()
                    },
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
    except (KeyError, TypeError, ValueError, OSError, subprocess.SubprocessError):
        print(
            "Release finalization incomplete; verify tagged sources and remote inventories before retrying.",
            file=sys.stderr,
        )
        return 1
    return 0


def confirm(
    remote: dict | None,
    expected: dict,
    artifacts: dict[str, publish.Artifact],
    registry: ReleaseRegistry,
) -> None:
    if remote is None or any(
        remote.get(key) != value for key, value in expected.items()
    ):
        raise ValueError(
            "existing release identity or notes conflict with the validated release"
        )
    assets = remote.get("assets")
    if not isinstance(assets, list):
        raise ValueError("existing release assets are invalid")
    actual = {}
    for item in assets:
        name = item.get("name")
        if not isinstance(name, str) or not registry.owns(name):
            continue
        if name in actual or item.get("browser_download_url") != registry.asset_url(
            name
        ):
            raise ValueError("existing release asset download identity is invalid")
        actual[name] = (
            item.get("digest", "").removeprefix("sha256:"),
            item.get("size"),
        )
    publish.verify_inventory(artifacts, actual)


def verify_downloads(
    registry: ReleaseRegistry,
    remote: dict,
    expected: dict[str, publish.Artifact],
    root: Path,
) -> None:
    inventory = remote.get("assets")
    if not isinstance(inventory, list) or len(inventory) != len(expected):
        raise ValueError("remote release contains missing or unowned assets")
    native = {item["name"]: item for item in inventory}
    if len(native) != len(inventory) or set(native) != set(expected):
        raise ValueError("remote release asset names differ")
    with tempfile.TemporaryDirectory(prefix="dokime-release-") as temporary:
        downloads = Path(temporary)
        for name, item in expected.items():
            registry.download(native[name], item, downloads / name)
        verify_signature(downloads, root)


def verify_signature(downloads: Path, root: Path) -> None:
    metadata = json.loads(
        (downloads / "windows--manifest.json").read_text(encoding="utf-8")
    )
    roles = {entry["role"]: entry for entry in metadata["files"]}
    if not {"signature", "installer", "updater"} <= roles.keys():
        raise ValueError("publication requires signed Windows updater bytes")
    installer = downloads / Path(roles["installer"]["path"]).name
    signature = downloads / Path(roles["signature"]["path"]).name
    config = json.loads(
        (root / "crates/dokime-gui/src-tauri/tauri.conf.json").read_text(
            encoding="utf-8"
        )
    )
    public_key = downloads / "minisign.pub"
    public_key.write_bytes(
        base64.b64decode(config["plugins"]["updater"]["pubkey"], validate=True)
    )
    decoded = downloads / "installer.minisig"
    decoded.write_bytes(
        base64.b64decode(signature.read_text(encoding="utf-8").strip(), validate=True)
    )
    subprocess.run(
        ["minisign", "-Vm", str(installer), "-p", str(public_key), "-x", str(decoded)],
        cwd=root,
        check=True,
        timeout=60,
    )


def release_notes(tag: str) -> str:
    result = subprocess.run(
        [
            "git-cliff",
            "--config",
            "cliff.toml",
            "--current",
            "--tag",
            tag,
            "--strip",
            "header",
        ],
        check=True,
        capture_output=True,
        text=True,
        timeout=60,
        env={
            key: value
            for key, value in os.environ.items()
            if not key.startswith("GIT_CLIFF_")
        },
        encoding="utf-8",
    )
    notes = result.stdout.strip()
    if not notes or len(notes.encode()) > 1024 * 1024:
        raise ValueError("release notes are empty or exceed the publication budget")
    return notes


# === Private Functions === #

# === Entry Point === #
if __name__ == "__main__":
    raise SystemExit(main())
