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
from pathlib import Path
from types import SimpleNamespace
from typing import Any, Protocol
from unittest.mock import Mock, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.ci import bootstrap

# Type Checking Imports

# === Public Exports === #

# === Constants === #

SETUP = runpy.run_path(str(Path(__file__).resolve().parents[4] / "tools/python/src/dokime_tools/ci/native.py"))
# Setup Functions Resolve Names In The Script Namespace, Which `run_path` Returns Only A Copy Of.
SETUP_GLOBALS = SETUP["verify_temporary_exclusions"].__globals__
SCANNING = {"AMRunningMode": "Normal", "RealTimeProtectionEnabled": True, "OnAccessProtectionEnabled": True}

# === Types === #

# === Classes === #


class NativeTextBuffer(Protocol):
    value: str


class PackageFetchTests(unittest.TestCase):
    def test_fetch_recovers_from_install_and_update_mirror_failures(self) -> None:
        failure = "Temporary failure resolving 'azure.archive.ubuntu.com'\nUnable to fetch some archives\n"
        runner = Mock(
            side_effect=[
                subprocess.CompletedProcess([], status, stdout=output)
                for status, output in ((0, ""), (100, failure), (100, failure), (0, ""), (0, ""))
            ]
        )
        sleep = Mock()
        with contextlib.redirect_stdout(io.StringIO()):
            bootstrap.fetch_packages(["fixture-package"], ["sudo", "-n"], runner=runner, sleep=sleep)
        commands = [call.args[0] for call in runner.call_args_list]
        self.assertEqual(
            ["install" if "install" in command else "update" for command in commands], ["update", "install", "update", "update", "install"]
        )
        self.assertTrue(all(command[:3] == ["sudo", "-n", "apt-get"] for command in commands))
        self.assertTrue(all(f"Acquire::Retries={bootstrap.APT_ACQUIRE_RETRIES}" in command for command in commands))
        self.assertTrue(all("APT::Update::Error-Mode=any" in command for command in commands if "update" in command))
        self.assertIn("fixture-package", commands[-1])
        self.assertEqual([call.args[0] for call in sleep.call_args_list], list(bootstrap.APT_RETRY_DELAYS_S[:2]))

    def test_fetch_stops_at_retry_bound_and_preserves_mirror_error(self) -> None:
        failure = "Temporary failure resolving 'azure.archive.ubuntu.com'\nUnable to fetch some archives\n"
        for operation in ("update", "install"):
            with self.subTest(operation=operation):
                outcomes = [subprocess.CompletedProcess([], 0, stdout="")] if operation == "install" else []
                outcomes += [subprocess.CompletedProcess([], 100, stdout=failure)]
                attempts = len(bootstrap.APT_RETRY_DELAYS_S) + 1
                runner = Mock(side_effect=outcomes * attempts)
                sleep = Mock()
                with contextlib.redirect_stdout(io.StringIO()), self.assertRaises(RuntimeError) as raised:
                    bootstrap.fetch_packages(["fixture-package"], runner=runner, sleep=sleep)
                self.assertIn(f"after {attempts} attempts", str(raised.exception))
                self.assertIn(failure.strip(), str(raised.exception))
                self.assertEqual(runner.call_count, len(outcomes) * attempts)
                self.assertEqual([call.args[0] for call in sleep.call_args_list], list(bootstrap.APT_RETRY_DELAYS_S))

    def test_successful_fetch_does_not_retry(self) -> None:
        runner = Mock(return_value=subprocess.CompletedProcess([], 0, stdout=""))
        sleep = Mock()
        with contextlib.redirect_stdout(io.StringIO()):
            bootstrap.fetch_packages(["fixture-package"], runner=runner, sleep=sleep)
        self.assertEqual(runner.call_count, 2)
        sleep.assert_not_called()


class NativeSetupTests(unittest.TestCase):
    def test_visual_studio_selects_latest_cpp_installation(self) -> None:
        installation = Path("C:/Visual Studio/Build Tools")
        with (
            patch.object(sys, "platform", "win32"),
            patch.dict(os.environ, {"ProgramFiles(x86)": "C:/Programs"}),
            patch.object(subprocess, "check_output", return_value=f"{installation}\r\n") as output,
        ):
            self.assertEqual(SETUP["visual_studio"](), installation)
        self.assertEqual(
            output.call_args.args[0],
            [
                str(Path("C:/Programs/Microsoft Visual Studio/Installer/vswhere.exe")),
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-property",
                "installationPath",
            ],
        )

    def test_visual_studio_refuses_missing_cpp_tools(self) -> None:
        with (
            patch.object(sys, "platform", "win32"),
            patch.dict(os.environ, {"ProgramFiles(x86)": "C:/Programs"}),
            patch.object(subprocess, "check_output", return_value="\r\n"),
            self.assertRaises(RuntimeError),
        ):
            SETUP["visual_studio"]()

    def test_cache_outputs(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            output = Path(directory) / "outputs"
            plan = {"sccache": {"key": "compiler-key", "restore-keys": "compiler-", "path": "compiler"}}
            environment = {"CACHE_PLAN": json.dumps(plan), "GITHUB_OUTPUT": str(output)}
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
        finally:
            directory_directory.cleanup()
        expected = (
            "sccache-path<<CACHE_VALUE\ncompiler\nCACHE_VALUE\n"
            "sccache-key<<CACHE_VALUE\ncompiler-key\nCACHE_VALUE\n"
            "sccache-restore-keys<<CACHE_VALUE\ncompiler-\nCACHE_VALUE\n"
        )
        self.assertEqual(exported, expected)

    def test_native_setup_preserves_native_and_python_temp_paths(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            root = Path(directory).resolve()
            native = root / "native"
            native.mkdir()
            (root / "other").mkdir()
            alias = root / "other/../native"
            python = root / "python"
            python.mkdir()

            def lookup(_: int, buffer: NativeTextBuffer) -> int:
                buffer.value = str(alias)
                return len(buffer.value)

            api = SimpleNamespace(kernel32=SimpleNamespace(GetTempPath2W=lookup))
            with (
                patch.object(sys, "platform", "win32"),
                patch.object(ctypes, "windll", api, create=True),
                patch.object(tempfile, "gettempdir", return_value=str(python)),
            ):
                paths = SETUP["native_temporary_paths"]()
            self.assertEqual(paths, [alias, native, python])
        finally:
            directory_directory.cleanup()

    def test_native_setup_refuses_unavailable_native_temp_path(self) -> None:
        api = SimpleNamespace(kernel32=SimpleNamespace(GetTempPath2W=lambda *_: 0))
        with patch.object(sys, "platform", "win32"), patch.object(ctypes, "windll", api, create=True), self.assertRaises(OSError):
            SETUP["native_temporary_paths"]()

    def test_windows_apis_refuse_other_platforms(self) -> None:
        with patch.object(sys, "platform", "linux"):
            with self.assertRaises(OSError):
                SETUP["visual_studio"]()
            with self.assertRaises(OSError):
                SETUP["windows"](Path.cwd())
            with self.assertRaises(OSError):
                SETUP["native_temporary_paths"]()

    def test_native_setup_checks_concrete_files_and_folders(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            root = Path(directory)
            checked = []

            def run(command: list[str], **options: Any) -> subprocess.CompletedProcess[str]:
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
                selected = []
                for selected_path, selected_folder in checked:
                    if not (selected_path.relative_to(root).parts[0].startswith(prefix)):
                        continue
                    selected.append((selected_path, selected_folder))

                self.assertTrue(any(folder for _, folder in selected))
                self.assertTrue(any(not folder and path.parent == root for path, folder in selected))
                self.assertTrue(any(not folder and path.parent != root for path, folder in selected))
            self.assertTrue(all(not path.exists() for path, _ in checked))
        finally:
            directory_directory.cleanup()

    def test_native_setup_rejects_failed_exclusion_matching(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            commands = []

            def run(command: list[str], **options: Any) -> subprocess.CompletedProcess[str]:
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
        finally:
            directory_directory.cleanup()

    def test_native_setup_skips_probes_without_on_access_scanning(self) -> None:
        for status in ({"RealTimeProtectionEnabled": False}, {"RealTimeProtectionEnabled": True, "OnAccessProtectionEnabled": False}):
            directory_directory = tempfile.TemporaryDirectory()
            directory = directory_directory.name
            try:
                with self.subTest(status=status):
                    with (
                        patch.dict(os.environ, {"ProgramFiles": directory}),
                        patch.dict(SETUP_GLOBALS, {"scanner_status": lambda status=status: status}),
                        patch.object(subprocess, "run", side_effect=AssertionError("no probe runs")),
                        contextlib.redirect_stdout(io.StringIO()),
                    ):
                        SETUP["verify_temporary_exclusions"]([Path(directory)])
                    self.assertEqual(list(Path(directory).iterdir()), [])
            finally:
                directory_directory.cleanup()


class SetupDownloadTests(unittest.TestCase):
    URL = "https://example.com/archive.zip"
    CHECKSUM = hashlib.sha256(b"archive").hexdigest()

    def setUp(self) -> None:
        self.calls: list = []
        self.pauses: list = []

    def fetch(self, codes: list[int], checksum: str | None = CHECKSUM) -> bytes:
        def opener(url: str, timeout: float) -> io.BytesIO:
            self.calls.append(url)
            code = codes.pop(0)
            if code != 200:
                raise urllib.error.HTTPError(url, code, "failure", email.message.Message(), io.BytesIO(b""))
            return io.BytesIO(b"archive")

        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            with contextlib.redirect_stdout(io.StringIO()):
                destination = Path(directory) / "archive.zip"
                try:
                    content = SETUP["download"](self.URL, checksum, destination, opener=opener, sleep=self.pauses.append)
                    self.assertEqual(destination.read_bytes(), content)
                    return content
                finally:
                    self.assertEqual([path.name for path in Path(directory).iterdir() if path != destination], [])
        finally:
            directory_directory.cleanup()

    def test_transient_status_retries_then_returns_bytes(self) -> None:
        self.assertEqual(self.fetch([503, 200]), b"archive")
        self.assertEqual(len(self.calls), 2)
        self.assertEqual(len(self.pauses), 1)

    def test_missing_archive_fails_without_retry(self) -> None:
        with self.assertRaises(RuntimeError):
            self.fetch([404])
        self.assertEqual(self.calls, [self.URL])
        self.assertEqual(self.pauses, [])

    def test_checksum_mismatch_fails_without_retry(self) -> None:
        with self.assertRaises(ValueError):
            self.fetch([200, 200], checksum="0" * 64)
        self.assertEqual(self.calls, [self.URL])

    def test_exhausted_attempts_name_url_and_count(self) -> None:
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
