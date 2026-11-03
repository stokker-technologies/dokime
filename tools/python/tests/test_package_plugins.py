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

# Local Imports
from dokime_tools.package import plugins


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


if __name__ == "__main__":
    unittest.main()
