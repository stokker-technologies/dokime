# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Mechanical Test Host Fixtures."""

# === Imports === #

# Standard Library Imports
import io
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import desktop_fixtures, host_fixtures, paths

# Type Checking Imports

# === Public Exports === #


# === Constants === #


# === Types === #

if __name__ == "__main__":
    unittest.main()

# === Classes === #


class HostFixtureTests(unittest.TestCase):
    def test_desktop_bundle_failure_keeps_tool_diagnostics(self) -> None:
        diagnostic = "fixture stderr\n"
        refusal = subprocess.CompletedProcess(["dokime"], 3, stderr=diagnostic)
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            with (
                patch.object(desktop_fixtures.configuration, "ROOT", Path(temporary)),
                patch.object(desktop_fixtures.subprocess, "run", side_effect=[subprocess.CompletedProcess(["mise"], 0), refusal]),
                patch.object(desktop_fixtures.shutil, "copy2"),
                patch.object(desktop_fixtures.sys, "stderr", new_callable=io.StringIO) as stderr,
                self.assertRaises(subprocess.CalledProcessError) as failure,
            ):
                desktop_fixtures.build("release")
        finally:
            temporary_directory.cleanup()
        self.assertEqual(failure.exception.returncode, refusal.returncode)
        self.assertEqual(stderr.getvalue(), diagnostic)

    def test_explicit_producers_build_only_the_selected_language(self) -> None:
        for language in ("rust", "python", "csharp", "cpp", "none"):
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(language=language):
                    root = Path(temporary)
                    (root / "Cargo.toml").write_text(
                        '[workspace.dependencies.harmos]\ngit = "https://example.test/library"\nrev = "fixture-revision"\n',
                        encoding="utf-8",
                    )
                    fixture = root / "library/sdk/python/tests/fixtures/conformance/__init__.py"
                    fixture.parent.mkdir(parents=True)
                    fixture.write_bytes(b"fixture source")
                    with (
                        patch.object(host_fixtures.configuration, "ROOT", root),
                        patch.object(host_fixtures.subprocess, "run") as run,
                        patch.object(host_fixtures.shutil, "copy2"),
                        patch.object(host_fixtures.archive, "stamp") as stamp,
                    ):
                        run.return_value.stdout = json.dumps(
                            {
                                "packages": [
                                    {
                                        "name": "harmos",
                                        "source": "git+https://example.test/library#fixture-revision",
                                        "manifest_path": str(root / "library/sdk/rust/Cargo.toml"),
                                    }
                                ]
                            }
                        )
                        host_fixtures.build("release", language)
                        commands = [call.args[0] for call in run.call_args_list]
                        self.assertEqual(
                            {command[2] for command in commands if command[1] == "-C"},
                            set() if language == "none" else {f"sdk/{language}"},
                        )
                        stamp.assert_not_called()
                        if language == "cpp":
                            self.assertEqual(commands[0][-1], "prepare")
                            self.assertIn("dokime-system-suite", commands[1])
                        if language == "csharp":
                            self.assertEqual([command[4] for command in commands], ["build", "package", "publish:system"])
                        if language == "rust":
                            staged = paths.output(root, "build", "release") / "fixtures/host/python/conformance/__init__.py"
                            self.assertEqual(staged.read_bytes(), fixture.read_bytes())
                            reference = next(command for command in commands if command[:2] == ["cargo", "install"])
                            self.assertEqual(reference[reference.index("--git") + 1], "https://example.test/library")
                            self.assertEqual(reference[reference.index("--rev") + 1], "fixture-revision")
            finally:
                temporary_directory.cleanup()

    def test_local_default_builds_present_toolchains_and_preserves_inventory(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            (root / "Cargo.toml").write_text(
                '[workspace.dependencies.harmos]\ngit = "https://example.test/library"\nrev = "fixture-revision"\n', encoding="utf-8"
            )
            fixture = root / "library/sdk/python/tests/fixtures/conformance/__init__.py"
            fixture.parent.mkdir(parents=True)
            fixture.write_bytes(b"fixture source")
            output = paths.output(root, "build", "dev") / "fixtures/host"
            output.mkdir(parents=True)
            (output / "fixture").write_bytes(b"fixture")
            stamp = output.parent / "stamp.json"
            stamp.write_text('{"files":{"existing":"digest"},"metadata":{"owner":"tests"}}')
            with (
                patch.object(host_fixtures.configuration, "ROOT", root),
                patch.object(host_fixtures.subprocess, "run") as run,
                patch.object(host_fixtures.shutil, "copy2"),
                patch.object(host_fixtures.shutil, "which", side_effect=lambda name: name if name == "dotnet" else None),
                patch.object(host_fixtures.archive, "stamp") as inventory,
            ):
                run.return_value.stdout = json.dumps(
                    {
                        "packages": [
                            {
                                "name": "harmos",
                                "source": "git+https://example.test/library#fixture-revision",
                                "manifest_path": str(root / "library/sdk/rust/Cargo.toml"),
                            }
                        ]
                    }
                )
                host_fixtures.build("dev")
                self.assertEqual(
                    {call.args[0][2] for call in run.call_args_list if call.args[0][1] == "-C"}, {"sdk/rust", "sdk/csharp", "sdk/python"}
                )
                expected = [
                    "existing",
                    (output / "fixture").relative_to(root).as_posix(),
                    (output / "python/conformance/__init__.py").relative_to(root).as_posix(),
                ]
                self.assertEqual(inventory.call_args.args[2], expected)
                self.assertEqual(inventory.call_args.args[3], {"owner": "tests"})
                run.reset_mock()
                host_fixtures.build("dev", "rust")
                self.assertEqual({call.args[0][2] for call in run.call_args_list if call.args[0][1] == "-C"}, {"sdk/rust"})
                self.assertEqual(inventory.call_args.args[2], expected)
        finally:
            temporary_directory.cleanup()

    def test_explicit_producer_failure_is_not_skipped(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            with (
                patch.object(host_fixtures.configuration, "ROOT", Path(temporary)),
                patch.object(host_fixtures.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "mise")),
                self.assertRaises(subprocess.CalledProcessError),
            ):
                host_fixtures.build("dev", "csharp")
        finally:
            temporary_directory.cleanup()


# === Public Functions === #


# === Private Functions === #


# === Entry Point === #
