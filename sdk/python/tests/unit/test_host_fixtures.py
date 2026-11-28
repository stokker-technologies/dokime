# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prepared SDK Fixture Selection"""

# === Imports === #

# Standard Library Imports
import os
import runpy
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest import TestCase
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.build import configuration, host_fixtures

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class HostFixtureStory(TestCase):
    def test_reference_host_builds_from_the_locked_library_source(self) -> None:
        root = Path(__file__).resolve().parents[4]
        source = root / "target/library/0.0.0"
        self.enterContext(patch.object(configuration, "ROOT", root))
        self.enterContext(patch.object(host_fixtures.protocol, "library", return_value=source))
        run = self.enterContext(patch.object(host_fixtures.subprocess, "run"))
        binary = host_fixtures.prepare_reference("dev")
        command = run.call_args.args[0]
        self.assertEqual(command[:5], ["cargo", "install", "--path", str(source / "tools/conformance"), "--locked"])
        self.assertIn("--debug", command)
        self.assertEqual(command[command.index("--target-dir") + 1], str(root / "target"))
        self.assertEqual(command[command.index("--root") + 1], str(root / "target/reference-host"))
        self.assertEqual(binary.parent, root / "target/reference-host/bin")

    def test_library_source_requires_one_locked_release(self) -> None:
        temporary_directory = TemporaryDirectory()
        try:
            root = Path(temporary_directory.name)
            versions = "".join(f'[[package]]\nname = "harmos"\nversion = "{version}"\n' for version in ("1.0.0", "1.1.0"))
            (root / "Cargo.lock").write_text(versions, encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "Lock Exactly One harmos Version"):
                host_fixtures.protocol.release(root)
        finally:
            temporary_directory.cleanup()

    def test_exported_fixture_profiles_nested_assets_and_explicit_overrides(self) -> None:
        root = Path(__file__).resolve().parents[4]
        host = runpy.run_path(str(root / "sdk/python/tests/support/host.py"))["Host"]
        self.enterContext(patch("platform.system", return_value="Windows" if os.name == "nt" else "Linux"))
        self.enterContext(patch("platform.machine", return_value="x86_64"))
        extension = ".exe" if os.name == "nt" else ""
        original = Path.is_file
        for profile in (None, "dev", "release"):
            for name, home in (
                ("dokime-host", "fixtures/host"),
                ("dokime", "bin"),
                ("rust", "fixtures/cli"),
                ("csharp/source/assets/plugin/Csharp.Plugin.dll", "fixtures/cli"),
            ):
                native = "windows-x86_64" if os.name == "nt" else "linux-x86_64"
                filename = name if Path(name).suffix else f"{name}{extension}"
                expected = root / "target/build" / (profile or "dev") / native / home / filename

                def present(path: Path, expected: Path = expected) -> bool:
                    return path == expected or original(path)

                self.enterContext(patch.dict(os.environ))
                if profile is None:
                    os.environ.pop("TEST_PROFILE", None)
                else:
                    os.environ["TEST_PROFILE"] = profile
                self.enterContext(patch.object(Path, "is_file", present))
                os.environ.pop("DOKIME_TEST_HOST", None)
                os.environ.pop("DOKIME_TEST_CLI", None)
                self.assertEqual(host.fixture(name, home=home), expected)

        temporary_directory = TemporaryDirectory()
        try:
            binary = Path(temporary_directory.name) / f"prepared{extension}"
            binary.write_bytes(b"prepared")
            for name, variable in (("dokime-host", "DOKIME_TEST_HOST"), ("dokime", "DOKIME_TEST_CLI")):
                self.enterContext(patch.dict(os.environ, {"TEST_PROFILE": "release", variable: str(binary)}))
                self.assertEqual(host.fixture(name), binary)
                binary.unlink()
                self.assertRaises(RuntimeError, host.fixture, name)
                binary.write_bytes(b"prepared")
        finally:
            temporary_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
