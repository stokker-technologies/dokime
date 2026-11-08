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
import copy
import ctypes
import email.message
import hashlib
import io
import json
import os
import runpy
import subprocess
import sys
import tempfile
import unittest
import urllib.error
from itertools import product
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

# Third Party Imports
# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

SETUP = runpy.run_path(str(Path(__file__).resolve().parents[4] / ".github/actions/setup/run.py"))
# Setup Functions Resolve Names In The Script Namespace, Which `run_path` Returns Only A Copy Of.
SETUP_GLOBALS = SETUP["verify_temporary_exclusions"].__globals__
SCANNING = {"AMRunningMode": "Normal", "RealTimeProtectionEnabled": True, "OnAccessProtectionEnabled": True}

# === Types === #

# === Classes === #


class NativeSetupTests(unittest.TestCase):
    def test_cache_order(self):
        combinations = product(("Linux-X64", "Windows-X64"), ("dev", "release"), ("product", "tests", "bench"))
        for platform, profile, kind in combinations:
            product_prefix = f"ci-v5-sccache-{platform}-{profile}-toolchain-"
            plan = {
                "sccache": {"key": product_prefix + "dependencies", "restore-keys": product_prefix, "path": ".cache/sccache/"},
                "mise": {"key": "tools", "path": "<MISE_DATA_DIR>"},
            }
            original = copy.deepcopy(plan)
            resolved = SETUP["cache_plan"](plan, {"COMPILER_CACHE_KIND": kind})
            self.assertEqual(plan, original)
            self.assertEqual(resolved["mise"], plan["mise"])
            compiler = resolved["sccache"]
            self.assertEqual(compiler["path"], plan["sccache"]["path"])
            if kind == "product":
                self.assertEqual(resolved, plan)
                self.assertEqual(SETUP["cache_plan"](plan, {}), original)
                continue
            build_profile = "bench" if kind == "bench" else profile
            prefix = f"ci-v5-sccache-{platform}-{build_profile}-{kind}-toolchain-"
            self.assertEqual(compiler["key"], prefix + "dependencies")
            self.assertEqual(compiler["restore-keys"].splitlines(), [prefix, product_prefix + "dependencies", product_prefix])

    def test_cache_locks(self):
        keys = set()
        combinations = product(("tests", "bench"), ("oldtools", "newtools"), ("oldcrates", "newcrates"))
        for kind, toolchain, dependencies in combinations:
            prefix = f"ci-v5-sccache-Linux-X64-release-{toolchain}-"
            plan = {"sccache": {"key": prefix + dependencies, "restore-keys": prefix, "path": "compiler"}}
            compiler = SETUP["cache_plan"](plan, {"COMPILER_CACHE_KIND": kind})["sccache"]
            self.assertNotIn(compiler["key"], keys)
            keys.add(compiler["key"])
            restores = compiler["restore-keys"].splitlines()
            self.assertTrue(all(f"-{toolchain}-" in restore for restore in restores))
            self.assertEqual(restores[0], compiler["key"].removesuffix(dependencies))
        self.assertEqual(SETUP["cache_plan"]({}, {"COMPILER_CACHE_KIND": "tests"}), {})
        with self.assertRaises(ValueError):
            SETUP["cache_plan"]({"sccache": {}}, {"COMPILER_CACHE_KIND": "unknown"})

    def test_cache_outputs(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "outputs"
            plan = {"sccache": {"key": "ci-v5-sccache-Linux-X64-dev-tools-crates", "restore-keys": "product-", "path": "compiler"}}
            environment = {"CACHE_PLAN": json.dumps(plan), "COMPILER_CACHE_KIND": "tests", "GITHUB_OUTPUT": str(output)}
            with (
                patch.dict(os.environ, environment, clear=True),
                patch.dict(SETUP_GLOBALS, {"rustup": lambda: None}),
                patch.dict(SETUP["EXPORTED"], {}, clear=True),
                patch.object(Path, "home", return_value=Path(directory) / "home"),
                patch.object(sys, "argv", ["setup", "--roots-only"]),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                SETUP["main"]()
            exported = output.read_text(encoding="utf-8")
        expected = (
            "sccache-path<<CACHE_VALUE\ncompiler\nCACHE_VALUE\n"
            "sccache-key<<CACHE_VALUE\nci-v5-sccache-Linux-X64-dev-tests-tools-crates\nCACHE_VALUE\n"
            "sccache-restore-keys<<CACHE_VALUE\nci-v5-sccache-Linux-X64-dev-tests-tools-\n"
            "ci-v5-sccache-Linux-X64-dev-tools-crates\nproduct-\nCACHE_VALUE\n"
        )
        self.assertEqual(exported, expected)

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


class SetupDownloadTests(unittest.TestCase):
    URL = "https://example.com/archive.zip"
    CHECKSUM = hashlib.sha256(b"archive").hexdigest()

    def setUp(self):
        self.calls = []
        self.pauses = []

    def fetch(self, codes, checksum=CHECKSUM):
        def opener(url, timeout):
            self.calls.append(url)
            code = codes.pop(0)
            if code != 200:
                raise urllib.error.HTTPError(url, code, "failure", email.message.Message(), io.BytesIO(b""))
            return io.BytesIO(b"archive")

        with tempfile.TemporaryDirectory() as directory, contextlib.redirect_stdout(io.StringIO()):
            destination = Path(directory) / "archive.zip"
            try:
                content = SETUP["download"](self.URL, checksum, destination, opener=opener, sleep=self.pauses.append)
                self.assertEqual(destination.read_bytes(), content)
                return content
            finally:
                self.assertEqual([path.name for path in Path(directory).iterdir() if path != destination], [])

    def test_transient_status_retries_then_returns_bytes(self):
        self.assertEqual(self.fetch([503, 200]), b"archive")
        self.assertEqual(len(self.calls), 2)
        self.assertEqual(len(self.pauses), 1)

    def test_missing_archive_fails_without_retry(self):
        with self.assertRaises(RuntimeError):
            self.fetch([404])
        self.assertEqual(self.calls, [self.URL])
        self.assertEqual(self.pauses, [])

    def test_checksum_mismatch_fails_without_retry(self):
        with self.assertRaises(ValueError):
            self.fetch([200, 200], checksum="0" * 64)
        self.assertEqual(self.calls, [self.URL])

    def test_exhausted_attempts_name_url_and_count(self):
        attempts = SETUP["DOWNLOAD_ATTEMPTS"]
        with self.assertRaises(RuntimeError) as raised:
            self.fetch([504] * attempts)
        self.assertEqual(len(self.calls), attempts)
        self.assertEqual(len(self.pauses), attempts - 1)
        self.assertIn(self.URL, str(raised.exception))
        self.assertIn(str(attempts), str(raised.exception))


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
