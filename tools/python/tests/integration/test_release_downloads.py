# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify The Download Manifest Classifies Staged Uploads And Leaves The Updater Feed Alone."""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

# Third Party Imports
# Local Imports
from dokime_tools import policy
from dokime_tools.release import layout, run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

SETTINGS = {"product": "dokime", "provider": "github", "repository": "example/dokime"}
EXECUTABLES = {"dokime-cli": "cli", "cargo-dokime": "cargo-dokime"}
EXPECTED = {
    "CHANGELOG.md": ("changelog", "any", "any", "md"),
    "SHA256SUMS": ("checksums", "any", "any", "txt"),
    "cargo-dokime-0.1.3-x86_64-pc-windows-msvc.zip": ("cargo-dokime", "windows", "x86_64", "zip"),
    "cargo-dokime-0.1.3-x86_64-unknown-linux-gnu.tar.gz": ("cargo-dokime", "linux", "x86_64", "tar.gz"),
    "dokime-0.1.3-1.x86_64.rpm": ("installer", "linux", "x86_64", "rpm"),
    "dokime-0.1.3-x86_64-pc-windows-msvc-setup.exe": ("installer", "windows", "x86_64", "exe"),
    "dokime-cli-0.1.3-x86_64-pc-windows-msvc.zip": ("cli", "windows", "x86_64", "zip"),
    "dokime-cli-0.1.3-x86_64-unknown-linux-gnu.tar.gz": ("cli", "linux", "x86_64", "tar.gz"),
    "dokime-distribution-0.1.3-x86_64-pc-windows-msvc.zip": ("payload", "windows", "x86_64", "zip"),
    "dokime-distribution-0.1.3-x86_64-unknown-linux-gnu.zip": ("payload", "linux", "x86_64", "zip"),
    "dokime-guests-0.1.3.zip": ("plugin", "any", "any", "zip"),
    "dokime-sidecars-0.1.3-x86_64-pc-windows-msvc.zip": ("plugin", "windows", "x86_64", "zip"),
    "dokime-sidecars-0.1.3-x86_64-unknown-linux-gnu.tar.gz": ("plugin", "linux", "x86_64", "tar.gz"),
    "dokime_0.1.3_amd64.deb": ("installer", "linux", "x86_64", "deb"),
    "dokime_python-0.1.3-cp313-abi3-manylinux_2_35_x86_64.whl": ("sdk-python", "linux", "x86_64", "whl"),
    "dokime_python-0.1.3-cp313-abi3-win_amd64.whl": ("sdk-python", "windows", "x86_64", "whl"),
    "dokime_python-0.1.3-py3-none-any.whl": ("sdk-python", "any", "any", "whl"),
    "latest.json": ("other", "any", "any", "json"),
    "plugins.json": ("other", "any", "any", "json"),
    "unexpected.bin": ("other", "any", "any", "bin"),
}

# === Types === #

# === Classes === #


class DownloadManifestTests(unittest.TestCase):
    def manifest(self, tag: str, channel: str) -> dict:
        files = [{"name": name, "sha256": f"{index:064x}", "size": index} for index, name in enumerate(sorted(EXPECTED))]
        sums = "".join(f"{item['sha256']}  {item['name']}\n" for item in files if item["name"] != "SHA256SUMS")
        return layout.downloads(
            SETTINGS,
            EXECUTABLES,
            version="0.1.3",
            channel=channel,
            tag=tag,
            published="2026-11-08T12:00:00Z",
            files=files,
            sums=sums,
        )

    def test_every_asset_family_is_classified(self):
        manifest = self.manifest("v0.1.3", "stable")
        actual = {item["name"]: (item["kind"], item["os"], item["arch"], item["format"]) for item in manifest["artifacts"]}
        self.assertEqual(actual, EXPECTED)
        self.assertEqual(
            {key: manifest[key] for key in ("version", "channel", "tag", "pub_date")},
            {"version": "0.1.3", "channel": "stable", "tag": "v0.1.3", "pub_date": "2026-11-08T12:00:00Z"},
        )

    def test_digests_come_from_sha256sums_and_sizes_from_uploads(self):
        files = [{"name": "dokime_0.1.3_amd64.deb", "sha256": "a" * 64, "size": 7}, {"name": "SHA256SUMS", "sha256": "b" * 64, "size": 3}]
        manifest = layout.downloads(
            SETTINGS,
            EXECUTABLES,
            version="0.1.3",
            channel="stable",
            tag="v0.1.3",
            published="2026-11-08T12:00:00Z",
            files=files,
            sums=f"{'c' * 64}  dokime_0.1.3_amd64.deb\n",
        )
        digests = {item["name"]: (item["sha256"], item["size"]) for item in manifest["artifacts"]}
        self.assertEqual(digests, {"dokime_0.1.3_amd64.deb": ("c" * 64, 7), "SHA256SUMS": ("b" * 64, 3)})

    def test_unlisted_upload_is_refused(self):
        with self.assertRaises(policy.ToolError):
            layout.downloads(
                SETTINGS,
                EXECUTABLES,
                version="0.1.3",
                channel="stable",
                tag="v0.1.3",
                published="2026-11-08T12:00:00Z",
                files=[{"name": "dokime_0.1.3_amd64.deb", "sha256": "a" * 64, "size": 1}],
                sums="",
            )

    def test_urls_compose_from_the_channel_tag(self):
        for tag, channel in (("v0.1.3", "stable"), ("nightly", "nightly")):
            manifest = self.manifest(tag, channel)
            for item in manifest["artifacts"]:
                self.assertEqual(item["url"], f"https://github.com/example/dokime/releases/download/{tag}/{item['name']}")

    def test_gitlab_urls_use_release_permalinks(self):
        settings = {"product": "dokime", "provider": "gitlab", "project_id": "group/dokime", "api_url": "https://gitlab.example/api/v4"}
        self.assertEqual(
            layout.download_url(settings, "nightly", "SHA256SUMS"),
            "https://gitlab.example/group/dokime/-/releases/nightly/downloads/SHA256SUMS",
        )


class StageTests(unittest.TestCase):
    def test_stage_requires_packaged_notices(self):
        for missing_archive in (True, False):
            with self.subTest(missing_archive=missing_archive), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                package = root / "target/package/release/windows-x86_64/rust/dokime-distribution-0.1.3-x86_64-pc-windows-msvc.zip"
                if not missing_archive:
                    package.parent.mkdir(parents=True)
                    with zipfile.ZipFile(package, "w") as bundle:
                        bundle.writestr("NOTICE", b"fixture")
                with (
                    mock.patch.object(run.layout, "triple", return_value="x86_64-pc-windows-msvc"),
                    self.assertRaises(policy.ToolError) as failure,
                ):
                    run.stage(root, SETTINGS, "v0.1.3", ["windows-x86_64"], profile="release", release_notes="notes")
                self.assertIn(str(package), str(failure.exception))
                if not missing_archive:
                    self.assertIn("THIRD-PARTY-NOTICES", str(failure.exception))

    def test_packaged_notices_must_match_between_platforms(self):
        for matching in (True, False):
            with self.subTest(matching=matching), tempfile.TemporaryDirectory() as directory:
                sources = [Path(directory) / name for name in ("windows.zip", "linux.zip")]
                for index, source in enumerate(sources):
                    with zipfile.ZipFile(source, "w") as bundle:
                        bundle.writestr("NOTICE", b"fixture")
                        bundle.writestr("THIRD-PARTY-NOTICES", b"licenses" if matching or index == 0 else b"different")
                if matching:
                    self.assertEqual(run.notices.packaged(sources), {"NOTICE": b"fixture", "THIRD-PARTY-NOTICES": b"licenses"})
                else:
                    with self.assertRaises(policy.ToolError) as failure:
                        run.notices.packaged(sources)
                    for source in sources:
                        self.assertIn(str(source), str(failure.exception))

    def test_stage_publishes_downloads_beside_an_unchanged_latest_json(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            feed = b'{"version":"0.1.3","platforms":{"windows-x86_64":{"url":"u","signature":"s"}}}'
            package = root / "target/package/release/windows-x86_64/rust/dokime-distribution-0.1.3-x86_64-pc-windows-msvc.zip"
            package.parent.mkdir(parents=True)
            with zipfile.ZipFile(package, "w") as bundle:
                for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                    bundle.writestr(name, b"fixture\n")
            shell_inventory = root / "target/release-assets/any/dokime-shell.cdx.json"

            def artifact(_, source, output, inventories):
                destination = output / (source.name + ".cdx.json")
                destination.write_text('{"components": []}')
                return destination

            def shell(*_):
                shell_inventory.write_text('{"components": []}')
                return shell_inventory

            source = root / "input" / "latest.json"
            source.parent.mkdir()
            source.write_bytes(feed)
            installer = root / "input" / "dokime-0.1.3-x86_64-pc-windows-msvc-setup.exe"
            installer.write_bytes(b"installer")
            declared = [
                {"name": "latest.json", "path": "input/latest.json", "target": "windows-x86_64"},
                {"name": installer.name, "path": "input/" + installer.name, "target": "windows-x86_64"},
            ]
            settings = {**SETTINGS, "targets": ["windows-x86_64"]}
            config = {"package": {"executables": [{"name": "dokime-cli", "command": "dokime"}]}}
            with (
                mock.patch.object(run.notices, "generate", side_effect=AssertionError("staging must not generate notices")),
                mock.patch.object(run.sbom, "shell", side_effect=shell),
                mock.patch.object(run.sbom, "workspace", return_value=[]),
                mock.patch.object(run.sbom, "artifact", side_effect=artifact),
                mock.patch.object(run.layout, "triple", return_value="x86_64-pc-windows-msvc"),
                mock.patch.object(run, "assets", return_value=declared),
                mock.patch.object(run, "_native", return_value="linux-x86_64"),
                mock.patch.object(run.policy, "config", return_value=config),
            ):
                files = run.stage(
                    root, settings, "v0.1.3", ["windows-x86_64"], profile="release", release_notes="notes\n", channel="nightly"
                )
            staged = root / run.ASSETS
            for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                self.assertEqual((staged / run.SHARED / name).read_bytes(), b"fixture\n")
                self.assertFalse((root / name).exists())
            self.assertEqual((staged / "windows-x86_64" / "latest.json").read_bytes(), feed)
            sums = (staged / run.SHARED / run.SUMS).read_text(encoding="utf-8")
            self.assertNotIn(run.DOWNLOADS, sums)
            manifest = json.loads((staged / run.SHARED / run.DOWNLOADS).read_text(encoding="utf-8"))
            self.assertEqual((manifest["channel"], manifest["tag"], manifest["version"]), ("nightly", "nightly", "0.1.3"))
            listed = {item["name"]: item["sha256"] for item in manifest["artifacts"]}
            self.assertEqual(listed["latest.json"], hashlib.sha256(feed).hexdigest())
            self.assertEqual(set(listed), {item["name"] for item in files} - {run.DOWNLOADS})
            inventory = {item["name"]: item for item in layout.uploads(root, staged=True)}
            self.assertIn(run.DOWNLOADS, inventory)
            run._checksums(staged / run.SHARED / run.SUMS, inventory)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
