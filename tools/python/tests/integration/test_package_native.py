# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Verify Local Kit Identity, Inventory and CLI Packaging Dispatch."

# === Imports === #

# Standard Library Imports
import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.package import distribution, native

from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class NativeTests(unittest.TestCase):
    def test_export_records_executable_identity_and_default_plugin_inventory(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        root = Path(scratch.name)
        build = root / "target/build/dev/windows-x86_64"
        (build / "product").mkdir(parents=True)
        (build / "product/target.txt").write_text("x86_64-pc-windows-msvc")
        for name in ("dokime.exe", "dokime-gui.exe", "bench-package.dokime"):
            path = build / "bin" / name
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(name.encode())
        for name in ("LICENSE", "target/notices/NOTICE", "target/notices/THIRD-PARTY-NOTICES", "organization/organization.toml"):
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(name.encode())
        (root / "tools").mkdir()
        (root / "tools/project.toml").write_text(
            '[version]\nvalue="1.2.3"\n[[release.plugins]]\npackage="bench-package"\n[release.plugins.targets]\nwindows-x86_64="x86_64-pc-windows-msvc"\n'
        )
        identity = {"version": "1.2.3", "revision": "f7c50e68ab01", "channel": "nightly"}
        result = subprocess.CompletedProcess([], 0, stdout=json.dumps(identity))
        inspected = subprocess.CompletedProcess([], 0, stdout=json.dumps({"identity": {"name": "bench"}}))
        with (
            patch.object(distribution.paths, "target", return_value="windows-x86_64"),
            patch.object(distribution.subprocess, "run", side_effect=[result, inspected]) as run,
        ):
            source = distribution.export(root, "dev")
        self.assertEqual(run.call_args_list[0].args[0], [str(build / "bin/dokime.exe"), "version", "--json"])
        self.assertEqual(
            run.call_args_list[1].args[0],
            [str(build / "bin/dokime.exe"), "plugin", "inspect", "--bundle", str(build / "bin/bench-package.dokime"), "--json"],
        )
        metadata = json.loads((source / "distribution.json").read_text())
        self.assertEqual(metadata["source_revision"], identity["revision"])
        self.assertEqual(metadata["channel"], identity["channel"])
        self.assertEqual(metadata["plugins"], ["bench"])
        self.assertEqual((source / "plugins/bench-package.dokime").read_bytes(), b"bench-package.dokime")
        archive_path = distribution.archive(source, root / "output")
        archive = zipfile.ZipFile(archive_path)
        try:
            self.assertEqual(set(archive.namelist()), {"distribution.json", *metadata["files"]})
            for entry in archive.infolist():
                mode = entry.external_attr >> 16
                self.assertTrue(stat.S_ISREG(mode))
                self.assertEqual(mode & 0o777, 0o755 if entry.filename.startswith("bin/") else 0o644)
        finally:
            archive.close()
        (source / "LICENSE").write_bytes(b"changed")
        self.assertRaises(policy.ToolError, distribution.archive, source, root / "changed")

    def test_package_calls_only_the_matching_cli_with_local_artifacts(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        root = Path(scratch.name)
        for windows, signed, stable in ((True, False, False), (True, True, True), (False, False, False), (True, False, True)):
            source = root / "target/build/dev/native/product/distribution"
            source.mkdir(parents=True, exist_ok=True)
            metadata = {
                "target": "x86_64-pc-windows-msvc" if windows else "x86_64-unknown-linux-gnu",
                "version": "1.2.3",
                "channel": "nightly",
            }
            (source / "distribution.json").write_text(json.dumps(metadata))
            environment = {"DOKIME_RELEASE_CONTEXT": "true" if stable else "false"}
            if signed:
                environment["TAURI_SIGNING_PRIVATE_KEY"] = "key reference"
            with (
                self.subTest(windows=windows, signed=signed, stable=stable),
                patch.dict(os.environ, environment, clear=True),
                patch.object(native.paths, "output", side_effect=lambda root, stage, profile: root / "target" / stage / profile / "native"),
                patch.object(native.subprocess, "run") as run,
            ):
                if windows and stable and not signed:
                    self.assertRaises(ValueError, native.package, root, "dev")
                    run.assert_not_called()
                    continue
                native.package(root, "dev")
            command = run.call_args.args[0]
            self.assertEqual(command[0], str(source / "bin" / ("dokime.exe" if windows else "dokime")))
            self.assertEqual(command[1:3], ["installer", "package"])
            self.assertEqual(command[command.index("--artifacts") + 1], str(source))
            self.assertEqual(command[command.index("--manifest") + 1], str(root / "installer.toml"))
            self.assertEqual(command[command.index("--format") + 1], "nsis" if windows else "deb,rpm")
            self.assertEqual("--sign" in command, signed)
            self.assertIs(run.call_args.kwargs["stdin"], sys.stdin)
            self.assertIs(run.call_args.kwargs["stdout"], sys.stdout)
            self.assertIs(run.call_args.kwargs["stderr"], sys.stderr)
            if signed:
                self.assertEqual(run.call_args.kwargs["env"]["TAURI_SIGNING_PRIVATE_KEY_PASSWORD"], "")
            self.assertEqual(
                run.call_args.kwargs["env"]["TAURI_UPDATER_ARTIFACT_BASE_URL"],
                "https://github.com/stokker-technologies/dokime/releases/download/nightly",
            )


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
