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
import io
import os
import tarfile
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.check import package as check_package
from dokime_tools.environment import native as native_environment
from dokime_tools.package import install_wheel as install_dist_wheel
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
                "packages/dokime/src/dokime/__init__.py": b"# Client",
                "packages/dokime/src/dokime/client/api.pyi": b"# Stubs",
                "packages/dokime/src/dokime/py.typed": b"",
                "packages/dokime/pyproject.toml": b"# Build",
                "packages/dokime/README.md": b"# Dokime",
                "packages/dokime/LICENSE-APACHE": b"Apache license",
            }
            for name, content in inputs.items():
                source = root / name
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_bytes(content)
            archive = root / "dokime-0.1.0.tar.gz"

            for missing in (None, *inputs):
                with self.subTest(missing=missing):
                    with tarfile.open(archive, "w:gz") as output:
                        metadata = b"Name: dokime\nVersion: 0.1.0\n"
                        entry = tarfile.TarInfo("dokime-0.1.0/PKG-INFO")
                        entry.size = len(metadata)
                        output.addfile(entry, io.BytesIO(metadata))
                        for name, content in inputs.items():
                            if name == missing:
                                continue
                            relative = name.removeprefix("packages/dokime/")
                            member = tarfile.TarInfo(f"dokime-0.1.0/{relative}")
                            member.size = len(content)
                            output.addfile(member, io.BytesIO(content))
                    if missing is None:
                        check_package._verify_sdist_build_inputs(archive, root)
                        source = root / "packages/dokime/src/dokime/__init__.py"
                        source.write_bytes(b"changed")
                        with self.assertRaisesRegex(SystemExit, "Stale.*__init__.py"):
                            check_package._verify_sdist_build_inputs(archive, root)
                        source.write_bytes(
                            inputs["packages/dokime/src/dokime/__init__.py"]
                        )
                    else:
                        with self.assertRaisesRegex(
                            SystemExit,
                            "Missing.*" + missing.removeprefix("packages/dokime/"),
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
