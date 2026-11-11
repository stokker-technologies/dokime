# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Shipping Sidecar Archives Mark Bundles As Regular Files."""

# === Imports === #

# Standard Library Imports
import stat
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

# Third Party Imports
# Local Imports
from dokime_tools.package import plugins
from dokime_tools.release import layout

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ArchiveTests(unittest.TestCase):
    def test_zip_members_are_regular_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            bundle = root / "build" / "plugins" / "plugin-host.dokime"
            bundle.parent.mkdir(parents=True)
            bundle.write_bytes(b"bundle")
            bundle.chmod(0o600)
            with mock.patch.object(plugins, "shipping", return_value={"plugin-host.dokime": bundle}):
                destination = plugins.archive(root, root / "build", root / "out", "0.0.0", "x86_64-pc-windows-msvc")
            with zipfile.ZipFile(destination) as archive:
                entry = archive.getinfo("plugin-host.dokime")
                self.assertTrue(stat.S_ISREG(entry.external_attr >> 16))
                self.assertEqual(archive.read(entry), b"bundle")


class ReleaseLayoutTests(unittest.TestCase):
    def test_release_zip_members_are_regular_files(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            (source / "plugin-host.dokime").write_bytes(b"bundle")
            root = source / "release"
            bundle = {
                "target": "x86_64-pc-windows-msvc",
                "archive": "sidecars.zip",
                "member": "plugin-host.dokime",
                "path": "plugin-host.dokime",
            }
            (destination,) = layout.plugin_archives(source, root, [bundle])
            with zipfile.ZipFile(destination) as archive:
                entry = archive.getinfo("plugin-host.dokime")
                self.assertTrue(stat.S_ISREG(entry.external_attr >> 16))
                self.assertEqual(archive.read(entry), b"bundle")


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
