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

# Type Checking Imports


# === Public Exports === #

# === Constants === #

SETUP = runpy.run_path(str(Path(__file__).resolve().parents[3] / ".github/actions/setup/run.py"))

# === Types === #

# === Classes === #


class NativeSetupTests(unittest.TestCase):
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
                self.assertTrue(options["check"])
                self.assertEqual(command[1:3], ["-CheckExclusion", "-Path"])
                checked.append((target, target.is_dir()))

            with (
                patch.dict(os.environ, {"ProgramFiles": str(root)}),
                patch.object(subprocess, "run", side_effect=run),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                SETUP["verify_temporary_exclusions"]([root])
            for prefix in ("dokime-exclusion-", ".tmp"):
                selected = [(path, folder) for path, folder in checked if path.relative_to(root).parts[0].startswith(prefix)]
                self.assertTrue(any(folder for _, folder in selected))
                self.assertTrue(any(not folder and path.parent == root for path, folder in selected))
                self.assertTrue(any(not folder and path.parent != root for path, folder in selected))
            self.assertTrue(all(not path.exists() for path, _ in checked))

    def test_native_setup_rejects_failed_exclusion_matching(self):
        with tempfile.TemporaryDirectory() as directory:
            with (
                patch.dict(os.environ, {"ProgramFiles": directory}),
                patch.object(subprocess, "run", side_effect=subprocess.CalledProcessError(1, ["fixture"])),
                contextlib.redirect_stdout(io.StringIO()),
                self.assertRaises(subprocess.CalledProcessError),
            ):
                SETUP["verify_temporary_exclusions"]([Path(directory)])
            self.assertEqual(list(Path(directory).iterdir()), [])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
