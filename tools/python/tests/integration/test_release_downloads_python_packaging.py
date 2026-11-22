# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Python Packaging Tests."""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import tarfile
from io import BytesIO
from pathlib import Path
from tarfile import TarInfo
from tempfile import TemporaryDirectory
from unittest import TestCase, mock
from zipfile import ZipFile

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.package.run
from dokime_tools import policy
from dokime_tools.build import wheel
from dokime_tools.policy import ToolError
from dokime_tools.release import (
    registry,
)

from .environment import isolate_environment
from .release_downloads_fixtures import SETTINGS

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class PythonPackagingTests(TestCase):
    def fixture(self, root: Path) -> Path:
        (root / "Cargo.toml").write_text('[workspace.dependencies.harmos]\ngit = "https://example.test/library"\nrev = "pinned"\n')
        exported = root / "target/protocol"
        exported.mkdir(parents=True, exist_ok=True)
        (exported / ".revision").write_text("https://example.test/library/archive/pinned.tar.gz")
        proto = root / "proto/dokime/host/v1/service.proto"
        proto.parent.mkdir(parents=True, exist_ok=True)
        proto.write_bytes(b'syntax = "proto3";')
        output = root / "target/build/dev/any/python"
        output.mkdir(parents=True, exist_ok=True)
        (output / ".gitignore").write_text("*")
        archive = ZipFile(output / "dokime_sdk-0.1.3-py3-none-any.whl", "w")
        try:
            archive.writestr("dokime_sdk-0.1.3.dist-info/METADATA", "Name: dokime-sdk\nVersion: 0.1.3\n")
            archive.writestr("dokime_sdk-0.1.3.dist-info/WHEEL", "Root-Is-Purelib: true\nTag: py3-none-any\n")
            archive.writestr(
                "dokime_sdk/_wire/sources.json",
                json.dumps({"dokime/host/v1/service.proto": hashlib.sha256(proto.read_bytes()).hexdigest()}),
            )
            archive.writestr("dokime_sdk/_wire/surface.pb", b"generated")
            for name in ("service_pb2.py", "service_pb2.pyi", "service_pb2_grpc.py"):
                archive.writestr("dokime_sdk/_wire/host/v1/" + name, b"generated")
        finally:
            archive.close()
        archive = tarfile.open(output / "dokime_sdk-0.1.3.tar.gz", "w:gz")
        try:
            member = TarInfo("dokime_sdk-0.1.3/proto/dokime/host/v1/service.proto")
            member.size = len(proto.read_bytes())
            archive.addfile(member, BytesIO(proto.read_bytes()))
        finally:
            archive.close()
        return output

    def test_build_rejects_stale_protocol_sources(self) -> None:
        temporary_directory = TemporaryDirectory()
        try:
            root = Path(temporary_directory.name)
            self.fixture(root)

            def build_distributions(*args: object, **kwargs: object) -> None:
                self.fixture(root)
                (root / "proto/dokime/host/v1/service.proto").write_bytes(b"changed")

            with (
                mock.patch.object(wheel.subprocess, "run", side_effect=build_distributions),
                mock.patch.object(wheel.policy, "config", return_value={"version": {"value": "0.1.3"}}),
                self.assertRaises(ToolError),
            ):
                wheel.build(root, "dev")
        finally:
            temporary_directory.cleanup()

    def test_package_copies_shared_pair_without_rebuilding(self) -> None:
        temporary_directory = TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary).resolve()
            output = self.fixture(root)
            with (
                mock.patch.object(dokime_tools.package.run.Path, "cwd", return_value=root),
                mock.patch.object(dokime_tools.package.run.policy, "config", return_value={"version": {"value": "0.1.3"}}),
                mock.patch("sys.argv", ["package", "python", "--target", "windows-x86_64"]),
                mock.patch.object(dokime_tools.package.run.subprocess, "run") as execute,
            ):
                dokime_tools.package.run.main()
            execute.assert_not_called()
            for source in wheel.distributions(root, output, "0.1.3"):
                self.assertEqual((root / "target/package/dev/any/python" / source.name).read_bytes(), source.read_bytes())
            self.assertFalse((root / "target/package/dev/windows-x86_64").exists())
        finally:
            temporary_directory.cleanup()

    def test_rejects_native_missing_and_stale_protocol_wheels(self) -> None:
        for invalid in ("native", "missing", "stale", "identity", "tag", "source"):
            temporary_directory = TemporaryDirectory()
            temporary = temporary_directory.name
            try:
                with self.subTest(invalid=invalid):
                    root = Path(temporary)
                    output = self.fixture(root)
                    source = output / "dokime_sdk-0.1.3-py3-none-any.whl"
                    archive = ZipFile(source)
                    try:
                        contents = {name: archive.read(name) for name in archive.namelist()}
                    finally:
                        archive.close()
                    if invalid == "native":
                        contents["dokime_sdk/native.pyd"] = b"native"
                    elif invalid == "missing":
                        del contents["dokime_sdk/_wire/host/v1/service_pb2_grpc.py"]
                    elif invalid == "stale":
                        (root / "proto/dokime/host/v1/service.proto").write_bytes(b"changed")
                    elif invalid == "identity":
                        contents["dokime_sdk-0.1.3.dist-info/METADATA"] = b"Name: other\nVersion: 0.1.3\n"
                    elif invalid == "tag":
                        contents["dokime_sdk-0.1.3.dist-info/WHEEL"] = b"Root-Is-Purelib: false\nTag: cp313-abi3-win_amd64\n"
                    else:
                        tarfile.open(output / "dokime_sdk-0.1.3.tar.gz", "w:gz").close()
                    archive = ZipFile(source, "w")
                    try:
                        for name, content in contents.items():
                            archive.writestr(name, content)
                    finally:
                        archive.close()
                    with self.assertRaises((ToolError, KeyError)):
                        wheel.verify(root, output, "0.1.3")
            finally:
                temporary_directory.cleanup()

    def test_rejects_wrong_version_and_extra_distribution(self) -> None:
        temporary_directory = TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            output = self.fixture(root)
            with self.assertRaises(ToolError):
                wheel.distributions(root, output, "0.2.0")
            (output / "dokime-0.1.3-cp313-abi3-win_amd64.whl").write_bytes(b"old")
            with self.assertRaises(ToolError):
                wheel.distributions(root, output, "0.1.3")
        finally:
            temporary_directory.cleanup()

    def test_registry_uploads_pair_once_for_all_targets(self) -> None:
        temporary_directory = TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            source = self.fixture(root)
            destination = root / "target/package/dev/any/python"
            destination.parent.mkdir(parents=True)
            source.rename(destination)
            (root / "sdk/python").mkdir(parents=True)
            (root / "sdk/python/pyproject.toml").write_text('[project]\nname = "dokime-sdk"\n', encoding="utf-8")
            settings = {**SETTINGS, "targets": ["linux-x86_64", "windows-x86_64"]}
            for dry_run in (True, False):
                with (
                    self.subTest(dry_run=dry_run),
                    mock.patch.object(policy, "profile", return_value="dev"),
                    mock.patch.object(policy, "config", return_value={"version": {"value": "0.1.3"}}),
                    mock.patch.object(registry.python, "digests", return_value={}),
                    mock.patch.object(policy, "execute") as execute,
                ):
                    registry.python.publish(root, settings, dry_run=dry_run)
                execute.assert_called_once()
                command = execute.call_args.args[1]
                self.assertEqual(command[:2], ["uv", "publish"])
                self.assertEqual(
                    command[-2:],
                    [f"target/package/dev/any/python/{name}" for name in ("dokime_sdk-0.1.3-py3-none-any.whl", "dokime_sdk-0.1.3.tar.gz")],
                )
                self.assertEqual("--dry-run" in command, dry_run)
        finally:
            temporary_directory.cleanup()


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
