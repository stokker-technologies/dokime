# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prove Documentation Entries, Link Checks and Site Image Assembly."""

# === Imports === #

# Standard Library Imports
import json
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.docs import image, site

# Type Checking Imports


# === Public Exports === #

# === Constants === #

SETTINGS = {"product": "dokime", "provider": "github", "repository": "example/dokime"}

# === Types === #

# === Classes === #


class DocumentationSiteTests(unittest.TestCase):
    def archive(self, root: Path, entry: str, version: str) -> Path:
        path = root / f"{entry}-{version}.zip"
        source = zipfile.ZipFile(path, "w")
        try:
            source.writestr(site.STAMP, json.dumps({"entry": entry, "version": version, "revision": "a" * 40}))
            source.writestr("index.html", "<!doctype html>")
            source.writestr("guide/index.html", "<!doctype html>")
        finally:
            source.close()
        return path

    def test_release_entry_is_the_release_line(self) -> None:
        with mock.patch.object(site.policy, "policy", return_value={"value": "1.12.3"}):
            self.assertEqual(site.entry(Path.cwd(), release=True), "1.12")
            self.assertEqual(site.entry(Path.cwd(), release=False), "dev")
        with mock.patch.object(site.policy, "policy", return_value={"value": "1.12.3-rc.1"}), self.assertRaises(policy.ToolError):
            site.entry(Path.cwd(), release=True)

    def test_links_report_missing_pages_and_headings_only(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            output = Path(directory)
            (output / "guide").mkdir()
            (output / "guide/index.html").write_text('<h2 id="setup">Setup</h2>', encoding="utf-8")
            (output / "guides/plugins").mkdir(parents=True)
            (output / "sdks/rust").mkdir(parents=True)
            (output / "sdks/rust/index.html").write_text('<h2 id="setup">Setup</h2>', encoding="utf-8")
            (output / "guides/plugins/index.html").write_text(
                '<a href="../../sdks/rust/#setup"></a><a href="/dev/sdks/rust/#setup"></a>', encoding="utf-8"
            )
            (output / "index.html").write_text(
                '<a href="/dev/guide/#setup"></a><a href="guide/"></a><a href="https://example.test/x"></a>'
                '<a href="/dev/guide/#missing"></a><a href="/dev/absent/"></a><a href="/0.1/guide/"></a>',
                encoding="utf-8",
            )
            broken = site.links(output, "/dev/")
            self.assertEqual(len(broken), 3)
            self.assertTrue(all(item.startswith("index.html: ") for item in broken))
        finally:
            directory_directory.cleanup()

    def test_assembly_orders_entries_and_redirects_latest_to_the_newest_line(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            root = Path(directory)
            archives = [self.archive(root, "0.1", "0.1.4"), self.archive(root, "dev", "0.2.4"), self.archive(root, "0.2", "0.2.3")]
            output = root / "image"
            entries = image.assemble(archives, output)
            self.assertEqual([item["entry"] for item in entries], ["latest", "0.2", "0.1", "dev"])
            self.assertEqual(json.loads((output / "versions.json").read_text(encoding="utf-8")), entries)
            for entry in ("0.1", "0.2", "dev"):
                self.assertTrue((output / entry / "guide/index.html").is_file())
            self.assertIn('url=/0.2/guide/"', (output / "latest/guide/index.html").read_text(encoding="utf-8"))
            self.assertIn('url=/latest/"', (output / "index.html").read_text(encoding="utf-8"))
        finally:
            directory_directory.cleanup()

    def test_assembly_without_releases_lands_on_development(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            root = Path(directory)
            entries = image.assemble([self.archive(root, "dev", "0.2.4")], root / "image")
            self.assertEqual([item["entry"] for item in entries], ["dev"])
            self.assertFalse((root / "image/latest").exists())
            self.assertIn('url=/dev/"', (root / "image/index.html").read_text(encoding="utf-8"))
        finally:
            directory_directory.cleanup()

    def test_assembly_refuses_duplicate_lines_and_foreign_entries(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            root = Path(directory)
            cases = (
                [self.archive(root, "0.2", "0.2.3"), self.archive(root, "0.2", "0.2.2")],
                [self.archive(root, "0.3", "0.2.3")],
                [self.archive(root, "dev", "0.2.3"), self.archive(root, "dev", "0.2.4")],
            )
            for archives in cases:
                with self.subTest(archives=[path.name for path in archives]), self.assertRaises(policy.ToolError):
                    image.assemble(archives, root / "image")
        finally:
            directory_directory.cleanup()

    def test_default_image_combines_the_current_archive_with_published_stable_lines(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary.name)
            output = root / site.OUTPUT
            output.mkdir(parents=True)
            current = self.archive(output, "dev", "0.3.1")
            current.rename(output / site.ARCHIVE)
            stable = self.archive(root, "0.2", "0.2.7")
            with (
                mock.patch.object(image.policy, "root", return_value=root),
                mock.patch.object(image.policy, "execute", return_value="a" * 40),
                mock.patch.object(image, "download", return_value=[stable]) as download,
            ):
                self.assertEqual(image.main([]), 0)
                download.assert_called_once_with(root)
            entries = json.loads((output / "image/versions.json").read_text())
            self.assertEqual([entry["entry"] for entry in entries], ["latest", "0.2", "dev"])
            stamp = json.loads((output / "image/dev/docs.json").read_text())
            self.assertEqual(stamp["revision"], "a" * 40)
            self.assertTrue((output / image.IMAGE).is_file())
        finally:
            temporary.cleanup()

    def test_default_image_refuses_missing_stale_or_release_archives_before_downloading(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary.name)
            output = root / site.OUTPUT
            output.mkdir(parents=True)
            for entry, revision in ((None, "a"), ("dev", "b"), ("0.2", "a")):
                if entry is not None:
                    source = self.archive(output, entry, "0.2.3")
                    source.replace(output / site.ARCHIVE)
                with (
                    self.subTest(entry=entry),
                    mock.patch.object(image.policy, "root", return_value=root),
                    mock.patch.object(image.policy, "execute", return_value=revision * 40),
                    mock.patch.object(image, "download") as download,
                ):
                    self.assertEqual(image.main([]), 1)
                    download.assert_not_called()
        finally:
            temporary.cleanup()

    def test_download_selects_stable_lines_without_the_published_development_entry(self) -> None:
        def release(tag: str, *, prerelease: bool = False, draft: bool = False, name: str | None = None) -> dict:
            asset = name or f"dokime-docs-{tag.removeprefix('v')}.zip"
            return {"tag_name": tag, "prerelease": prerelease, "draft": draft, "assets": [{"name": asset}]}

        releases = [
            release("v0.2.3"),
            release("v0.2.10"),
            release("v0.3.0", prerelease=True),
            release("v0.4.0", draft=True),
            release("v0.1.7", name="dokime-0.1.7.zip"),
            release("v0.1.6"),
            release("nightly", prerelease=True, name="dokime-docs-nightly.zip"),
        ]
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            with (
                mock.patch.object(image.policy, "policy", return_value=SETTINGS),
                mock.patch.object(image.registry.provider, "pages", return_value=releases),
                mock.patch.object(image.notices, "download", return_value=b"archive") as download,
            ):
                archives = image.download(Path(directory))
                self.assertEqual(sorted(path.name for path in archives), ["dokime-docs-0.1.6.zip", "dokime-docs-0.2.10.zip"])
                self.assertEqual(download.call_count, 2)
        finally:
            directory_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
