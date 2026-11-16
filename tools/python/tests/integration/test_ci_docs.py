# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Exercise Guide Surface Rules Without Compilation or Authored Prose Assertions."""

# === Imports === #

# Standard Library Imports
import tempfile
import unittest
from pathlib import Path

# isort: split

# Third Party Imports
# isort: split

# Local Imports
from dokime_tools.ci import docs

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class GuideTests(unittest.TestCase):
    def test_guides_allow_public_calls_and_source_links_but_reject_wire_imports(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary.name)
            (root / "README.md").write_text("")
            (root / "docs").mkdir(parents=True)
            path = root / "docs/guide.md"
            for source, invalid in (
                ("client.operations.list()", False),
                ("[Source](../../proto/product/v1/client.proto)", False),
                ("/product.client.v1.Service/List", False),
                ("from product.client.v1 import Request", True),
                ("client._wire.Request()", True),
            ):
                path.write_text(source, encoding="utf-8")
                if invalid:
                    with self.assertRaises(ValueError):
                        docs.check(root)
                else:
                    docs.check(root)
            (root / "README.md").unlink()
            with self.assertRaises(FileNotFoundError):
                docs.check(root)
        finally:
            temporary.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
