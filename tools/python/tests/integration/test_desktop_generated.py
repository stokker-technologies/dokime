# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"""Prove Canonical Desktop Preference and Command Generation."""

# === Imports === #

# Standard Library Imports
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from dokime_tools.prepare import generated

# Third Party Imports
# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DesktopMessageGenerationTests(unittest.TestCase):
    def test_projects_canonical_preferences_and_native_commands(self):
        root = Path(__file__).resolve().parents[4]
        with tempfile.TemporaryDirectory(prefix="desktop-generation-") as scratch:
            directory = Path(scratch)
            fixture = directory / "preferences.bin"
            subprocess.run(
                [
                    "node",
                    "--input-type=commonjs",
                    "-e",
                    r"""
const { createRequire } = require("node:module");
const { readFileSync, writeFileSync } = require("node:fs");
const { resolve } = require("node:path");
const packageRequire = createRequire(resolve("crates/dokime-gui/package.json"));
const { create, createFileRegistry, fromBinary, toBinary } = packageRequire("@bufbuild/protobuf");
const { FileDescriptorSetSchema } = packageRequire("@bufbuild/protobuf/wkt");
const registry = createFileRegistry(fromBinary(FileDescriptorSetSchema, readFileSync("crates/dokime-protocol/src/descriptors.bin")));
const type = registry.getMessage("dokime.v1.SettingsSnapshot");
const value = create(type, { revision: 7n, settings: {
  appearance: { themeMode: 1, density: 2 }, report: { plotDownsample: 4, plotPoints: 1537 },
  client: { connectTimeoutMs: 4321n, requestTimeoutMs: 8765n }, telemetry: { intervalMs: 321n },
  listener: { enabled: true, certificateNames: ["fixture.local"] }, plugins: { startupTimeoutS: 31, stopTimeoutS: 17 },
}});
writeFileSync(process.argv[1], toBinary(type, value));
""",
                    str(fixture),
                ],
                cwd=root,
                check=True,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
            output = directory / "modules"
            subprocess.run(
                ["node", str(root / "tools/typescript/generate.mjs"), "--output", str(output), "--defaults", str(fixture)],
                cwd=root,
                check=True,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
            settings = json.loads((output / "application-settings.json").read_text(encoding="utf-8"))
            self.assertEqual(settings["client"], {"connect_timeout_ms": 4321, "request_timeout_ms": 8765})
            self.assertIsNone(settings["appearance"]["theme_family"])
            self.assertEqual(settings["listener"]["certificate_names"], ["fixture.local"])
            declarations = (output / "application-settings.ts").read_text(encoding="utf-8")
            self.assertIn("theme_family: string | null", declarations)
            self.assertIn("endpoints: Array<string>", declarations)
            commands = (output / "native-preferences.ts").read_text(encoding="utf-8")
            self.assertIn("update: number[]", commands)
            self.assertIn("invoke<ArrayBuffer>", commands)

    def test_product_round_trip_keeps_both_message_roots(self):
        with tempfile.TemporaryDirectory(prefix="desktop-product-modules-") as scratch:
            root = Path(scratch)
            modules = root / "target/generated/typescript"
            host = root / "target/generated/host"
            modules.mkdir(parents=True)
            host.mkdir()
            (modules / "preferences.ts").write_text("export const revision = 7;", encoding="utf-8")
            (host / "client.ts").write_text("export const revision = 11;", encoding="utf-8")
            with patch.object(generated, "ROOT", root), patch.object(generated, "MODULES", modules):
                staged = generated.stage("dev")
                (modules / "preferences.ts").unlink()
                (host / "client.ts").unlink()
                generated.restore("dev")
                self.assertEqual((modules / "preferences.ts").read_bytes(), (staged / "preferences.ts").read_bytes())
                self.assertEqual((host / "client.ts").read_bytes(), (staged.parent / "host/client.ts").read_bytes())
                (staged.parent / "host/client.ts").unlink()
                (staged.parent / "host").rmdir()
                with self.assertRaises(SystemExit):
                    generated.restore("dev")
                self.assertTrue((modules / "preferences.ts").is_file())


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
