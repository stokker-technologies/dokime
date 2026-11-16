# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Download Manifest Tests."""

# === Imports === #

# Standard Library Imports
import unittest

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.release import (
    layout,
)

from .environment import isolate_environment
from .release_downloads_fixtures import EXECUTABLES, EXPECTED, SETTINGS

# Type Checking Imports

# === Public Exports === #

# === Constants === #

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

    def test_every_asset_family_is_classified(self) -> None:
        manifest = self.manifest("v0.1.3", "stable")
        actual = {item["name"]: (item["kind"], item["os"], item["arch"], item["format"]) for item in manifest["artifacts"]}
        self.assertEqual(actual, EXPECTED)
        self.assertEqual(
            {key: manifest[key] for key in ("version", "channel", "tag", "pub_date")},
            {"version": "0.1.3", "channel": "stable", "tag": "v0.1.3", "pub_date": "2026-11-08T12:00:00Z"},
        )

    def test_digests_come_from_sha256sums_and_sizes_from_uploads(self) -> None:
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

    def test_unlisted_upload_is_refused(self) -> None:
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

    def test_urls_compose_from_the_channel_tag(self) -> None:
        for tag, channel in (("v0.1.3", "stable"), ("nightly", "nightly")):
            manifest = self.manifest(tag, channel)
            for item in manifest["artifacts"]:
                self.assertEqual(item["url"], f"https://github.com/example/dokime/releases/download/{tag}/{item['name']}")

    def test_gitlab_urls_use_release_permalinks(self) -> None:
        settings = {"product": "dokime", "provider": "gitlab", "project_id": "group/dokime", "api_url": "https://gitlab.example/api/v4"}
        self.assertEqual(
            layout.download_url(settings, "nightly", "SHA256SUMS"),
            "https://gitlab.example/group/dokime/-/releases/nightly/downloads/SHA256SUMS",
        )


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
