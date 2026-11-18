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
                    path = root / "sdk/python/src" / name
                    path.parent.mkdir(parents=True)
                    path.write_text(source, encoding="utf-8")
                    self.assertRaisesRegex(ValueError, "Retired Python", check_public_surfaces, root)
                finally:
                    scratch.cleanup()

    def test_duplicate_exports_distinguish_namespaces_from_values(self) -> None:
        value = "class Value: pass\n__all__ = ['Value']\n"
        namespace = "__all__ = ['Value']\n"
        cases = (
            ("classes", {"__init__.py": value, "client.py": value}, True),
            ("functions", {"__init__.py": "def Value(): pass\n" + namespace, "client.py": value}, True),
            ("constants", {"__init__.py": "Value = 1\n" + namespace, "client.py": value}, True),
            ("module_files", {"client.py": namespace, "Value.py": "", "plugin/__init__.py": namespace, "plugin/Value.py": ""}, False),
            (
                "subpackages",
                {"client.py": namespace, "Value/__init__.py": "", "plugin/__init__.py": namespace, "plugin/Value/__init__.py": ""},
                False,
            ),
            (
                "relative_imports",
                {
                    "__init__.py": "from . import Value\n" + namespace,
                    "Value.py": "",
                    "plugin/__init__.py": "from ..nested import Value\n" + namespace,
                    "nested/Value.py": "",
                },
                False,
            ),
            (
                "reexported_modules",
                {
                    "__init__.py": "from .shared import Value\n" + namespace,
                    "shared.py": "from .nested import Value\n",
                    "nested/Value.py": "",
                    "client.py": "from .shared import Value\n" + namespace,
                },
                False,
            ),
            ("class_then_module", {"client.py": value, "plugin/__init__.py": namespace, "plugin/Value.py": ""}, True),
            ("module_then_class", {"client.py": namespace, "Value.py": "", "plugin/__init__.py": value}, True),
            (
                "class_between_modules",
                {
                    "__init__.py": "from .nested import Value\n" + namespace,
                    "nested/Value.py": "",
                    "client.py": value,
                    "plugin/__init__.py": namespace,
                    "plugin/Value.py": "",
                },
                True,
            ),
            (
                "imported_classes",
                {
                    "__init__.py": "from .shared import Value\n" + namespace,
                    "client.py": "from .shared import Value\n" + namespace,
                    "shared.py": value,
                },
                True,
            ),
            (
                "cyclic_imports",
                {"__init__.py": "from .client import Value\n" + namespace, "client.py": "from . import Value\n" + namespace},
                True,
            ),
        )
        for name, files, duplicate in cases:
            with self.subTest(name=name):
                scratch_directory = tempfile.TemporaryDirectory()
                try:
                    root = Path(scratch_directory.name)
                    for relative, source in files.items():
                        path = root / "sdk/python/src/dokime_sdk" / relative
                        path.parent.mkdir(parents=True, exist_ok=True)
                        path.write_text(source, encoding="utf-8")
                    if duplicate:
                        self.assertRaisesRegex(ValueError, "Duplicate Public Export Value", check_public_surfaces, root)
                    else:
                        check_public_surfaces(root)
                finally:
                    scratch_directory.cleanup()

    def test_private_wire_paths_are_refused_only_at_public_entry_points(self) -> None:
        scratch_directory = tempfile.TemporaryDirectory()
        scratch = scratch_directory.name
        try:
            root = Path(scratch)
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
            exports = root / "sdk/python/src/dokime_sdk"
            exports.mkdir(parents=True)
            (exports / "__init__.py").write_text("__all__ = ['Value']", encoding="utf-8")
            (exports / "client.py").write_text("__all__ = ['Value']", encoding="utf-8")
            self.assertRaises(ValueError, check_public_surfaces, root)
            (exports / "client.py").write_text("__all__ = ['Client']", encoding="utf-8")
            check_public_surfaces(root)
        finally:
            scratch_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
