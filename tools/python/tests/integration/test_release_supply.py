# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Release Inventory Integrity and Unsafe Archive Refusal."""

# === Imports === #

# Standard Library Imports
import hashlib
import io
import json
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

# Third Party Imports
# Local Imports
from dokime_tools.release import notices, policy, sbom

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class SupplyTests(unittest.TestCase):
    def test_archive_traversal_is_rejected_before_extraction(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = root / "input.zip"
            with zipfile.ZipFile(archive, "w") as output:
                output.writestr("safe.txt", "content")
                output.writestr("../outside.txt", "content")
            destination = root / "unpacked"
            destination.mkdir()
            with self.assertRaises(policy.ToolError):
                sbom.unpack(archive, destination)
            self.assertEqual(list(destination.iterdir()), [])
            self.assertFalse((root / "outside.txt").exists())

    def test_artifact_records_digest_and_deduplicates_components(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "cli.zip"
            with zipfile.ZipFile(source, "w") as output:
                output.writestr("dokime", b"binary")
            cargo = root / "cargo.json"
            component = {"type": "library", "name": "dependency", "purl": "pkg:cargo/dependency@1.0"}
            cargo.write_text(json.dumps({"components": [component]}))
            scan = {"bomFormat": "CycloneDX", "specVersion": "1.5", "version": 1, "components": [component]}
            with mock.patch.object(policy, "execute", return_value=json.dumps(scan)):
                result = sbom.artifact(root, source, root / "output", [cargo])
            document = json.loads(result.read_text())
            self.assertEqual(len(document["components"]), 1)
            self.assertEqual(document["metadata"]["component"]["hashes"][0]["content"], hashlib.sha256(source.read_bytes()).hexdigest())

    def test_python_wheel_hash_mismatch_is_rejected(self):
        lock = {
            "package": [
                {
                    "name": "dependency",
                    "version": "1",
                    "source": {"registry": "https://pypi.org/simple"},
                    "wheels": [{"url": "https://example.invalid/wheel", "hash": "sha256:wrong"}],
                }
            ]
        }
        with (
            mock.patch("urllib.request.urlopen", return_value=io.BytesIO(b"changed")),
            self.assertRaisesRegex(policy.ToolError, "hash mismatch"),
        ):
            notices.python_notices(lock)

    def test_python_notices_use_verified_license_files(self):
        content = io.BytesIO()
        with zipfile.ZipFile(content, "w") as output:
            output.writestr("dependency-1.dist-info/METADATA", "Name: dependency\nVersion: 1\nLicense-Expression: MIT\n")
            output.writestr("dependency-1.dist-info/licenses/LICENSE", "fixture license text")
        data = content.getvalue()
        lock = {
            "package": [
                {
                    "name": "dependency",
                    "version": "1",
                    "source": {"registry": "https://pypi.org/simple"},
                    "wheels": [{"url": "https://example.invalid/wheel", "hash": "sha256:" + hashlib.sha256(data).hexdigest()}],
                }
            ]
        }
        with mock.patch("urllib.request.urlopen", return_value=io.BytesIO(data)):
            text, count = notices.python_notices(lock)
        self.assertEqual(count, 1)
        self.assertIn("fixture license text", text)

    def test_shell_missing_license_text_fails(self):
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(policy.ToolError):
            notices.shell_notices({"MIT": [{"name": "dependency", "versions": ["1"], "paths": [temporary]}]})

    def test_staged_notice_hashes_match_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            destination = root / "distribution"
            destination.mkdir()
            (destination / "distribution.json").write_text(json.dumps({"files": {"existing": "digest"}}))
            for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                (root / name).write_bytes(name.encode())
            notices.stage(root, destination)
            files = json.loads((destination / "distribution.json").read_text())["files"]
            self.assertEqual(files["existing"], "digest")
            for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                self.assertEqual(files[name], hashlib.sha256((destination / name).read_bytes()).hexdigest())


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
