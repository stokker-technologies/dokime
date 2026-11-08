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
import base64
import csv
import hashlib
import io
import json
import os
import subprocess
import tempfile
import unittest
import zipfile
from datetime import date
from pathlib import Path
from unittest import mock

# Third Party Imports
# Local Imports
from dokime_tools.release import layout, notices, policy, sbom

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
            (root / "LICENSE").write_text("fixture license")
            (destination / "distribution.json").write_text(json.dumps({"files": {"existing": "digest"}}))
            for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                (root / name).write_bytes(name.encode())
            notices.stage(root, destination)
            files = json.loads((destination / "distribution.json").read_text())["files"]
            self.assertEqual(files["existing"], "digest")
            for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                self.assertEqual(files[name], hashlib.sha256((destination / name).read_bytes()).hexdigest())

    def test_expiry_refuses_the_expiry_date(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "supply-chain-exceptions.toml").write_text(
                '[[advisory]]\npackage="dependency"\nversion="1"\nid="RUSTSEC-example"\n'
                'reason="reviewed"\nowner="maintainer"\nfixed="2"\nexpires=2030-01-02\n'
            )
            document = sbom.exceptions(root, today=date(2030, 1, 1))
            self.assertIsNotNone(sbom.accepted(document, "advisory", "dependency", "1", "RUSTSEC-example"))
            self.assertIsNone(sbom.accepted(document, "advisory", "dependency", "2", "RUSTSEC-example"))
            with self.assertRaises(policy.ToolError):
                sbom.exceptions(root, today=date(2030, 1, 2))

    def test_unknown_audit_finding_cannot_use_another_versions_exception(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            report = {
                "vulnerabilities": {"list": [{"advisory": {"id": "RUSTSEC-example"}, "package": {"name": "dependency", "version": "2"}}]}
            }
            results = [
                subprocess.CompletedProcess([], 0, "", ""),
                subprocess.CompletedProcess([], 1, json.dumps(report), ""),
                subprocess.CompletedProcess([], 0, '{"advisories": {}}', ""),
                subprocess.CompletedProcess([], 0, "{}", ""),
            ]
            document = {"advisory": [{"id": "RUSTSEC-example", "package": "dependency", "version": "1"}]}
            with (
                mock.patch.object(sbom, "exceptions", return_value=document),
                mock.patch.object(sbom.subprocess, "run", side_effect=results),
                self.assertRaises(policy.ToolError),
            ):
                sbom.check(root)

    def test_wheel_notices_preserve_members_and_update_record(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                (root / name).write_bytes(name.encode())
            wheel = root / "fixture.whl"
            record = "fixture-1.dist-info/RECORD"
            with zipfile.ZipFile(wheel, "w") as output:
                output.writestr("native.pyd", b"native bytes")
                output.writestr(record, "native.pyd,,\n" + record + ",,\n")
            notices.archive(root, wheel)
            with zipfile.ZipFile(wheel) as archive:
                self.assertEqual(archive.read("native.pyd"), b"native bytes")
                rows = list(csv.reader(io.StringIO(archive.read(record).decode())))
                for name, digest, size in rows:
                    if not digest:
                        continue
                    data = archive.read(name)
                    self.assertEqual(int(size), len(data))
                    expected = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).decode().rstrip("=")
                    self.assertEqual(digest, "sha256=" + expected)

    def test_release_wrapper_preserves_profile_and_existing_cache(self):
        with (
            mock.patch.dict(os.environ, {"RUSTC_WRAPPER": "cache"}, clear=True),
            mock.patch.object(sbom.shutil, "which", return_value="auditable"),
            mock.patch.object(sbom.subprocess, "run", return_value=subprocess.CompletedProcess([], 0)) as run,
        ):
            sbom.auditable("release", ["python", "-m", "existing.build", "--profile", "release"])
        arguments = run.call_args.args[0]
        self.assertEqual(arguments[-2:], ["--profile", "release"])
        environment = run.call_args.kwargs["env"]
        self.assertEqual(environment["RUSTC_WRAPPER"], "cache")
        self.assertEqual(environment["RUSTC_WORKSPACE_WRAPPER"], "auditable")
        self.assertTrue(json.loads(environment["CARGO_AUDITABLE_ORIG_ARGS"])["locked"])

    def test_sbom_manifest_classifies_the_underlying_target(self):
        item = layout.classify({"product": "dokime"}, {}, "dokime-0.1-x86_64-pc-windows-msvc-setup.exe.cdx.json")
        self.assertEqual((item["kind"], item["os"], item["arch"]), ("sbom", "windows", "x86_64"))


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
