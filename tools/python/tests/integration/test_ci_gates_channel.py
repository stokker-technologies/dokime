# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Channel Tests."""

# === Imports === #

# Standard Library Imports
import copy
import json
import os
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.package.definition
import dokime_tools.package.installation
import dokime_tools.package.verification
from dokime_tools import policy
from dokime_tools.release import (
    context,
    layout,
    nightly,
    registry,
    staging,
    verify,
)

from .ci_gates_fixtures import LATER, NEW, OLD, FakeGitHub, _staged_files
from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ChannelTests(unittest.TestCase):
    def test_build_line_preserves_links_consumption_and_previous_changes(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        root = Path(scratch.name)
        # Notes No Longer Read Dependency Provenance or the Product Version.
        (root / "Cargo.toml").write_text("not a manifest", encoding="utf-8")
        for provider in ("github", "gitlab"):
            settings = {"provider": provider, "repository": "owner/product", "project_id": "owner/product", "product": "product"}
            project = "https://github.com/owner/product" if provider == "github" else "https://gitlab.com/owner/product"
            commit = project + ("/commit/" if provider == "github" else "/-/commit/")
            source_run = "[run 9](https://example.test/runs/9)"

            # Generate Both Builds Without Reading Provenance Inputs.
            with patch.object(nightly.policy, "execute") as execute, patch.object(nightly.policy, "policy") as configuration:
                previous = nightly._notes(root, settings, OLD, "[run 8](https://example.test/runs/8)", "")
                notes = nightly._notes(root, settings, NEW, source_run, "")
                execute.assert_not_called()
                configuration.assert_not_called()
            metadata = notes.split("\n\n")[1]
            self.assertEqual(metadata, f"Built from [`{NEW[:8]}`]({commit}{NEW}) · {source_run}")
            self.assertFalse(any(line.startswith("|") for line in notes.splitlines()))
            self.assertIn('product = { git = "' + project + '", branch = "dev" }', notes)
            if provider == "github":
                self.assertIn('[tools."github:owner/product"]', notes)
            else:
                self.assertIn(f"{project}/-/releases/nightly", notes)

            # Keep History and Newer-Run Protection on the New Metadata Format.
            with patch.object(nightly.policy, "execute", side_effect=[OLD, "- fix: preserved change"]) as execute:
                updated = nightly._notes(root, settings, NEW, source_run, previous)
            self.assertEqual(execute.call_args_list[0].args[1], ["git", "rev-parse", "--verify", "--quiet", OLD + "^{commit}"])
            self.assertEqual(execute.call_args_list[1].args[1], ["git", "log", "--format=- %s", "-20", f"{OLD}..{NEW}"])
            self.assertIn("- fix: preserved change", updated)
            field = "body" if provider == "github" else "description"
            self.assertTrue(nightly._superseded({field: updated}, "7", field))
            self.assertFalse(nightly._superseded({field: updated}, "9", field))
            self.assertFalse(nightly._superseded({field: updated}, "10", field))
            self.assertFalse(nightly._superseded(None, "7", field))

    def test_nightly_installation_uses_mise_mutable_version(self) -> None:
        definition = {
            "installers": ["mise", "cargo-binstall"],
            "output": "dokime.zip",
            "binary": "dokime.exe",
            "verify": ["--version"],
        }

        def install(command: list[str], **kwargs: Any) -> None:
            destination = Path(command[-1])
            destination.mkdir(parents=True, exist_ok=True)
            (destination / "dokime.exe").write_bytes(b"executable")

        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            with (
                patch.object(dokime_tools.package.definition, "definitions", return_value=[definition]),
                patch.object(dokime_tools.package.verification, "binary_bytes", return_value=b"executable"),
                patch.object(
                    dokime_tools.package.installation.subprocess,
                    "run",
                    side_effect=lambda command, **kwargs: install(command) if command[0] == "mise" else None,
                ) as execute,
            ):
                settings = {"provider": "github", "repository": "example/dokime", "tag": "nightly"}
                dokime_tools.package.installation.installed(Path(temporary), settings, Path(temporary), "windows-x86_64")
                commands = [call.args[0] for call in execute.call_args_list]
                self.assertEqual(len(commands), 2)
                self.assertEqual(commands[0][0:2], ["mise", "install-into"])
                self.assertTrue(commands[0][2].endswith("@nightly"))
                self.assertEqual(commands[1][1:], ["--version"])
        finally:
            temporary_directory.cleanup()

    def test_shared_verifier_preserves_channel_and_signing_policy(self) -> None:
        for channel in ("stable", "nightly"):
            tag = "v0.1.5" if channel == "stable" else "nightly"
            remote = {"id": 1, "draft": False, "prerelease": channel == "nightly", "target_commitish": NEW}
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with (
                    self.subTest(channel=channel),
                    patch.object(context, "release_tag", return_value="v0.1.5"),
                    patch.object(context, "revision", return_value=NEW),
                    patch.object(registry.provider, "remote_tag", return_value=NEW),
                    patch.object(registry.provider, "document", return_value=remote),
                    patch.object(registry.provider, "pages", return_value=[{"name": "SHA256SUMS", "id": 2}]),
                    patch.object(registry.provider, "verify_download", return_value=b"checksums") as download,
                    patch.object(registry.provider, "verify_signatures") as signatures,
                    patch.object(layout, "uploads", return_value=[{"name": "SHA256SUMS"}]),
                    patch.object(staging, "checksums"),
                    patch.object(context, "native_target", return_value="windows-x86_64"),
                    patch.object(dokime_tools.package.installation, "installed") as installed,
                ):
                    root = Path(temporary)
                    verify.verify(root, {"provider": "github"}, channel=channel)
                    self.assertEqual(download.call_args.kwargs["channel"], channel == "nightly")
                    self.assertEqual(signatures.call_count, int(channel == "stable"))
                    self.assertEqual(installed.call_args.args[1]["tag"], tag)
                    state = json.loads((root / "target/state/release-publication.json").read_text())
                    self.assertEqual(state, {"sha": NEW, "tag": tag, "verified": True})
                    remote["prerelease"] = not remote["prerelease"]
                    with self.assertRaises(policy.ToolError):
                        verify.verify(root, {"provider": "github"}, channel=channel)
            finally:
                temporary_directory.cleanup()

    def test_nightly_source(self) -> None:
        base = {"GITHUB_REPOSITORY": "owner/repo", "GITHUB_RUN_ID": "42", "GITHUB_SHA": "a" * 40, "GITHUB_SERVER_URL": "https://github.com"}
        expected = ("a" * 40, "42", "https://github.com/owner/repo/actions/runs/42")
        for event in ("schedule", "workflow_dispatch"):
            environment = {**base, "GITHUB_EVENT_NAME": event, "GITHUB_REF": "refs/heads/dev"}
            with self.subTest(event=event), patch.dict(os.environ, environment, clear=True):
                self.assertEqual(nightly._source(True), expected)

        for event, ref in (("workflow_dispatch", "refs/heads/feature"), ("push", "refs/heads/dev")):
            environment = {**base, "GITHUB_EVENT_NAME": event, "GITHUB_REF": ref}
            with self.subTest(event=event, ref=ref), patch.dict(os.environ, environment, clear=True), self.assertRaises(policy.ToolError):
                nightly._source(True)

    def test_publication_replaces_previous(self) -> None:
        for previous in (True, False):
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(previous=previous):
                    directory, files = _staged_files(Path(temporary))
                    github = FakeGitHub(previous=previous)
                    github.releases[3] = {"id": 3, "name": "v0.1.2", "tag_name": "v0.1.2", "draft": False, "prerelease": False}
                    github.releases[4] = {"id": 4, "name": nightly.TITLE, "tag_name": "untagged-4", "draft": True, "prerelease": True}
                    github.assets.update({3: [], 4: []})
                    with github.patched():
                        nightly._github(directory, {}, files, NEW, "7", "notes")
                    published = github.published()
                    self.assertEqual(published["target_commitish"], NEW)
                    self.assertTrue(published["prerelease"])
                    self.assertEqual(github.tag, NEW)
                    self.assertEqual({asset["name"] for asset in github.assets[published["id"]]}, {item["name"] for item in files})
                    self.assertEqual(set(github.releases), {3, published["id"]})
            finally:
                temporary_directory.cleanup()

    def test_failure_before_swap_keeps_previous(self) -> None:
        for fault in ("create", "upload:linux.zip", "upload:SHA256SUMS", "size", "pending", "digest", "download"):
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(fault=fault):
                    directory, files = _staged_files(Path(temporary))
                    github = FakeGitHub(previous=True, faults={fault})
                    before = copy.deepcopy((github.releases, github.assets, github.tag))
                    with github.patched(), self.assertRaises(policy.ToolError):
                        nightly._github(directory, {}, files, NEW, "7", "notes")
                    self.assertEqual((github.releases, github.assets, github.tag), before)
            finally:
                temporary_directory.cleanup()

    def test_failure_during_swap_restores_previous(self) -> None:
        for fault in ("tag-create", "draft-previous", "publish", "lookup"):
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(fault=fault):
                    directory, files = _staged_files(Path(temporary))
                    github = FakeGitHub(previous=True, faults={fault})
                    before = copy.deepcopy((github.releases, github.assets, github.tag))
                    with github.patched(), self.assertRaises(policy.ToolError):
                        nightly._github(directory, {}, files, NEW, "7", "notes")
                    self.assertEqual((github.releases, github.assets, github.tag), before)
            finally:
                temporary_directory.cleanup()

    def test_superseded_and_immutable_previous(self) -> None:
        for state in ("superseded", "immutable"):
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(state=state):
                    directory, files = _staged_files(Path(temporary))
                    github = FakeGitHub(previous=True)
                    if state == "superseded":
                        github.releases[1]["body"] = (
                            f"Built from [`{LATER[:8]}`](https://example.invalid/commit/{LATER}) · [run 9](https://example.invalid/9)"
                        )
                    else:
                        github.releases[1]["immutable"] = True
                    before = copy.deepcopy((github.releases, github.assets, github.tag))
                    with github.patched():
                        if state == "superseded":
                            nightly._github(directory, {}, files, NEW, "7", "notes")
                        else:
                            with self.assertRaisesRegex(policy.ToolError, "Enable release immutability"):
                                nightly._github(directory, {}, files, NEW, "7", "notes")
                    self.assertEqual((github.releases, github.assets, github.tag), before)
            finally:
                temporary_directory.cleanup()

    def test_dev_workflow_change_supersedes(self) -> None:
        for workflows, published in (("changed", False), ("same", True)):
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(workflows=workflows):
                    directory, files = _staged_files(Path(temporary))
                    github = FakeGitHub(previous=True)
                    github.dev = LATER
                    github.workflows[LATER] = "changed-workflows" if workflows == "changed" else github.workflows[NEW]
                    before = copy.deepcopy((github.releases, github.assets, github.tag))
                    with github.patched():
                        nightly._github(directory, {}, files, NEW, "7", "notes")
                    if published:
                        self.assertEqual(github.published()["target_commitish"], NEW)
                    else:
                        self.assertEqual((github.releases, github.assets, github.tag), before)
            finally:
                temporary_directory.cleanup()

    def test_repository_immutability_is_reported(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            directory, files = _staged_files(Path(temporary))
            github = FakeGitHub(previous=True, immutable=True)
            with github.patched(), self.assertRaisesRegex(policy.ToolError, "Enable release immutability"):
                nightly._github(directory, {}, files, NEW, "7", "notes")
            self.assertEqual(github.published()["target_commitish"], NEW)
        finally:
            temporary_directory.cleanup()


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
