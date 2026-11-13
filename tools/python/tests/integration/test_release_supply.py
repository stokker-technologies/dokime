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
import os
import ssl
import subprocess
import tempfile
import tomllib
import unittest
import urllib.error
import zipfile
from datetime import date
from pathlib import Path
from unittest import mock

# Third Party Imports
# Local Imports
from dokime_tools import policy
from dokime_tools.release import layout, notices, sbom

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class SupplyTests(unittest.TestCase):
    def test_notice_config_uses_policy_and_dated_exceptions(self):
        repository = Path(__file__).resolve().parents[4]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "tools/config").mkdir(parents=True)
            (root / "tools/config/deny.toml").write_bytes((repository / "tools/config/deny.toml").read_bytes())
            (root / "tools/config/supply-chain-exceptions.toml").write_text(
                '[[license]]\npackage="dependency"\nversion="1"\nlicense="NCSA"\n'
                'reason="reviewed"\nowner="maintainer"\nfixed="2"\nexpires=2030-01-02\n'
            )
            document = sbom.exceptions(root, today=date(2030, 1, 1))
            metadata = {"packages": [{"name": "dependency", "version": "1"}]}
            with (
                mock.patch.object(sbom, "exceptions", return_value=document),
                mock.patch.object(policy, "execute", side_effect=[json.dumps(metadata), RuntimeError("stop before collection")]),
                self.assertRaisesRegex(RuntimeError, "stop before collection"),
            ):
                notices.generate(root)
            config = tomllib.loads((root / "target/notices/about.toml").read_text(encoding="utf-8"))
            license_policy = tomllib.loads((root / "tools/config/deny.toml").read_text(encoding="utf-8"))
            self.assertEqual(config["accepted"], license_policy["licenses"]["allow"])
            self.assertEqual(config["dependency"]["accepted"], ["NCSA"])

    def test_notice_scope_without_installed_shell(self):
        for installed in (False, True):
            with self.subTest(installed=installed), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                output = root / "target/notices"
                output.mkdir(parents=True)
                (root / "tools/config").mkdir(parents=True)
                (root / "tools/config/deny.toml").write_text('[licenses]\nallow = ["MIT"]\n')
                (root / "uv.lock").write_text("package = []\n")
                (root / "NOTICE").write_text("notice")
                (output / "THIRD-PARTY-NOTICES").write_text("product")
                (output / "rust.json").write_text('{"licenses": []}')
                (output / "rust.txt").write_text("rust")
                if installed:
                    (root / "crates/dokime-gui/node_modules").mkdir(parents=True)
                with (
                    mock.patch.object(sbom, "exceptions", return_value={}),
                    mock.patch.object(policy, "execute", return_value='{"packages": []}') as execute,
                    mock.patch.object(notices, "shell_notices", return_value=("shell", 3)) as shell,
                    mock.patch.object(notices, "python_notices", return_value=("python", 2)),
                ):
                    if installed:
                        counts = notices.generate(root)
                        self.assertEqual(counts, {"rust": 0, "shell": 3, "python": 2})
                        self.assertEqual((output / "THIRD-PARTY-NOTICES").read_text(), "rust\n\nshell\n\npython\n")
                        shell.assert_called_once()
                        self.assertEqual(execute.call_args.args[1][0], "pnpm")
                    else:
                        with self.assertRaises(policy.ToolError):
                            notices.generate(root)
                    if not installed:
                        shell.assert_not_called()
                        self.assertTrue(all(call.args[1][0] == "cargo" for call in execute.call_args_list))
                self.assertFalse((root / "THIRD-PARTY-NOTICES").exists())
                if not installed:
                    self.assertEqual((output / "THIRD-PARTY-NOTICES").read_text(), "product")

    def test_shell_notice_missing_package_is_tool_error(self):
        with tempfile.TemporaryDirectory() as temporary:
            document = {"MIT": [{"name": "missing", "version": "1", "paths": [str(Path(temporary) / "missing")]}]}
            with self.assertRaises(policy.ToolError):
                notices.shell_notices(document)

    def test_notice_exception_requires_every_resolved_version(self):
        with tempfile.TemporaryDirectory() as temporary:
            document = {"license": [{"package": "dependency", "version": "1", "license": "NCSA"}]}
            metadata = {"packages": [{"name": "dependency", "version": version} for version in ("1", "2")]}
            with (
                mock.patch.object(sbom, "exceptions", return_value=document),
                mock.patch.object(policy, "execute", return_value=json.dumps(metadata)),
                self.assertRaisesRegex(policy.ToolError, "does not cover every resolved dependency version"),
            ):
                notices.generate(Path(temporary))

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
            mock.patch.object(notices.os, "walk", return_value=[]),
            mock.patch.object(policy, "execute", return_value="unused-cache"),
            mock.patch("urllib.request.urlopen", return_value=io.BytesIO(b"changed")),
            self.assertRaisesRegex(policy.ToolError, "hash mismatch.*https://example.invalid/wheel"),
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
        for cached in (False, True):
            with self.subTest(cached=cached), tempfile.TemporaryDirectory() as temporary:
                if cached:
                    (Path(temporary) / hashlib.sha256(data).hexdigest()).write_bytes(data)
                with (
                    mock.patch.dict(os.environ, {"UV_CACHE_DIR": temporary}),
                    mock.patch("urllib.request.urlopen", side_effect=[ssl.SSLEOFError("dropped"), io.BytesIO(data)]) as opener,
                    mock.patch.object(notices.time, "sleep") as sleep,
                ):
                    text, count = notices.python_notices(lock)
                self.assertEqual(count, 1)
                self.assertIn("fixture license text", text)
                self.assertEqual(opener.call_count, 0 if cached else 2)
                self.assertEqual(sleep.call_count, 0 if cached else 1)

    def test_notice_download_http_status_and_retry_exhaustion(self):
        url = "https://example.invalid/license"
        for status, attempts in ((404, 1), (503, 3)):
            with (
                self.subTest(status=status),
                mock.patch.object(notices, "DOWNLOAD_ATTEMPTS", 3),
                mock.patch("urllib.request.urlopen", side_effect=urllib.error.HTTPError(url, status, "failure", {}, None)) as opener,
                mock.patch.object(notices.time, "sleep") as sleep,
                self.assertRaisesRegex(policy.ToolError, f"{url} after {attempts} attempts:.*{status}"),
            ):
                notices.download(url)
            self.assertEqual(opener.call_count, attempts)
            self.assertEqual(sleep.call_count, attempts - 1)

    def test_shell_notice_download_retries_read_failure(self):
        url = "https://example.invalid/license"
        response = mock.MagicMock()
        response.__enter__.return_value.read.side_effect = TimeoutError("dropped body")
        with (
            tempfile.TemporaryDirectory() as temporary,
            mock.patch.dict(notices.SHELL_LICENSE_SOURCES, {("dependency", "1"): url}),
            mock.patch("urllib.request.urlopen", side_effect=[response, io.BytesIO(b"license bytes")]) as opener,
            mock.patch.object(notices.time, "sleep"),
        ):
            text, count = notices.shell_notices({"MIT": [{"name": "dependency", "version": "1", "paths": [temporary]}]})
        self.assertEqual(count, 1)
        self.assertIn("license bytes", text)
        self.assertEqual(opener.call_count, 2)

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
            generated = root / "target/notices"
            generated.mkdir(parents=True)
            for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                (generated / name).write_bytes(name.encode())
            notices.stage(root, destination)
            self.assertEqual((destination / "LICENSE").read_text(), "fixture license\n\nNOTICE\n\nTHIRD-PARTY-NOTICES")
            files = json.loads((destination / "distribution.json").read_text())["files"]
            self.assertEqual(files["existing"], "digest")
            for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                self.assertEqual(files[name], hashlib.sha256((destination / name).read_bytes()).hexdigest())

    def test_expiry_refuses_the_expiry_date(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "tools/config").mkdir(parents=True)
            (root / "tools/config/supply-chain-exceptions.toml").write_text(
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
