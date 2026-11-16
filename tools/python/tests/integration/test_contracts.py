# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Protocol Archive Import Closure"""

# === Imports === #

# Standard Library Imports
import shutil
import tempfile
import unittest
import zipfile
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools import policy
from dokime_tools.release import contracts

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ContractsTests(unittest.TestCase):
    def test_archive_requires_the_complete_import_closure(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, scratch)
        files = {
            "dokime/host/v1/service.proto": 'syntax = "proto3"; import public "harmos/model/v1/value.proto";',
            "harmos/model/v1/value.proto": 'syntax = "proto3"; import weak "google/protobuf/timestamp.proto";',
            "google/protobuf/timestamp.proto": 'syntax = "proto3"; import "google/protobuf/descriptor.proto";',
            "google/protobuf/descriptor.proto": 'syntax = "proto2"; /* import "ignored.proto"; */ // import "ignored.proto";',
            "README.md": "Usage",
            "buf.yaml": "version: v2",
            "licenses/Apache-2.0.txt": "Apache",
            "licenses/Protobuf.txt": "BSD",
        }
        for missing in (None, "harmos/model/v1/value.proto", "google/protobuf/descriptor.proto", "licenses/Protobuf.txt"):
            with self.subTest(missing=missing):
                exported = Path(tempfile.mkdtemp(dir=scratch))
                for name, content in files.items():
                    if name == missing:
                        continue
                    path = exported / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text(content, encoding="utf-8")
                destination = exported.parent / "contracts.zip"
                destination.unlink(missing_ok=True)
                if missing is not None:
                    with self.assertRaises(policy.ToolError):
                        contracts.package(exported, destination)
                    self.assertFalse(destination.exists())
                    continue
                contracts.package(exported, destination)
                archive = zipfile.ZipFile(destination)
                try:
                    self.assertEqual(set(archive.namelist()), set(files))
                    self.assertEqual({name: archive.read(name).decode() for name in archive.namelist()}, files)
                finally:
                    archive.close()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
