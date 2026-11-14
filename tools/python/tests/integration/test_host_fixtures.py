# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prove Independent SDK Fixture Producers And Local Toolchain Selection."""

# === Imports === #

# Standard Library Imports
import io
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
from harmos_tools.build import paths

# Local Imports
from dokime_tools.build import desktop_fixtures, host_fixtures

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class HostFixtureTests(unittest.TestCase):
    def test_desktop_bundle_failure_keeps_tool_diagnostics(self):
        diagnostic = "fixture stderr\n"
        refusal = subprocess.CompletedProcess(["dokime-next"], 3, stderr=diagnostic)
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.object(desktop_fixtures.configuration, "ROOT", Path(temporary)),
            patch.object(desktop_fixtures.subprocess, "run", side_effect=[subprocess.CompletedProcess(["mise"], 0), refusal]),
            patch.object(desktop_fixtures.shutil, "copy2"),
            patch.object(desktop_fixtures.sys, "stderr", new_callable=io.StringIO) as stderr,
            self.assertRaises(subprocess.CalledProcessError) as failure,
        ):
            desktop_fixtures.build("release")
        self.assertEqual(failure.exception.returncode, refusal.returncode)
        self.assertEqual(stderr.getvalue(), diagnostic)

    def test_explicit_producers_build_only_the_selected_language(self):
        for language in ("rust", "python", "csharp", "cpp", "none"):
            with self.subTest(language=language), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                (root / "Cargo.toml").write_text(
                    '[workspace.dependencies.harmos]\ngit = "https://example.test/library"\nrev = "fixture-revision"\n', encoding="utf-8"
                )
                with (
                    patch.object(host_fixtures.configuration, "ROOT", root),
                    patch.object(host_fixtures.subprocess, "run") as run,
                    patch.object(host_fixtures.shutil, "copy2"),
                    patch.object(host_fixtures.wheel, "build") as wheel,
                    patch.object(host_fixtures.archive, "stamp") as stamp,
                ):
                    host_fixtures.build("release", language)
                    commands = [call.args[0] for call in run.call_args_list]
                    self.assertEqual(
                        {command[2] for command in commands if command[1] == "-C"},
                        set() if language in ("none", "python") else {f"sdk/{language}"},
                    )
                    self.assertEqual(wheel.call_count, int(language == "python"))
                    stamp.assert_not_called()
                    if language == "cpp":
                        self.assertIn("--fetch-dependencies", commands[0])
                        self.assertEqual(commands[0][-1], commands[1][-1])
                    if language == "csharp":
                        self.assertEqual([command[4] for command in commands], ["build", "package", "publish:system"])
                    if language == "rust":
                        reference = next(command for command in commands if command[:2] == ["cargo", "install"])
                        self.assertEqual(reference[reference.index("--git") + 1], "https://example.test/library")
                        self.assertEqual(reference[reference.index("--rev") + 1], "fixture-revision")

    def test_local_default_builds_present_toolchains_and_preserves_inventory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "Cargo.toml").write_text(
                '[workspace.dependencies.harmos]\ngit = "https://example.test/library"\nrev = "fixture-revision"\n', encoding="utf-8"
            )
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
                patch.object(host_fixtures.wheel, "build") as wheel,
                patch.object(host_fixtures.archive, "stamp") as inventory,
            ):
                host_fixtures.build("dev")
                wheel.assert_called_once_with(root, "dev")
                self.assertEqual({call.args[0][2] for call in run.call_args_list if call.args[0][1] == "-C"}, {"sdk/rust", "sdk/csharp"})
                self.assertEqual(inventory.call_args.args[2], ["existing", (output / "fixture").relative_to(root).as_posix()])
                self.assertEqual(inventory.call_args.args[3], {"owner": "tests"})
                run.reset_mock()
                host_fixtures.build("dev", "rust")
                self.assertEqual({call.args[0][2] for call in run.call_args_list if call.args[0][1] == "-C"}, {"sdk/rust"})
                self.assertEqual(inventory.call_args.args[2], ["existing", (output / "fixture").relative_to(root).as_posix()])

    def test_explicit_producer_failure_is_not_skipped(self):
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.object(host_fixtures.configuration, "ROOT", Path(temporary)),
            patch.object(host_fixtures.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "mise")),
            self.assertRaises(subprocess.CalledProcessError),
        ):
            host_fixtures.build("dev", "csharp")


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
