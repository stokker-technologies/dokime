# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prove Product Public Surface Rejection at the Generator Boundary."""

# === Imports === #

# Standard Library Imports
import runpy
import shutil
import tempfile
import unittest
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build.protocol_callers import check_public_surfaces

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class GenerationContracts(unittest.TestCase):
    def test_retired_python_package_and_imports_are_refused(self) -> None:
        cases = (
            ("dokime/__init__.py", ""),
            ("dokime_sdk/__init__.py", "import dokime\n"),
            ("dokime_sdk/_internal.py", "from dokime.client import Client\n"),
            ("dokime_sdk/_wire/generated.pyi", "from dokime import Value\n"),
        )
        for name, source in cases:
            with self.subTest(name=name):
                scratch = tempfile.TemporaryDirectory()
                try:
                    root = Path(scratch.name)
                    generator = root / "sdk/python/tools/generate.py"
                    generator.parent.mkdir(parents=True)
                    shutil.copy2(Path(__file__).resolve().parents[4] / "sdk/python/tools/generate.py", generator)
                    path = root / "sdk/python/src" / name
                    path.parent.mkdir(parents=True)
                    path.write_text(source, encoding="utf-8")
                    self.assertRaisesRegex(ValueError, "Retired Python", check_public_surfaces, root)
                finally:
                    scratch.cleanup()

    def test_three_public_homes_have_one_inventory(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        try:
            package = scratch / "sdk/python/src/dokime_sdk"
            generator = scratch / "sdk/python/tools/generate.py"
            generator.parent.mkdir(parents=True)
            shutil.copy2(Path(__file__).resolve().parents[4] / "sdk/python/tools/generate.py", generator)
            homes = runpy.run_path(str(generator))["PUBLIC_HOMES"]
            for module, names in homes.items():
                relative = f"{module}/__init__.py" if module == "templates" else f"{module}.py" if module else "__init__.py"
                path = package / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("\n".join(f"{name} = object()" for name in sorted(names)), encoding="utf-8")
            check_public_surfaces(scratch)
            root = package / "__init__.py"
            original = root.read_text()
            for source in (
                "Context = object()",
                "__all__ = []",
                "from ._client import Client as OtherClient",
                "def __getattr__(name): return object()",
                "from ._generated import Other",
            ):
                with self.subTest(source=source):
                    root.write_text(original + "\n" + source, encoding="utf-8")
                    self.assertRaises(ValueError, check_public_surfaces, scratch)
            root.write_text(original, encoding="utf-8")
            fourth = package / "client.py"
            fourth.write_text("from ._generated.client import ClientRoot", encoding="utf-8")
            self.assertRaisesRegex(ValueError, "Fourth Python Public Home", check_public_surfaces, scratch)
            fourth.unlink()
            templates = package / "templates/__init__.py"
            templates.write_text(templates.read_text().replace("Context = object()", ""), encoding="utf-8")
            self.assertRaisesRegex(ValueError, "Missing.*Context", check_public_surfaces, scratch)
        finally:
            shutil.rmtree(scratch)

    def test_private_wire_paths_are_refused_only_at_public_entry_points(self) -> None:
        scratch_directory = tempfile.TemporaryDirectory()
        scratch = scratch_directory.name
        try:
            root = Path(scratch)
            generator = root / "sdk/python/tools/generate.py"
            generator.parent.mkdir(parents=True)
            shutil.copy2(Path(__file__).resolve().parents[4] / "sdk/python/tools/generate.py", generator)
            readme = root / "README.md"
            readme.write_text("/application.host.v1.Resource/Call", encoding="utf-8")
            internal = root / "sdk/cpp/include/detail/messages.hpp"
            internal.parent.mkdir(parents=True)
            internal.write_text("application::model::v1::Value value;", encoding="utf-8")
            check_public_surfaces(root)
            public = root / "sdk/cpp/include/messages.hpp"
            internal.replace(public)
            self.assertRaises(ValueError, check_public_surfaces, root)
            public.write_text('#include "detail/messages.hpp"', encoding="utf-8")
        finally:
            scratch_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
