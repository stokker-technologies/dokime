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
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.build import host_fixtures, paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class HostFixtureTests(unittest.TestCase):
    def test_explicit_producers_build_only_the_selected_language(self):
        for language in ("rust", "python", "csharp", "cpp", "none"):
            with self.subTest(language=language), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
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

    def test_local_default_builds_present_toolchains_and_preserves_inventory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
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
