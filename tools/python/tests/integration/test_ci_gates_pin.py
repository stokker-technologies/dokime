# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Pin Tests."""

# === Imports === #

# Standard Library Imports
import contextlib
import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.ci import tasks
from dokime_tools.release import (
    dry,
    pins,
    registry,
    run,
    version,
)

from .ci_gates_fixtures import ROOT
from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class PinTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.directory = Path(scratch.name)

    def test_release_and_nightly(self) -> None:
        packages = [
            {"name": "direct", "source": "git+https://example.test/direct#123"},
            {"name": "transitive", "source": "git+https://example.test/transitive#456"},
        ]
        with (
            patch.object(policy, "execute", return_value=json.dumps({"packages": packages})),
            patch.dict(os.environ, {}, clear=True),
            self.assertRaises(policy.ToolError),
        ):
            pins.check(self.directory)
        with patch.object(policy, "execute", return_value=json.dumps({"packages": packages})), patch.dict(os.environ, {}, clear=True):
            self.assertFalse(pins.check(self.directory, warn=True))
        with patch.object(policy, "execute", return_value=json.dumps({"packages": [{"name": "local", "source": None}]})):
            self.assertTrue(pins.check(self.directory))

    def test_starter_git_requirement_is_rejected_without_cargo_git_sources(self) -> None:
        manifest = self.directory / "sdk/python/examples/test/pyproject.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text(
            '[project]\ndependencies = ["dokime-sdk @ git+https://example.test/sdk@nightly#subdirectory=sdk/python"]\n',
            encoding="utf-8",
        )
        with patch.object(policy, "execute", return_value='{"packages": []}'), self.assertRaises(policy.ToolError):
            pins.check(self.directory)
        with patch.object(policy, "execute", return_value='{"packages": []}'):
            self.assertFalse(pins.check(self.directory, warn=True))

    def test_python_uv_git_sources_are_rejected(self) -> None:
        for source in ('{ git = "https://example.test/sdk", tag = "nightly" }', '[{ git = "https://example.test/sdk" }]'):
            manifest = self.directory / "sdk/python/examples/plugin/pyproject.toml"
            manifest.parent.mkdir(parents=True, exist_ok=True)
            manifest.write_text(
                '[project]\ndependencies = ["dokime-sdk"]\n[tool.uv.sources]\ndokime-sdk = ' + source + "\n", encoding="utf-8"
            )
            with (
                self.subTest(source=source),
                patch.object(policy, "execute", return_value='{"packages": []}'),
                self.assertRaises(policy.ToolError),
            ):
                pins.check(self.directory)

    def test_wheel_optional_git_requirement_is_reported_without_development_groups(self) -> None:
        manifest = self.directory / "sdk/python/pyproject.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text(
            '[project.optional-dependencies]\ntransport = ["transport @ git+https://example.test/transport"]\n'
            '[dependency-groups]\nbuild = ["generator @ git+https://example.test/generator"]\n'
            '[tool.uv.sources]\ngenerator = { git = "https://example.test/generator" }\n',
            encoding="utf-8",
        )
        self.assertEqual(pins.python_git_dependencies(self.directory), {"transport (sdk/python/pyproject.toml)"})

    def test_registry_starter_is_allowed_beside_a_git_lock(self) -> None:
        manifest = self.directory / "sdk/python/examples/test/pyproject.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[project]\ndependencies = ["dokime-sdk==1.2.3"]\n', encoding="utf-8")
        lock = manifest.parent / "uv.lock"
        lock.write_text(
            '[[package]]\nname = "transport"\nsource = {git = "https://example.test/transport?rev=abc#abc"}\n', encoding="utf-8"
        )
        with patch.object(policy, "execute", return_value='{"packages": []}'):
            self.assertTrue(pins.check(self.directory))

    def test_version_sync_moves_starter_registry_requirements(self) -> None:
        # Sync Moves Each Starter's SDK Requirement, Keeping Their Own and Unrelated Versions.
        policy_file = self.directory / "tools/project.toml"
        policy_file.parent.mkdir()
        requirements = '[version.requirements]\ndokime = ["rust/Cargo.toml"]\ndokime-sdk = ["python/pyproject.toml"]\n'
        policy_file.write_text('[version]\nfiles = ["pyproject.toml"]\nvalue = "0.1.5"\n' + requirements, encoding="utf-8")
        rust = self.directory / "rust/Cargo.toml"
        python = self.directory / "python/pyproject.toml"
        for path, text in [
            (self.directory / "pyproject.toml", '[project]\nversion = "0.1.5"\n'),
            (rust, '[package]\nversion = "0.1.0"\n[dependencies]\ndokime = "0.1.5"\nmy-dokime = "0.1.5"\n'),
            (python, '[project]\nversion = "0.1.0"\ndependencies = ["dokime-sdk~=0.1.5", "rich"]\n'),
        ]:
            path.parent.mkdir(exist_ok=True)
            path.write_text(text, encoding="utf-8")
        version.run(self.directory, "check")
        version.run(self.directory, "sync", value="0.1.6")
        self.assertEqual(
            rust.read_text(encoding="utf-8"), '[package]\nversion = "0.1.0"\n[dependencies]\ndokime = "0.1.6"\nmy-dokime = "0.1.5"\n'
        )
        self.assertEqual(python.read_text(encoding="utf-8"), '[project]\nversion = "0.1.0"\ndependencies = ["dokime-sdk~=0.1.6", "rich"]\n')
        # Check Refuses a Drifted or Missing Requirement.
        policy_file.write_text('[version]\nfiles = ["pyproject.toml"]\nvalue = "0.1.6"\n' + requirements, encoding="utf-8")
        version.run(self.directory, "check")
        for text in ['dependencies = ["dokime-sdk~=0.1.5"]\n', 'dependencies = ["dokime-sdk"]\n']:
            python.write_text("[project]\n" + text, encoding="utf-8")
            with self.subTest(text=text), self.assertRaises(policy.ToolError):
                version.run(self.directory, "check")

    def test_manifest_audit_refuses_registry_requirements_on_unpublished_crates(self) -> None:
        def package(name: str, publish: list | None, *dependencies: dict) -> dict:
            manifest = str(ROOT / "crates" / name / "Cargo.toml")
            return {
                "id": name,
                "name": name,
                "publish": publish,
                "manifest_path": manifest,
                "targets": [{"kind": ["lib"]}],
                "dependencies": list(dependencies),
            }

        hint = "is unpublished; publish internal or declare it as a path-only dev-dependency"
        cases = (
            ({"name": "internal", "kind": None, "req": "^1"}, [f"library: its normal dependency internal {hint}"]),
            ({"name": "internal", "kind": "dev", "req": "^1"}, [f"library: its dev dependency internal {hint}"]),
            ({"name": "internal", "kind": "dev", "req": "*"}, []),
        )
        for dependency, expected in cases:
            metadata = {
                "workspace_members": ["library", "internal"],
                "packages": [package("library", None, dependency), package("internal", [])],
            }
            with self.subTest(dependency=dependency), patch.object(policy, "execute", return_value=json.dumps(metadata)):
                self.assertEqual(registry.crates.audit(ROOT), expected)

    def test_nightly_skips(self) -> None:
        with (
            patch.object(pins, "check", return_value=False) as check,
            patch.dict(os.environ, {"GITHUB_WORKFLOW_REF": "owner/repo/.github/workflows/nightly.yml@refs/heads/dev"}, clear=True),
        ):
            registry.crates.rehearse(ROOT, channel="nightly")
            self.assertTrue(check.call_args.kwargs["warn"])
            registry.crates.rehearse(ROOT, channel="stable")
            self.assertFalse(check.call_args.kwargs["warn"])

    def test_dry_run_channel_reaches_the_crate_dry_run(self) -> None:
        # Nightly Reports Git Pins as Pending; Every Other Caller Keeps the Stable Refusal.
        cases = (
            ({"RELEASE_CHANNEL": "nightly"}, [], "nightly"),
            ({}, [], "stable"),
            ({"RELEASE_CHANNEL": "nightly"}, ["--channel", "stable"], "stable"),
        )
        for environment, arguments, expected in cases:
            with (
                self.subTest(environment=environment, arguments=arguments),
                patch.dict(os.environ, environment, clear=True),
                patch.object(policy, "policy", return_value={}),
                patch.object(registry.crates, "rehearse") as rehearse,
            ):
                self.assertEqual(run.main(["crates-dry", *arguments]), 0)
                self.assertEqual(rehearse.call_args.kwargs["channel"], expected)
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(policy, "policy", return_value={}),
            patch.object(dry, "dry") as stage_dry_run,
            contextlib.redirect_stderr(io.StringIO()),
        ):
            self.assertEqual(run.main(["dry", "--channel", "nightly"]), 1)
            stage_dry_run.assert_not_called()
        for channel in ("stable", "nightly"):
            with (
                self.subTest(channel=channel),
                patch.dict(os.environ, {}, clear=True),
                patch.object(tasks.subprocess, "run", return_value=Mock(returncode=0)) as execute,
            ):
                self.assertEqual(tasks.main(["publish", "--dry", "--channel", channel]), 0)
                self.assertEqual(execute.call_args.args[0], ["mise", "run", "release:publish:dry", "--"])
                self.assertEqual(os.environ["RELEASE_CHANNEL"], channel)


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
