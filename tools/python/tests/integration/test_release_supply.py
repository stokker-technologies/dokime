# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Supply Tests."""

# === Imports === #

# Standard Library Imports
import email.message
import hashlib
import io
import json
import os
import ssl
import subprocess
import sys
import tempfile
import tomllib
import unittest
import urllib.error
import urllib.request
import zipfile
from datetime import date
from pathlib import Path
from unittest import mock

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.release import layout, notices, sbom

from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class SupplyTests(unittest.TestCase):
    def test_notice_nightly_unaccepted_licenses_include_text_and_warnings(self) -> None:
        root, execute, _ = self.notice_tools(rejected=True)
        summary = root / "summary.md"
        errors = io.StringIO()
        self.enterContext(mock.patch.dict(os.environ, {"GITHUB_STEP_SUMMARY": str(summary)}))
        self.enterContext(mock.patch.object(sys, "stderr", errors))
        self.enterContext(mock.patch.object(sys, "argv", ["notices"]))
        self.enterContext(mock.patch.dict(os.environ, {"RELEASE_CHANNEL": "nightly"}))
        self.enterContext(mock.patch.object(policy, "root", return_value=root))
        self.assertEqual(notices.main(), 0)
        text = (root / "target/notices/THIRD-PARTY-NOTICES").read_text()
        self.assertIn("Unicode License Text", text)
        self.assertIn("BSD License Text", text)
        for license_name in ("Unicode-3.0", "BSD-3-Clause"):
            self.assertIn(f"dependency 1: {license_name}", errors.getvalue())
            self.assertIn(f"dependency 1: {license_name}", summary.read_text())
        self.assertEqual(errors.getvalue().count("::warning::"), 2)
        self.assertEqual((root / "tools/config/deny.toml").read_text(), '[licenses]\nallow = ["MIT"]\n')
        self.assertEqual(sum("about" in call.args[1] for call in execute.call_args_list), 3)

    def test_notice_stable_unaccepted_license_preserves_tool_error(self) -> None:
        for channel in ("stable", None, "ci"):
            with self.subTest(channel=channel):
                root, execute, failure = self.notice_tools(rejected=True)
                self.enterContext(mock.patch.dict(os.environ, {} if channel is None else {"RELEASE_CHANNEL": channel}, clear=True))
                self.enterContext(mock.patch.object(sys, "argv", ["notices"]))
                self.enterContext(mock.patch.object(policy, "root", return_value=root))
                with self.assertRaises(policy.ToolError) as raised:
                    notices.main()
                self.assertIs(raised.exception, failure)
                self.assertFalse((root / "target/notices/THIRD-PARTY-NOTICES").exists())
                self.assertEqual(sum("about" in call.args[1] for call in execute.call_args_list), 1)

    def test_notice_accepted_licenses_match_on_both_channels(self) -> None:
        contents = []
        for strict in (True, False):
            with self.subTest(strict=strict):
                root, execute, _ = self.notice_tools(rejected=False)
                warning = self.enterContext(mock.patch.object(notices, "_warning"))
                counts = notices.generate(root, strict=strict)
                contents.append((root / "target/notices/THIRD-PARTY-NOTICES").read_bytes())
                self.assertEqual(counts, {"rust": 1, "shell": 0, "python": 0})
                self.assertEqual(sum("about" in call.args[1] for call in execute.call_args_list), 2)
                warning.assert_not_called()
        self.assertEqual(contents[0], contents[1])

    def test_notice_nightly_does_not_hide_other_tool_failures(self) -> None:
        root, execute, _ = self.notice_tools(rejected=False)
        failure = policy.ToolError("registry unavailable")
        execute.side_effect = [json.dumps({"packages": []}), failure]
        with self.assertRaises(policy.ToolError) as raised:
            notices.generate(root, strict=False)
        self.assertIs(raised.exception, failure)

    def test_notice_nightly_reports_invalid_exceptions_without_blocking(self) -> None:
        root, _, _ = self.notice_tools(rejected=True)
        self.enterContext(mock.patch.object(sbom, "exceptions", side_effect=policy.ToolError("expired exception: dependency")))
        warning = self.enterContext(mock.patch.object(notices, "_warning"))
        counts = notices.generate(root, strict=False)
        self.assertEqual(counts["rust"], 1)
        self.assertEqual(warning.call_count, 3)

    def test_notice_nightly_reports_exception_version_mismatch(self) -> None:
        root, _, _ = self.notice_tools(rejected=True)
        document = {"license": [{"package": "dependency", "version": "2", "license": "NCSA"}]}
        metadata = {"packages": [{"name": "dependency", "version": "2", "manifest_path": str(root / "Cargo.toml")}]}
        metadata["packages"].append({"name": "dependency", "version": "3", "manifest_path": str(root / "other/Cargo.toml")})
        (root / "metadata.json").write_text(json.dumps(metadata))
        self.enterContext(mock.patch.object(sbom, "exceptions", return_value=document))
        warning = self.enterContext(mock.patch.object(notices, "_warning"))
        notices.generate(root, strict=False)
        self.assertEqual(warning.call_count, 3)

    def test_notice_rejected_spans_include_with_requirements_and_multiple_crates(self) -> None:
        root = Path.cwd()
        packages = [
            {"name": "first", "version": "1", "manifest_path": str(root / "first/Cargo.toml")},
            {"name": "second", "version": "2", "manifest_path": str(root / "second/Cargo.toml")},
        ]
        diagnostics = ""
        for package in packages:
            requirement = "Apache-2.0 WITH LLVM-exception"
            diagnostics += (
                "error: failed to satisfy license requirements\n"
                f"  ┌─ {package['manifest_path']}:4:12\n"
                f'4 │ license = "{requirement}"\n'
                f"  │            {'-' * len(requirement)}\n\n"
            )
        self.assertEqual(
            notices._findings(root, diagnostics, packages),
            [("first", "1", requirement), ("second", "2", requirement)],
        )

    def test_notice_unreadable_rejections_fail_instead_of_omitting_notices(self) -> None:
        for diagnostics in (
            "error: failed to satisfy license requirements\n",
            "error: failed to satisfy license requirements\n  ┌─ missing/Cargo.toml:4:12\n",
        ):
            with self.subTest(diagnostics=diagnostics), self.assertRaises(policy.ToolError):
                notices._findings(Path.cwd(), diagnostics, [])

    def notice_tools(self, *, rejected: bool) -> tuple[Path, mock.Mock, policy.ToolError]:
        temporary_directory = tempfile.TemporaryDirectory()
        self.addCleanup(temporary_directory.cleanup)
        root = Path(temporary_directory.name)
        (root / "tools/config").mkdir(parents=True)
        (root / "crates/dokime-gui/node_modules").mkdir(parents=True)
        (root / "tools/config/deny.toml").write_text('[licenses]\nallow = ["MIT"]\n')
        (root / "uv.lock").write_text("package = []\n")
        (root / "NOTICE").write_text("notice")
        metadata = {"packages": [{"name": "dependency", "version": "1", "manifest_path": str(root / "Cargo.toml")}]}
        (root / "metadata.json").write_text(json.dumps(metadata))
        requirements = ["Unicode-3.0", "BSD-3-Clause"] if rejected else ["MIT"]
        expression = " AND ".join(requirements)
        failure = policy.ToolError(
            "error: failed to satisfy license requirements\n"
            f"  ┌─ {root / 'Cargo.toml'}:4:12\n"
            f'4 │ license = "{expression}"\n'
            f"  │            {'-' * len(requirements[0])}     {'-' * len(requirements[-1])}\n\n"
        )

        def execute(directory: Path, command: list[str], *, capture: bool = False) -> str:
            if command[:2] == ["cargo", "metadata"]:
                return (root / "metadata.json").read_text()
            if command[0] == "pnpm":
                return "{}"
            self.assertIn("about", command)
            config = tomllib.loads((root / "target/notices/about.toml").read_text())
            allowed = set(config["accepted"]) | set(config.get("dependency", {}).get("accepted", []))
            if not set(requirements) <= allowed:
                raise failure
            destination = Path(command[command.index("--output-file") + 1])
            if destination.suffix == ".json":
                data = {"licenses": [{"used_by": [{"crate": {"name": "dependency", "version": "1"}}]}]}
                destination.write_text(json.dumps(data))
            else:
                self.assertIn("tools/config/about.hbs", command)
                destination.write_text("Unicode License Text\nBSD License Text" if rejected else "MIT License Text")
            return ""

        self.enterContext(mock.patch.object(sbom, "exceptions", return_value={}))
        self.enterContext(mock.patch.object(notices, "shell_notices", return_value=("shell", 0)))
        self.enterContext(mock.patch.object(notices, "python_notices", return_value=("python", 0)))
        mocked = self.enterContext(mock.patch.object(policy, "execute", side_effect=execute))
        return root, mocked, failure

    def test_notice_config_uses_policy_and_dated_exceptions(self) -> None:
        repository = Path(__file__).resolve().parents[4]
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
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
        finally:
            temporary_directory.cleanup()

    def test_notice_scope_without_installed_shell(self) -> None:
        for installed in (False, True):
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(installed=installed):
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
            finally:
                temporary_directory.cleanup()

    def test_shell_notice_missing_package_is_tool_error(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            document = {"MIT": [{"name": "missing", "version": "1", "paths": [str(Path(temporary) / "missing")]}]}
            with self.assertRaises(policy.ToolError):
                notices.shell_notices(document)
        finally:
            temporary_directory.cleanup()

    def test_notice_exception_requires_every_resolved_version(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            document = {"license": [{"package": "dependency", "version": "1", "license": "NCSA"}]}
            metadata = {"packages": [{"name": "dependency", "version": version} for version in ("1", "2")]}
            with (
                mock.patch.object(sbom, "exceptions", return_value=document),
                mock.patch.object(policy, "execute", return_value=json.dumps(metadata)),
                self.assertRaisesRegex(policy.ToolError, "does not cover every resolved dependency version"),
            ):
                notices.generate(Path(temporary))
        finally:
            temporary_directory.cleanup()

    def test_archive_traversal_is_rejected_before_extraction(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            archive = root / "input.zip"
            output = zipfile.ZipFile(archive, "w")
            try:
                output.writestr("safe.txt", "content")
                output.writestr("../outside.txt", "content")
            finally:
                output.close()
            destination = root / "unpacked"
            destination.mkdir()
            with self.assertRaises(policy.ToolError):
                sbom.unpack(archive, destination)
            self.assertEqual(list(destination.iterdir()), [])
            self.assertFalse((root / "outside.txt").exists())
        finally:
            temporary_directory.cleanup()

    def test_artifact_records_digest_and_deduplicates_components(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            source = root / "cli.zip"
            output = zipfile.ZipFile(source, "w")
            try:
                output.writestr("dokime", b"binary")
            finally:
                output.close()
            cargo = root / "cargo.json"
            component = {"type": "library", "name": "dependency", "purl": "pkg:cargo/dependency@1.0"}
            cargo.write_text(json.dumps({"components": [component]}))
            scan = {"bomFormat": "CycloneDX", "specVersion": "1.5", "version": 1, "components": [component]}
            with mock.patch.object(policy, "execute", return_value=json.dumps(scan)):
                result = sbom.artifact(root, source, root / "output", [cargo])
            document = json.loads(result.read_text())
            self.assertEqual(len(document["components"]), 1)
            self.assertEqual(document["metadata"]["component"]["hashes"][0]["content"], hashlib.sha256(source.read_bytes()).hexdigest())
        finally:
            temporary_directory.cleanup()

    def test_python_wheel_hash_mismatch_is_rejected(self) -> None:
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

    def test_python_notices_use_verified_license_files(self) -> None:
        content = io.BytesIO()
        output = zipfile.ZipFile(content, "w")
        try:
            output.writestr("dependency-1.dist-info/METADATA", "Name: dependency\nVersion: 1\nLicense-Expression: MIT\n")
            output.writestr("dependency-1.dist-info/licenses/LICENSE", "fixture license text")
        finally:
            output.close()
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
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(cached=cached):
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
            finally:
                temporary_directory.cleanup()

    def test_python_notices_verify_versioned_upstream_license_sources(self) -> None:
        license_data = bytes(range(32, 127)) + b"\n"
        source_url = "https://example.invalid/source-license"
        for version, source_data in (("1", license_data), ("1", b"changed"), ("2", license_data)):
            temporary_directory = tempfile.TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(version=version, source_data=source_data):
                    content = io.BytesIO()
                    output = zipfile.ZipFile(content, "w")
                    try:
                        output.writestr("dependency.dist-info/METADATA", f"Name: dependency\nVersion: {version}\nLicense-Expression: MIT\n")
                    finally:
                        output.close()
                    data = content.getvalue()
                    lock = {
                        "package": [
                            {
                                "name": "dependency",
                                "version": version,
                                "source": {"registry": "https://pypi.org/simple"},
                                "wheels": [{"url": "https://example.invalid/wheel", "hash": "sha256:" + hashlib.sha256(data).hexdigest()}],
                            }
                        ]
                    }
                    with (
                        mock.patch.dict(os.environ, {"UV_CACHE_DIR": temporary}),
                        mock.patch.dict(
                            notices.PYTHON_LICENSE_SOURCES,
                            {("dependency", "1"): (source_url, hashlib.sha256(license_data).hexdigest())},
                            clear=True,
                        ),
                        mock.patch("urllib.request.urlopen", side_effect=[io.BytesIO(data), io.BytesIO(source_data)]) as opener,
                    ):
                        if version == "1" and source_data == license_data:
                            text, count = notices.python_notices(lock)
                            self.assertEqual(count, 1)
                            self.assertIn(license_data.decode("utf-8"), text)
                        else:
                            with self.assertRaises(policy.ToolError):
                                notices.python_notices(lock)
                    self.assertEqual(opener.call_count, 2 if version == "1" else 1)
            finally:
                temporary_directory.cleanup()

    def test_notice_download_http_status_and_retry_exhaustion(self) -> None:
        url = "https://example.invalid/license"
        for status, attempts in ((404, 1), (503, 3)):
            with (
                self.subTest(status=status),
                mock.patch.object(notices, "DOWNLOAD_ATTEMPTS", 3),
                mock.patch(
                    "urllib.request.urlopen", side_effect=urllib.error.HTTPError(url, status, "failure", email.message.Message(), None)
                ) as opener,
                mock.patch.object(notices.time, "sleep") as sleep,
                self.assertRaisesRegex(policy.ToolError, f"{url} after {attempts} attempts:.*{status}"),
            ):
                notices.download(url)
            self.assertEqual(opener.call_count, attempts)
            self.assertEqual(sleep.call_count, attempts - 1)

    def test_shell_notice_download_retries_read_failure(self) -> None:
        url = "https://example.invalid/license"
        response = mock.MagicMock()
        response.read.side_effect = TimeoutError("dropped body")
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            with (
                mock.patch.dict(notices.SHELL_LICENSE_SOURCES, {("dependency", "1"): url}),
                mock.patch("urllib.request.urlopen", side_effect=[response, io.BytesIO(b"license bytes")]) as opener,
                mock.patch.object(notices.time, "sleep"),
            ):
                text, count = notices.shell_notices({"MIT": [{"name": "dependency", "version": "1", "paths": [temporary]}]})
        finally:
            temporary_directory.cleanup()
        self.assertEqual(count, 1)
        self.assertIn("license bytes", text)
        self.assertEqual(opener.call_count, 2)

    def test_shell_missing_license_text_fails(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            with self.assertRaises(policy.ToolError):
                notices.shell_notices({"MIT": [{"name": "dependency", "versions": ["1"], "paths": [temporary]}]})
        finally:
            temporary_directory.cleanup()

    def test_staged_notice_hashes_match_bytes(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
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
        finally:
            temporary_directory.cleanup()

    def test_expiry_refuses_the_expiry_date(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
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
        finally:
            temporary_directory.cleanup()

    def test_unknown_audit_finding_cannot_use_another_versions_exception(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
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
        finally:
            temporary_directory.cleanup()

    def test_release_wrapper_preserves_profile_and_existing_cache(self) -> None:
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

    def test_sbom_manifest_classifies_the_underlying_target(self) -> None:
        item = layout.classify({"product": "dokime"}, {}, "dokime-0.1-x86_64-pc-windows-msvc-setup.exe.cdx.json")
        self.assertEqual((item["kind"], item["os"], item["arch"]), ("sbom", "windows", "x86_64"))


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
