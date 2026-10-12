# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Wheel consumers reject missing and ambiguous build outputs."""

# === Imports === #

# Standard Library Imports
import os
import io
import tarfile
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

# Third Party Imports

# Local Imports
from dokime_tools.check import package as check_package
from dokime_tools.package import install_wheel as install_dist_wheel
from dokime_tools.environment import native as native_environment
from dokime_tools.package.wheel import WheelSelectionError, select_wheel

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class WheelTests(unittest.TestCase):
    def case_sdist_inputs(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = {
                "branding/brand.toml": b'name = "Dokime"',
                "branding/theme/base.css": b"body {}",
                "branding/fonts/font.otf": b"font",
                "branding/logos/logo.svg": b"<svg />",
                "crates/dokime-protocol/proto/dokime/v1/common.proto": b'syntax = "proto3";',
                "crates/dokime-protocol/proto/dokime/v1/execution.proto": b"message Execution {}",
                "crates/dokime-app/dist-report/report.html": b"<html>Report</html>",
                "crates/dokime-python/python/dokime/core/__init__.py": b"# Core API",
                "crates/dokime-python/python/dokime/core/suites.pyi": b"# Stubs",
                "crates/dokime-python/python/dokime/py.typed": b"",
            }
            for name, content in inputs.items():
                source = root / name
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_bytes(content)
            archive = root / "dokime-0.1.0.tar.gz"

            for missing in (None, *inputs):
                with self.subTest(missing=missing):
                    with tarfile.open(archive, "w:gz") as output:
                        for name, content in inputs.items():
                            if name == missing:
                                continue
                            relative = name.removeprefix("crates/dokime-python/")
                            member = tarfile.TarInfo(f"dokime-0.1.0/{relative}")
                            member.size = len(content)
                            output.addfile(member, io.BytesIO(content))
                    if missing is None:
                        check_package._verify_sdist_build_inputs(archive, root)
                        source = (
                            root / "crates/dokime-protocol/proto/dokime/v1/common.proto"
                        )
                        source.write_bytes(b"changed")
                        with self.assertRaisesRegex(SystemExit, "Stale.*common.proto"):
                            check_package._verify_sdist_build_inputs(archive, root)
                        source.write_bytes(
                            inputs[
                                "crates/dokime-protocol/proto/dokime/v1/common.proto"
                            ]
                        )
                    else:
                        with self.assertRaisesRegex(
                            SystemExit,
                            "Missing.*" + missing.removeprefix("crates/dokime-python/"),
                        ):
                            check_package._verify_sdist_build_inputs(archive, root)

    def case_selection(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            with self.assertRaises(WheelSelectionError):
                select_wheel(root, "dokime-*.whl")
            (root / "unrelated.whl").touch()
            wheel = root / "dokime-1.whl"
            wheel.touch()
            self.assertEqual(select_wheel(root, "dokime-*.whl"), wheel)
            (root / "dokime-0.whl").touch()
            with self.assertRaises(WheelSelectionError) as caught:
                select_wheel(root, "dokime-*.whl")
            self.assertEqual(
                [p.name for p in caught.exception.wheels],
                ["dokime-0.whl", "dokime-1.whl"],
            )

    def case_consumer_diagnostics(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            for resolver, missing, multiple in (
                (
                    lambda: native_environment._unique_wheel(root),
                    f"No dokime wheel artifact exists in {root}.",
                    f"Expected one dokime wheel in {root}, found: dokime-0.whl, dokime-1.whl.",
                ),
                (
                    lambda: install_dist_wheel._resolve_wheel(root, "dokime-*.whl"),
                    f"No wheel matching 'dokime-*.whl' exists in {root}.",
                    "Multiple wheels match 'dokime-*.whl': dokime-0.whl, dokime-1.whl.",
                ),
            ):
                with self.assertRaises(SystemExit) as caught:
                    resolver()
                self.assertEqual(str(caught.exception), missing)
                for name in ("dokime-1.whl", "dokime-0.whl"):
                    (root / name).touch()
                with self.assertRaises(SystemExit) as caught:
                    resolver()
                self.assertEqual(str(caught.exception), multiple)
                for wheel in root.glob("*.whl"):
                    wheel.unlink()

    def case_install_command(self):
        with TemporaryDirectory() as temporary:
            wheel = Path(temporary) / "custom-1.whl"
            wheel.touch()
            with patch.object(install_dist_wheel.subprocess, "run") as run:
                self.assertEqual(
                    install_dist_wheel.main([temporary, "custom-*.whl", "--no-deps"]), 0
                )
            run.assert_called_once_with(
                [
                    "uv",
                    "pip",
                    "install",
                    "--python",
                    str(
                        Path(__file__).resolve().parents[4]
                        / ".venv"
                        / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
                    ),
                    "--reinstall",
                    "--no-deps",
                    str(wheel),
                ],
                check=True,
            )

    def test_contract(self):
        for scenario in (
            "case_sdist_inputs",
            "case_selection",
            "case_consumer_diagnostics",
            "case_install_command",
        ):
            with self.subTest(scenario=scenario):
                type(self)(scenario).debug()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
