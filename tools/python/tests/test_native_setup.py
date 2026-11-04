# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Native Runner Temporary Storage Provisioning"""

# === Imports === #

# Standard Library Imports
import contextlib
import ctypes
import io
import os
import runpy
import subprocess
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.ci import environment as step_environment

# Type Checking Imports


# === Public Exports === #

# === Constants === #

SETUP = runpy.run_path(str(Path(__file__).resolve().parents[3] / ".github/actions/setup/run.py"))
# Setup Functions Resolve Names In The Script Namespace, Which `run_path` Returns Only A Copy Of.
SETUP_GLOBALS = SETUP["verify_temporary_exclusions"].__globals__
SCANNING = {"AMRunningMode": "Normal", "RealTimeProtectionEnabled": True, "OnAccessProtectionEnabled": True}

# === Types === #

# === Classes === #


class NativeSetupTests(unittest.TestCase):
    def test_setup_environment_round_trip_preserves_provider_guards(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            environment = root / "step.env"
            paths = root / "step.path"
            paths.write_text("", encoding="utf-8")
            values = SETUP["roots"]({"ImageOS": "native-fixture", "ImageVersion": "20261102"}, root / "workspace", root / "home")
            with patch.dict(os.environ, {"GITHUB_ENV": str(environment)}), patch.dict(SETUP["EXPORTED"], {}, clear=True):
                SETUP["export"](values)
            self.assertEqual(step_environment.read(environment, paths, {}), values)
            self.assertEqual(environment.read_text(encoding="utf-8"), "")

            for name in ("GITHUB_WORKSPACE", "GITHUB_TOKEN", "RUNNER_OS", "RUNNER_TEMP", "NODE_OPTIONS"):
                with self.subTest(name=name):
                    environment.write_text(f"{name}=refused\n", encoding="utf-8")
                    with self.assertRaises(ValueError):
                        step_environment.read(environment, paths, {})

    def test_native_setup_preserves_native_and_python_temp_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            native = root / "native"
            native.mkdir()
            (root / "other").mkdir()
            alias = root / "other/../native"
            python = root / "python"
            python.mkdir()

            def lookup(_, buffer):
                buffer.value = str(alias)
                return len(buffer.value)

            api = SimpleNamespace(kernel32=SimpleNamespace(GetTempPath2W=lookup))
            with patch.object(ctypes, "windll", api, create=True), patch.object(tempfile, "gettempdir", return_value=str(python)):
                paths = SETUP["native_temporary_paths"]()
            self.assertEqual(paths, [alias, native, python])

    def test_native_setup_refuses_unavailable_native_temp_path(self):
        api = SimpleNamespace(kernel32=SimpleNamespace(GetTempPath2W=lambda *_: 0))
        with patch.object(ctypes, "windll", api, create=True), self.assertRaises(OSError):
            SETUP["native_temporary_paths"]()

    def test_native_setup_checks_concrete_files_and_folders(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            checked = []

            def run(command, **options):
                target = Path(command[-1])
                self.assertTrue(target.exists())
                self.assertEqual(command[1:3], ["-CheckExclusion", "-Path"])
                checked.append((target, target.is_dir()))
                return subprocess.CompletedProcess(command, 0)

            with (
                patch.dict(os.environ, {"ProgramFiles": str(root)}),
                patch.dict(SETUP_GLOBALS, {"scanner_status": lambda: SCANNING}),
                patch.object(subprocess, "run", side_effect=run),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                SETUP["verify_temporary_exclusions"]([root])
            for prefix in ("dokime-exclusion-", ".tmp", "project-tmp-"):
                selected = [(path, folder) for path, folder in checked if path.relative_to(root).parts[0].startswith(prefix)]
                self.assertTrue(any(folder for _, folder in selected))
                self.assertTrue(any(not folder and path.parent == root for path, folder in selected))
                self.assertTrue(any(not folder and path.parent != root for path, folder in selected))
            self.assertTrue(all(not path.exists() for path, _ in checked))

    def test_native_setup_rejects_failed_exclusion_matching(self):
        with tempfile.TemporaryDirectory() as directory:
            commands = []

            def run(command, **options):
                commands.append(command)
                return subprocess.CompletedProcess(command, 1 if "-CheckExclusion" in command else 0)

            with (
                patch.dict(os.environ, {"ProgramFiles": directory}),
                patch.dict(SETUP_GLOBALS, {"scanner_status": lambda: SCANNING}),
                patch.object(subprocess, "run", side_effect=run),
                contextlib.redirect_stdout(io.StringIO()),
                self.assertRaises(RuntimeError),
            ):
                SETUP["verify_temporary_exclusions"]([Path(directory)])
            self.assertTrue(any("(Get-MpPreference).ExclusionPath" in command for command in commands))
            self.assertEqual(list(Path(directory).iterdir()), [])

    def test_native_setup_skips_probes_without_on_access_scanning(self):
        for status in ({"RealTimeProtectionEnabled": False}, {"RealTimeProtectionEnabled": True, "OnAccessProtectionEnabled": False}):
            with self.subTest(status=status), tempfile.TemporaryDirectory() as directory:
                with (
                    patch.dict(os.environ, {"ProgramFiles": directory}),
                    patch.dict(SETUP_GLOBALS, {"scanner_status": lambda status=status: status}),
                    patch.object(subprocess, "run", side_effect=AssertionError("no probe runs")),
                    contextlib.redirect_stdout(io.StringIO()),
                ):
                    SETUP["verify_temporary_exclusions"]([Path(directory)])
                self.assertEqual(list(Path(directory).iterdir()), [])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
