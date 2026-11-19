# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Native Bundle Configuration and Product Bytes"""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.package import installer

from .environment import isolate_environment

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class InstallerTests(unittest.TestCase):
    def test_windows_bundles_verified_resources_with_user_scope_and_update_feed(self) -> None:
        self._bundle("win32")

    def test_linux_bundles_verified_resources_as_native_packages(self) -> None:
        self._bundle("linux")

    def _bundle(self, platform: str) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        root = Path(scratch.name)
        windows = platform == "win32"
        native = "windows-x86_64" if windows else "linux-x86_64"
        triple = "x86_64-pc-windows-msvc" if windows else "x86_64-unknown-linux-gnu"
        suffix = ".exe" if windows else ""

        # Prepare Verified Product Inputs.
        source = root / "target/build/dev" / native / "product/distribution"
        binary = source / "bin" / f"dokime-gui{suffix}"
        binary.parent.mkdir(parents=True)
        files = {f"bin/{binary.name}": b"desktop", f"bin/dokime{suffix}": b"command line", "organization/logo.png": b"artwork"}
        for name, content in files.items():
            path = source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        manifest = {
            "target": triple,
            "version": "1.2.3",
            "files": {name: hashlib.sha256(value).hexdigest() for name, value in files.items()},
        }
        (source / "distribution.json").write_text(json.dumps(manifest), encoding="utf-8")
        config = root / "crates/dokime-gui/src-tauri/tauri.conf.json"
        config.parent.mkdir(parents=True)
        config.write_text(json.dumps({"plugins": {"updater": {"pubkey": "verification key"}}}), encoding="utf-8")

        # Inspect the Real Overlay Without Running a Bundler or Installer.
        def bundle(command: list[str], *, env: dict[str, str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
            overlay = json.loads(Path(command[command.index("--config") + 1]).read_text(encoding="utf-8"))["bundle"]
            resources = {name.removeprefix("bin/"): str(source / name) for name in files if name != f"bin/{binary.name}"}
            expected: dict[str, object] = {
                "active": True,
                "createUpdaterArtifacts": windows,
                "resources": {path: name for name, path in resources.items()},
            }
            if windows:
                expected["windows"] = {"nsis": {"installMode": "currentUser", "installerHooks": str(config.parent / "installer-hooks.nsh")}}
            self.assertEqual(overlay, expected)
            self.assertEqual(command[command.index("--bundles") + 1], "nsis" if windows else "deb,rpm")
            self.assertIn("--no-binary-patching", command)
            self.assertIn("--debug", command)
            target = Path(env["CARGO_TARGET_DIR"])
            self.assertEqual((target / "debug" / binary.name).read_bytes(), files[f"bin/{binary.name}"])
            output = target / "debug/bundle"
            output.mkdir(parents=True)
            for ending in (".exe", ".exe.sig") if windows else (".deb", ".rpm"):
                (output / ("fixture" + ending)).write_bytes(b"signature" if ending == ".exe.sig" else b"installer")
            print(f"Bundle Overlay ({platform}): {json.dumps(overlay, sort_keys=True)}")
            return subprocess.CompletedProcess(command, 0, stdout="", stderr="")

        # Package Only the Simulated Native Bundler's Existing Bytes.
        with (
            patch.object(installer.sys, "platform", platform),
            patch.object(installer.paths, "target", return_value=native),
            patch.dict(os.environ, {"TAURI_SIGNING_PRIVATE_KEY": "test key"}, clear=True),
            patch.object(installer.subprocess, "run", side_effect=bundle),
        ):
            outputs = installer.bundle(root, "dev")
        self.assertEqual(len(outputs), 2)
        self.assertTrue(all(path.is_relative_to(root / "target/package/dev" / native / "rust") for path in outputs))
        self.assertEqual(binary.read_bytes(), b"desktop")
        if windows:
            feed = json.loads((outputs[0].parent / "latest.json").read_text(encoding="utf-8"))
            update = feed["platforms"]["windows-x86_64"]
            self.assertEqual(update["signature"], "signature")
            self.assertTrue(update["url"].endswith("/" + next(path.name for path in outputs if path.suffix == ".exe")))
        else:
            self.assertEqual({path.suffix for path in outputs}, {".deb", ".rpm"})


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
