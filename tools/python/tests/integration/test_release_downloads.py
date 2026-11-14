# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify The Unified Release Index And Staged Asset Inventory."""

# === Imports === #

# Standard Library Imports
import hashlib
import io
import json
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

# Third Party Imports
from harmos_tools.release import policy

# Local Imports
from dokime_tools.build import wheel
from dokime_tools.package import run as package_run
from dokime_tools.release import layout, registry, reports, run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

SETTINGS = {"product": "dokime", "provider": "github", "repository": "example/dokime"}
EXECUTABLES = {"dokime-cli": "cli", "companion-tool": "companion-tool"}
EXPECTED = {
    "CHANGELOG.md": ("changelog", "any", "any", "md"),
    "SHA256SUMS": ("checksums", "any", "any", "txt"),
    "companion-tool-0.1.3-x86_64-pc-windows-msvc.zip": ("companion-tool", "windows", "x86_64", "zip"),
    "companion-tool-0.1.3-x86_64-unknown-linux-gnu.tar.gz": ("companion-tool", "linux", "x86_64", "tar.gz"),
    "dokime-0.1.3-1.x86_64.rpm": ("installer", "linux", "x86_64", "rpm"),
    "dokime-0.1.3-x86_64-pc-windows-msvc-setup.exe": ("installer", "windows", "x86_64", "exe"),
    "dokime-cli-0.1.3-x86_64-pc-windows-msvc.zip": ("cli", "windows", "x86_64", "zip"),
    "dokime-cli-0.1.3-x86_64-unknown-linux-gnu.tar.gz": ("cli", "linux", "x86_64", "tar.gz"),
    "dokime-distribution-0.1.3-x86_64-pc-windows-msvc.zip": ("payload", "windows", "x86_64", "zip"),
    "dokime-distribution-0.1.3-x86_64-unknown-linux-gnu.zip": ("payload", "linux", "x86_64", "zip"),
    "dokime-guests-0.1.3.zip": ("plugin", "any", "any", "zip"),
    "dokime-plugins-0.1.3-x86_64-pc-windows-msvc.zip": ("plugin", "windows", "x86_64", "zip"),
    "dokime-plugins-0.1.3-x86_64-unknown-linux-gnu.tar.gz": ("plugin", "linux", "x86_64", "tar.gz"),
    "dokime_0.1.3_amd64.deb": ("installer", "linux", "x86_64", "deb"),
    "dokime_sdk-0.1.3-py3-none-any.whl": ("sdk-python", "any", "any", "whl"),
    "dokime_sdk-0.1.3.tar.gz": ("sdk-python", "any", "any", "tar.gz"),
    "dokime-compliance-0.1.3.zip": ("compliance", "any", "any", "zip"),
    "dokime-proto-0.1.3.zip": ("proto", "any", "any", "zip"),
    "dokime-reports-0.1.3.zip": ("reports", "any", "any", "zip"),
    "unexpected.bin": ("other", "any", "any", "bin"),
}

# === Types === #

# === Classes === #


class DownloadManifestTests(unittest.TestCase):
    def manifest(self, tag: str, channel: str) -> dict:
        files = [{"name": name, "sha256": f"{index:064x}", "size": index} for index, name in enumerate(sorted(EXPECTED))]
        sums = "".join(f"{item['sha256']}  {item['name']}\n" for item in files if item["name"] != "SHA256SUMS")
        return layout.downloads(
            SETTINGS,
            EXECUTABLES,
            version="0.1.3",
            channel=channel,
            tag=tag,
            published="2026-11-08T12:00:00Z",
            files=files,
            sums=sums,
        )

    def test_every_asset_family_is_classified(self):
        manifest = self.manifest("v0.1.3", "stable")
        actual = {item["name"]: (item["kind"], item["os"], item["arch"], item["format"]) for item in manifest["artifacts"]}
        self.assertEqual(actual, EXPECTED)
        self.assertEqual(
            {key: manifest[key] for key in ("version", "channel", "tag", "pub_date")},
            {"version": "0.1.3", "channel": "stable", "tag": "v0.1.3", "pub_date": "2026-11-08T12:00:00Z"},
        )

    def test_digests_come_from_sha256sums_and_sizes_from_uploads(self):
        files = [{"name": "dokime_0.1.3_amd64.deb", "sha256": "a" * 64, "size": 7}, {"name": "SHA256SUMS", "sha256": "b" * 64, "size": 3}]
        manifest = layout.downloads(
            SETTINGS,
            EXECUTABLES,
            version="0.1.3",
            channel="stable",
            tag="v0.1.3",
            published="2026-11-08T12:00:00Z",
            files=files,
            sums=f"{'c' * 64}  dokime_0.1.3_amd64.deb\n",
        )
        digests = {item["name"]: (item["sha256"], item["size"]) for item in manifest["artifacts"]}
        self.assertEqual(digests, {"dokime_0.1.3_amd64.deb": ("c" * 64, 7), "SHA256SUMS": ("b" * 64, 3)})

    def test_unlisted_upload_is_refused(self):
        with self.assertRaises(policy.ToolError):
            layout.downloads(
                SETTINGS,
                EXECUTABLES,
                version="0.1.3",
                channel="stable",
                tag="v0.1.3",
                published="2026-11-08T12:00:00Z",
                files=[{"name": "dokime_0.1.3_amd64.deb", "sha256": "a" * 64, "size": 1}],
                sums="",
            )

    def test_urls_compose_from_the_channel_tag(self):
        for tag, channel in (("v0.1.3", "stable"), ("nightly", "nightly")):
            manifest = self.manifest(tag, channel)
            for item in manifest["artifacts"]:
                self.assertEqual(item["url"], f"https://github.com/example/dokime/releases/download/{tag}/{item['name']}")

    def test_gitlab_urls_use_release_permalinks(self):
        settings = {"product": "dokime", "provider": "gitlab", "project_id": "group/dokime", "api_url": "https://gitlab.example/api/v4"}
        self.assertEqual(
            layout.download_url(settings, "nightly", "SHA256SUMS"),
            "https://gitlab.example/group/dokime/-/releases/nightly/downloads/SHA256SUMS",
        )


class StageTests(unittest.TestCase):
    def test_stage_requires_packaged_notices(self):
        for missing_archive in (True, False):
            with self.subTest(missing_archive=missing_archive), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                package = root / "target/package/release/windows-x86_64/rust/dokime-distribution-0.1.3-x86_64-pc-windows-msvc.zip"
                if not missing_archive:
                    package.parent.mkdir(parents=True)
                    with zipfile.ZipFile(package, "w") as bundle:
                        bundle.writestr("NOTICE", b"fixture")
                with (
                    mock.patch.object(run.layout, "triple", return_value="x86_64-pc-windows-msvc"),
                    self.assertRaises(policy.ToolError) as failure,
                ):
                    run.stage(root, SETTINGS, "v0.1.3", ["windows-x86_64"], profile="release", release_notes="notes")
                self.assertIn(str(package), str(failure.exception))
                if not missing_archive:
                    self.assertIn("THIRD-PARTY-NOTICES", str(failure.exception))

    def test_packaged_notices_must_match_between_platforms(self):
        for matching in (True, False):
            with self.subTest(matching=matching), tempfile.TemporaryDirectory() as directory:
                sources = [Path(directory) / name for name in ("windows.zip", "linux.zip")]
                for index, source in enumerate(sources):
                    with zipfile.ZipFile(source, "w") as bundle:
                        bundle.writestr("NOTICE", b"fixture")
                        bundle.writestr("THIRD-PARTY-NOTICES", b"licenses" if matching or index == 0 else b"different")
                if matching:
                    self.assertEqual(run.notices.packaged(sources), {"NOTICE": b"fixture", "THIRD-PARTY-NOTICES": b"licenses"})
                else:
                    with self.assertRaises(policy.ToolError) as failure:
                        run.notices.packaged(sources)
                    for source in sources:
                        self.assertIn(str(source), str(failure.exception))

    def test_stage_publishes_one_index_and_exact_layout(self):
        for channel, binary in (("stable", "dokime"), ("nightly", "dokime.exe")):
            with self.subTest(channel=channel, binary=binary), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                feed = {
                    "version": "0.1.3",
                    "platforms": {"windows-x86_64": {"url": "https://example.test/setup.exe", "signature": "signed"}},
                }
                package = root / "target/package/release/windows-x86_64/rust/dokime-distribution-0.1.3-x86_64-pc-windows-msvc.zip"
                package.parent.mkdir(parents=True)
                with zipfile.ZipFile(package, "w") as bundle:
                    for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
                        bundle.writestr(name, b"fixture\n")
                python = root / "target/package/release/any/python"
                python.mkdir(parents=True)
                for name in ("dokime_sdk-0.1.3-py3-none-any.whl", "dokime_sdk-0.1.3.tar.gz"):
                    (python / name).write_bytes(b"packaged Python")
                shell_inventory = root / "target/release-assets/any/dokime-shell.cdx.json"

                def artifact(_, source, output, inventories):
                    destination = output / (source.name + ".cdx.json")
                    destination.write_text('{"components": []}')
                    return destination

                def shell(*_, shell_inventory=shell_inventory):
                    shell_inventory.write_text('{"components": []}')
                    return shell_inventory

                def export(_, command):
                    self.assertTrue(Path(command[0]).is_file())
                    exported = Path(command[-1])
                    for name in ("dokime/v1/execution.proto", "harmos/v1/plugin.proto"):
                        path = exported / name
                        path.parent.mkdir(parents=True)
                        path.write_text('syntax = "proto3";')

                source = root / "input/latest.json"
                source.parent.mkdir()
                source.write_text(json.dumps(feed))
                installer = root / "input/dokime-0.1.3-x86_64-pc-windows-msvc-setup.exe"
                installer.write_bytes(b"installer")
                declared = [
                    {"name": "latest.json", "path": "input/latest.json", "target": "windows-x86_64"},
                    {"name": installer.name, "path": "input/" + installer.name, "target": "windows-x86_64"},
                ]
                settings = {**SETTINGS, "targets": ["windows-x86_64"]}
                config = {"package": {"executables": [{"name": "dokime-cli", "command": "dokime-next"}]}}
                with (
                    mock.patch.object(run.sbom, "shell", side_effect=shell),
                    mock.patch.object(run.sbom, "workspace", return_value=[]),
                    mock.patch.object(run.sbom, "artifact", side_effect=artifact),
                    mock.patch.object(run.layout, "triple", return_value="x86_64-pc-windows-msvc"),
                    mock.patch.object(run, "assets", return_value=declared),
                    mock.patch.object(run, "_native", return_value="windows-x86_64"),
                    mock.patch.object(run.packages, "verify"),
                    mock.patch.object(
                        run.packages, "definitions", return_value=[{"command": "dokime-next", "binary": binary, "output": "cli.zip"}]
                    ),
                    mock.patch.object(run.packages, "binary_bytes", return_value=b"cli"),
                    mock.patch.object(run.policy, "execute", side_effect=export),
                    mock.patch.object(run.policy, "config", return_value=config),
                    mock.patch.object(run.version, "run"),
                    mock.patch.object(run, "notes", return_value="notes"),
                    mock.patch.object(run, "release_tag", return_value="v0.1.3"),
                    mock.patch.object(run, "_revision", return_value="a" * 40),
                    mock.patch.object(run, "continuous_integration", return_value=False),
                ):
                    files = run.stage(
                        root, settings, "v0.1.3", ["windows-x86_64"], profile="release", release_notes="notes\n", channel=channel
                    )
                    if channel == "stable":
                        run.rehearse(root, settings, target="windows-x86_64", profile="release", stage_only=True)
                        files = layout.uploads(root, staged=True)
                staged = root / run.ASSETS
                expected = {
                    "dokime_sdk-0.1.3-py3-none-any.whl",
                    "dokime_sdk-0.1.3.tar.gz",
                    installer.name,
                    "dokime-compliance-0.1.3.zip",
                    "dokime-proto-0.1.3.zip",
                    "release.json",
                    "SHA256SUMS",
                }
                if channel == "nightly":
                    expected.update({"dokime-reports-0.1.3.zip", "benchmarks.json"})
                self.assertEqual({item["name"] for item in files}, expected)
                document = json.loads((staged / run.SHARED / run.INDEX).read_text())
                self.assertEqual(document["platforms"], feed["platforms"])
                self.assertEqual(document["plugins"], {"format_version": 2, "plugins": []})
                manifest = document["downloads"]
                self.assertEqual((manifest["channel"], manifest["tag"]), (channel, "v0.1.3" if channel == "stable" else "nightly"))
                self.assertEqual({item["name"] for item in manifest["artifacts"]}, expected - {"release.json", "SHA256SUMS"})
                with zipfile.ZipFile(staged / run.SHARED / "dokime-compliance-0.1.3.zip") as archive:
                    self.assertIn("NOTICE", archive.namelist())
                    self.assertIn("THIRD-PARTY-NOTICES", archive.namelist())
                    self.assertIn("artifacts/" + installer.name + ".cdx.json", archive.namelist())
                    self.assertIn("crates/any/dokime-shell.cdx.json", archive.namelist())
                with zipfile.ZipFile(staged / run.SHARED / "dokime-proto-0.1.3.zip") as archive:
                    self.assertEqual(set(archive.namelist()), {"dokime/v1/execution.proto", "harmos/v1/plugin.proto"})
                inventory = {item["name"]: item for item in layout.uploads(root, staged=True)}
                run._checksums(staged / run.SHARED / run.SUMS, inventory)


class PythonPackagingTests(unittest.TestCase):
    def fixture(self, root):
        proto = root / "proto/dokime/v1/service.proto"
        proto.parent.mkdir(parents=True)
        proto.write_bytes(b'syntax = "proto3";')
        output = root / "target/build/dev/any/python"
        output.mkdir(parents=True)
        (output / ".gitignore").write_text("*")
        with zipfile.ZipFile(output / "dokime_sdk-0.1.3-py3-none-any.whl", "w") as archive:
            archive.writestr("dokime_sdk-0.1.3.dist-info/METADATA", "Name: dokime-sdk\nVersion: 0.1.3\n")
            archive.writestr("dokime_sdk-0.1.3.dist-info/WHEEL", "Root-Is-Purelib: true\nTag: py3-none-any\n")
            archive.writestr(
                "dokime/v1/sources.json", json.dumps({"dokime/v1/service.proto": hashlib.sha256(proto.read_bytes()).hexdigest()})
            )
            for name in ("surface.pb", "service_pb2.py", "service_pb2.pyi", "service_pb2_grpc.py"):
                archive.writestr("dokime/v1/" + name, b"generated")
        with tarfile.open(output / "dokime_sdk-0.1.3.tar.gz", "w:gz") as archive:
            member = tarfile.TarInfo("dokime_sdk-0.1.3/proto/dokime/v1/service.proto")
            member.size = len(proto.read_bytes())
            archive.addfile(member, io.BytesIO(proto.read_bytes()))
        return output

    def test_package_copies_shared_pair_without_rebuilding(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            output = self.fixture(root)
            with (
                mock.patch.object(package_run.Path, "cwd", return_value=root),
                mock.patch.object(package_run.policy, "config", return_value={"version": {"value": "0.1.3"}}),
                mock.patch("sys.argv", ["package", "python", "--target", "windows-x86_64"]),
                mock.patch.object(package_run.subprocess, "run") as execute,
            ):
                package_run.main()
            execute.assert_not_called()
            for source in wheel.distributions(root, output, "0.1.3"):
                self.assertEqual((root / "target/package/dev/any/python" / source.name).read_bytes(), source.read_bytes())
            self.assertFalse((root / "target/package/dev/windows-x86_64").exists())

    def test_rejects_native_missing_and_stale_protocol_wheels(self):
        for invalid in ("native", "missing", "stale", "identity", "tag", "source"):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                output = self.fixture(root)
                source = output / "dokime_sdk-0.1.3-py3-none-any.whl"
                with zipfile.ZipFile(source) as archive:
                    contents = {name: archive.read(name) for name in archive.namelist()}
                if invalid == "native":
                    contents["dokime/native.pyd"] = b"native"
                elif invalid == "missing":
                    del contents["dokime/v1/service_pb2_grpc.py"]
                elif invalid == "stale":
                    (root / "proto/dokime/v1/service.proto").write_bytes(b"changed")
                elif invalid == "identity":
                    contents["dokime_sdk-0.1.3.dist-info/METADATA"] = b"Name: other\nVersion: 0.1.3\n"
                elif invalid == "tag":
                    contents["dokime_sdk-0.1.3.dist-info/WHEEL"] = b"Root-Is-Purelib: false\nTag: cp313-abi3-win_amd64\n"
                else:
                    with tarfile.open(output / "dokime_sdk-0.1.3.tar.gz", "w:gz"):
                        pass
                with zipfile.ZipFile(source, "w") as archive:
                    for name, content in contents.items():
                        archive.writestr(name, content)
                with self.assertRaises((policy.ToolError, KeyError)):
                    wheel.verify(root, output, "0.1.3")

    def test_rejects_wrong_version_and_extra_distribution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = self.fixture(root)
            with self.assertRaises(policy.ToolError):
                wheel.distributions(root, output, "0.2.0")
            (output / "dokime-0.1.3-cp313-abi3-win_amd64.whl").write_bytes(b"old")
            with self.assertRaises(policy.ToolError):
                wheel.distributions(root, output, "0.1.3")

    def test_registry_uploads_pair_once_for_all_targets(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = self.fixture(root)
            destination = root / "target/package/dev/any/python"
            destination.parent.mkdir(parents=True)
            source.rename(destination)
            settings = {**SETTINGS, "targets": ["linux-x86_64", "windows-x86_64"]}
            for dry_run in (True, False):
                with (
                    self.subTest(dry_run=dry_run),
                    mock.patch.object(registry.policy, "profile", return_value="dev"),
                    mock.patch.object(registry.policy, "config", return_value={"version": {"value": "0.1.3"}}),
                    mock.patch.object(registry.policy, "execute") as execute,
                ):
                    registry.python(root, settings, dry_run=dry_run)
                execute.assert_called_once()
                command = execute.call_args.args[1]
                self.assertEqual(command[:2], ["uv", "publish"])
                self.assertEqual(
                    command[-2:],
                    [f"target/package/dev/any/python/{name}" for name in ("dokime_sdk-0.1.3-py3-none-any.whl", "dokime_sdk-0.1.3.tar.gz")],
                )
                self.assertEqual("--dry-run" in command, dry_run)


class NightlyReportTests(unittest.TestCase):
    def test_collects_all_suites_baselines_and_coverage(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shared = root / "shared"
            shared.mkdir()
            for suite in ("unit", "integration", "integration-plots", "system"):
                output = root / "target/nightly-reports" / suite
                output.mkdir(parents=True)
                rows = [(suite + "/fast", 10.0, 20.0), (suite + "/regressed", 40.0, 20.0), (suite + "/seeded", 1.0, None)]
                policy.write_json(output / f"benchmarks-rust-{suite}.json", reports.measurements(rows, "rust/" + suite, "a" * 40, 15))
            coverage = root / "target/nightly-reports/coverage/rust/coverage.xml"
            coverage.parent.mkdir(parents=True)
            coverage.write_text('<coverage line-rate="0.75"/>')
            outputs = reports.stage(root, shared, "0.1.3")
            self.assertEqual({path.name for path in outputs}, {"benchmarks.json", "dokime-reports-0.1.3.zip"})
            values = json.loads((shared / "benchmarks.json").read_text())["benchmarks"]
            self.assertEqual(len(values), 12)
            self.assertEqual(
                {row["suite"] for row in values}, {"rust/" + suite for suite in ("unit", "integration", "integration-plots", "system")}
            )
            self.assertTrue(all(row["unit"] == "ns" and row["revision"] == "a" * 40 for row in values))
            summary = json.loads((shared / "reports.json").read_text())
            self.assertEqual(summary["coverage"][0]["line_percent"], 75.0)
            notes = reports.table(summary, {"bench": {"result": "failure"}}, "https://example.test/run")
            self.assertIn("100.0%", notes)
            self.assertNotIn("/fast", notes)
            with zipfile.ZipFile(shared / "dokime-reports-0.1.3.zip") as archive:
                self.assertIn("summary.json", archive.namelist())
                self.assertIn("coverage/rust/coverage.xml", archive.namelist())

    def test_failed_jobs_without_reports_still_produce_assets(self):
        with tempfile.TemporaryDirectory() as directory:
            shared = Path(directory)
            partial = shared / "target/nightly-reports/coverage/python/coverage.xml"
            partial.parent.mkdir(parents=True)
            partial.write_text("<coverage")
            reports.stage(shared, shared, "0.1.3")
            summary = json.loads((shared / "reports.json").read_text())
            self.assertEqual(len(summary["errors"]), 1)
            self.assertEqual(json.loads((shared / "benchmarks.json").read_text()), {"benchmarks": []})
            self.assertTrue((shared / "dokime-reports-0.1.3.zip").is_file())


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
