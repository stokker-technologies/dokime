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

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class HostFixtureStory(TestCase):
    def test_sdk_fixture_lookup_uses_prepared_profiles_and_explicit_overrides(self) -> None:
        root = Path(__file__).resolve().parents[4]
        host = runpy.run_path(str(root / "sdk/python/tests/support/host.py"))["Host"]
        self.enterContext(patch("platform.system", return_value="Windows" if os.name == "nt" else "Linux"))
        self.enterContext(patch("platform.machine", return_value="x86_64"))
        extension = ".exe" if os.name == "nt" else ""
        original = Path.is_file
        for profile in (None, "dev", "release"):
            for name, home in (("dokime-host", "fixtures/host"), ("dokime", "bin")):
                native = "windows-x86_64" if os.name == "nt" else "linux-x86_64"
                expected = (
                    root / "target/build" / profile / native / home / f"{name}{extension}"
                    if profile
                    else root / "target/debug" / f"{name}{extension}"
                )

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
                self.assertEqual(host.fixture(name), expected)

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
