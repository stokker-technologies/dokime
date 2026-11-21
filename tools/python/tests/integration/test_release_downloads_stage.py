# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Stage Tests."""

# === Imports === #

# Standard Library Imports
import json
import os
import tempfile
import unittest
import zipfile
from pathlib import Path
from typing import Any
from unittest import mock

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.package.definition
import dokime_tools.package.verification
from dokime_tools import policy
from dokime_tools.docs import site
from dokime_tools.release import (
    context,
    dry,
    layout,
    notices,
    registry,
    rehearse,
    sbom,
    staging,
    version,
)

from .environment import isolate_environment
from .release_downloads_fixtures import SETTINGS

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class StageTests(unittest.TestCase):
    def test_nightly_rehearsal_stages_git_packages_and_stable_keeps_registry_gate(self) -> None:
        settings = {"targets": ["windows-x86_64"]}
        with (
            mock.patch.object(context, "continuous_integration", return_value=False),
            mock.patch.object(context, "native_target", return_value="windows-x86_64"),
            mock.patch.object(context, "refuse_overrides"),
            mock.patch.object(context, "task_defined", return_value=True),
            mock.patch.object(policy, "execute", return_value=""),
            mock.patch.object(dry, "dry") as stage,
            mock.patch.object(registry.crates, "rehearse", side_effect=policy.ToolError("Git Dependencies Refused")) as registry_gate,
            mock.patch.object(registry.python, "publish") as python_registry,
        ):
            rehearse.rehearse(Path.cwd(), settings, channel="nightly")
            self.assertEqual(stage.call_args.kwargs["channel"], "nightly")
            registry_gate.assert_not_called()
            python_registry.assert_not_called()
            self.assertRaises(policy.ToolError, rehearse.rehearse, Path.cwd(), settings, channel="stable")
            registry_gate.assert_called_once_with(Path.cwd(), skip_published=False, channel="stable")

    def test_local_rehearsal_prepares_native_packages_without_credentials(self) -> None:
        commands: list[list[str]] = []
        settings = {"targets": ["windows-x86_64", "linux-x86_64"]}

        def execute(directory: Path, arguments: list[str]) -> str:
            self.assertNotIn("GH_TOKEN", os.environ)
            self.assertEqual(os.environ["RELEASE_PROFILE"], "dev")
            commands.append(arguments)
            return ""

        def verify_packages(*args: Any, **kwargs: Any) -> None:
            self.assertEqual(
                commands,
                [
                    ["mise", "run", "build", "--", "--language", "python", "--profile", "dev"],
                    ["mise", "run", "build", "--", "--profile", "dev"],
                    ["mise", "run", "package", "--", "--profile", "dev", "--target", "windows-x86_64"],
                ],
            )
            self.assertNotIn("GH_TOKEN", os.environ)

        with (
            mock.patch.dict(os.environ, {"GH_TOKEN": "rehearsal-secret"}),
            mock.patch.object(context, "continuous_integration", return_value=False),
            mock.patch.object(context, "native_target", return_value="windows-x86_64"),
            mock.patch.object(context, "refuse_overrides"),
            mock.patch.object(context, "task_defined", return_value=False),
            mock.patch.object(policy, "execute", side_effect=execute),
            mock.patch.object(dry, "dry", side_effect=verify_packages) as verify,
        ):
            rehearse.rehearse(Path.cwd(), settings, profile="dev")
            verify.assert_called_once()
            self.assertEqual(verify.call_args.kwargs["targets"], ["windows-x86_64"])
            self.assertEqual(os.environ["GH_TOKEN"], "rehearsal-secret")

    def test_stage_requires_packaged_notices(self) -> None:
        for missing_archive in (True, False):
            directory_directory = tempfile.TemporaryDirectory()
            directory = directory_directory.name
            try:
                root = Path(directory)
                package = root / "target/package/release/windows-x86_64/rust/dokime-distribution-0.1.3-x86_64-pc-windows-msvc.zip"
                if not missing_archive:
                    package.parent.mkdir(parents=True)
                    bundle = zipfile.ZipFile(package, "w")
                    try:
                        bundle.writestr("NOTICE", b"fixture")
                    finally:
                        bundle.close()
                with (
                    mock.patch.object(layout, "triple", return_value="x86_64-pc-windows-msvc"),
                    self.assertRaises(policy.ToolError) as failure,
                ):
                    staging.stage(root, SETTINGS, "v0.1.3", ["windows-x86_64"], profile="release", release_notes="notes")
                self.assertIn(str(package), str(failure.exception))
                if not missing_archive:
                    self.assertIn("THIRD-PARTY-NOTICES", str(failure.exception))
            finally:
                directory_directory.cleanup()

    def test_packaged_notices_must_match_between_platforms(self) -> None:
        for matching in (True, False):
            directory_directory = tempfile.TemporaryDirectory()
            directory = directory_directory.name
            try:
                sources = [Path(directory) / name for name in ("windows.zip", "linux.zip")]
                for index, source in enumerate(sources):
                    bundle = zipfile.ZipFile(source, "w")
                    try:
                        bundle.writestr("NOTICE", b"fixture")
                        bundle.writestr("THIRD-PARTY-NOTICES", b"licenses" if matching or index == 0 else b"different")
                    finally:
                        bundle.close()
                if matching:
                    self.assertEqual(notices.packaged(sources), {"NOTICE": b"fixture", "THIRD-PARTY-NOTICES": b"licenses"})
                else:
                    with self.assertRaises(policy.ToolError) as failure:
                        notices.packaged(sources)
                    for source in sources:
                        self.assertIn(str(source), str(failure.exception))
            finally:
                directory_directory.cleanup()

    def test_stage_publishes_one_index_and_exact_layout(self) -> None:
        for channel, binary in (("stable", "dokime"), ("nightly", "dokime.exe")):
            directory_directory = tempfile.TemporaryDirectory()
            directory = directory_directory.name
            try:
                root = Path(directory)
                feed = {
                    "version": "0.1.3",
                    "platforms": {"windows-x86_64": {"url": "https://example.test/setup.exe", "signature": "signed"}},
                }
                package = root / "target/package/release/windows-x86_64/rust/dokime-distribution-0.1.3-x86_64-pc-windows-msvc.zip"
                package.parent.mkdir(parents=True)
                bundle = zipfile.ZipFile(package, "w")
                try:
                    for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                        bundle.writestr(name, b"fixture\n")
                finally:
                    bundle.close()
                python = root / "target/package/release/any/python"
                python.mkdir(parents=True)
                for name in ("dokime_sdk-0.1.3-py3-none-any.whl", "dokime_sdk-0.1.3.tar.gz"):
                    (python / name).write_bytes(b"packaged Python")
                documentation = root / site.OUTPUT / site.ARCHIVE
                documentation.parent.mkdir(parents=True)
                archive = zipfile.ZipFile(documentation, "w")
                try:
                    entry = "0.1" if channel == "stable" else "dev"
                    archive.writestr(site.STAMP, json.dumps({"entry": entry, "version": "0.1.3", "revision": "a" * 40}))
                    archive.writestr("index.html", "<!doctype html>")
                finally:
                    archive.close()
                shell_inventory = root / "target/release-assets/any/dokime-shell.cdx.json"

                def artifact(_: object, source: Path, output: Path, inventories: list[dict]) -> Path:
                    destination = output / (source.name + ".cdx.json")
                    destination.write_text('{"components": []}')
                    return destination

                def shell(*_: Any, shell_inventory: Path = shell_inventory) -> Path:
                    shell_inventory.write_text('{"components": []}')
                    return shell_inventory

                protocol_sources = set()
                for protocol_sources_owner in ("dokime", "harmos"):
                    for protocol_sources_audience in ("model", "host", "plugin"):
                        protocol_sources.add(f"{protocol_sources_owner}/{protocol_sources_audience}/v1/service.proto")

                def export(_: Path, command: list[str], protocol_sources: set[str] = protocol_sources) -> None:
                    self.assertTrue(Path(command[0]).is_file())
                    exported = Path(command[-1])
                    for name in protocol_sources:
                        path = exported / name
                        path.parent.mkdir(parents=True)
                        path.write_text('syntax = "proto3";')

                    for name in ("README.md", "buf.yaml", "licenses/Apache-2.0.txt", "licenses/Protobuf.txt"):
                        path = exported / name
                        path.parent.mkdir(parents=True, exist_ok=True)
                        path.write_text("exported support file")

                source = root / "input/release.json"
                source.parent.mkdir()
                source.write_text(json.dumps(feed))
                installer = root / "input/dokime-0.1.3-x86_64-pc-windows-msvc-setup.exe"
                installer.write_bytes(b"installer")
                declared = [
                    {"name": "release.json", "path": "input/release.json", "target": "windows-x86_64"},
                    {"name": installer.name, "path": "input/" + installer.name, "target": "windows-x86_64"},
                ]
                settings = {**SETTINGS, "targets": ["windows-x86_64"]}
                config = {"package": {"executables": [{"name": "dokime-cli", "command": "dokime"}]}}
                with (
                    mock.patch.object(sbom, "shell", side_effect=shell),
                    mock.patch.object(sbom, "workspace", return_value=[]),
                    mock.patch.object(sbom, "artifact", side_effect=artifact),
                    mock.patch.object(layout, "triple", return_value="x86_64-pc-windows-msvc"),
                    mock.patch.object(staging, "assets", return_value=declared),
                    mock.patch.object(context, "native", return_value="windows-x86_64"),
                    mock.patch.object(dokime_tools.package.verification, "verify"),
                    mock.patch.object(
                        dokime_tools.package.definition,
                        "definitions",
                        return_value=[{"command": "dokime", "binary": binary, "output": "cli.zip"}],
                    ),
                    mock.patch.object(dokime_tools.package.verification, "binary_bytes", return_value=b"cli"),
                    mock.patch.object(policy, "execute", side_effect=export),
                    mock.patch.object(policy, "config", return_value=config),
                    mock.patch.object(version, "run"),
                    mock.patch.object(staging, "notes", return_value="notes"),
                    mock.patch.object(context, "release_tag", return_value="v0.1.3"),
                    mock.patch.object(context, "revision", return_value="a" * 40),
                    mock.patch.object(context, "continuous_integration", return_value=False),
                ):
                    staging.stage(root, settings, "v0.1.3", ["windows-x86_64"], profile="release", release_notes="notes\n", channel=channel)
                    if channel == "stable":
                        rehearse.rehearse(root, settings, target="windows-x86_64", profile="release", stage_only=True)
                staged = root / context.ASSETS
                document = json.loads((staged / context.SHARED / context.INDEX).read_text())
                self.assertEqual(document["platforms"], feed["platforms"])
                self.assertEqual(document["plugins"], {"format_version": 2, "plugins": []})
                manifest = document["downloads"]
                self.assertEqual((manifest["channel"], manifest["tag"]), (channel, "v0.1.3" if channel == "stable" else "nightly"))
                archive = zipfile.ZipFile(staged / context.SHARED / "dokime-compliance-0.1.3.zip")
                try:
                    self.assertIn("NOTICE", archive.namelist())
                    self.assertIn("THIRD-PARTY-NOTICES", archive.namelist())
                    self.assertIn("artifacts/" + installer.name + ".cdx.json", archive.namelist())
                    self.assertIn("crates/any/dokime-shell.cdx.json", archive.namelist())
                finally:
                    archive.close()
                inventory = {}
                for item in layout.uploads(root, staged=True):
                    inventory[item["name"]] = item
                staging.checksums(staged / context.SHARED / context.SUMS, inventory)
            finally:
                directory_directory.cleanup()

    def test_stable_publication_requires_the_documentation_of_its_release_line(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            root = Path(directory)
            shared = root / context.ASSETS / context.SHARED
            shared.mkdir(parents=True)
            self.assertIsNone(staging._documentation(root, root / context.ASSETS, "v0.2.4", "stable", required=False))
            self.assertRaises(policy.ToolError, staging._documentation, root, root / context.ASSETS, "v0.2.4", "stable", required=True)
            archive = root / site.OUTPUT / site.ARCHIVE
            archive.parent.mkdir(parents=True)
            for entry, version, channel in (("dev", "0.2.4", "stable"), ("0.2", "0.2.3", "stable"), ("0.2", "0.2.4", "nightly")):
                source = zipfile.ZipFile(archive, "w")
                try:
                    source.writestr(site.STAMP, json.dumps({"entry": entry, "version": version}))
                finally:
                    source.close()
                self.assertRaises(policy.ToolError, staging._documentation, root, root / context.ASSETS, "v0.2.4", channel, required=True)
        finally:
            directory_directory.cleanup()


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
