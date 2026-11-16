# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Non-Mutating SDK Drift Checks"""

# === Imports === #

# Standard Library Imports
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

# Third Party Imports
# Local Imports
from dokime_tools.build import resource_roots

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ResourceRootsTests(unittest.TestCase):
    def test_check_compares_complete_inventory_without_mutation(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, scratch)
        expected = scratch / "expected"
        outputs = {
            "sdk/rust/src/client/resources.rs": b"// @generated from protocol descriptors; do not edit.\n",
            "sdk/python/src/dokime/_generated/client.py": b"class Client: pass\n",
            "sdk/csharp/src/Dokime/Generated/Wire/Request.cs": b"class Request {}\n",
            "sdk/cpp/include/dokime/_generated/client.hpp": b"#pragma once\n",
        }
        for name, content in outputs.items():
            path = expected / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        cases = (
            ("clean", None, None),
            ("changed", "sdk/rust/src/client/resources.rs", b"stale"),
            ("missing", "sdk/python/src/dokime/_generated/client.py", None),
            ("obsolete", "sdk/csharp/src/Dokime/Generated/Wire/Retired.cs", b"retired"),
            ("obsolete", "sdk/rust/src/client/retired.rs", next(iter(outputs.values()))),
            ("obsolete", "sdk/cpp/include/dokime/_generated/detail/retired.hpp", b"retired"),
        )
        for index, (difference, name, content) in enumerate(cases):
            with self.subTest(difference=difference, name=name):
                root = scratch / str(index)
                shutil.copytree(expected, root)
                if name is not None:
                    path = root / name
                    if content is None:
                        path.unlink()
                    else:
                        path.parent.mkdir(parents=True, exist_ok=True)
                        path.write_bytes(content)
                before = {path.relative_to(root): path.read_bytes() for path in root.rglob("*") if path.is_file()}
                with mock.patch.object(
                    resource_roots, "_render", side_effect=lambda source, output: shutil.copytree(expected, output, dirs_exist_ok=True)
                ):
                    if difference == "clean":
                        resource_roots.generate(check=True, root=root)
                    else:
                        with self.assertRaisesRegex(ValueError, difference.title() + ": " + str(name)):
                            resource_roots.generate(check=True, root=root)
                after = {path.relative_to(root): path.read_bytes() for path in root.rglob("*") if path.is_file()}
                self.assertEqual(after, before)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
