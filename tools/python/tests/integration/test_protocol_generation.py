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
            with self.assertRaises(ValueError):
                check_public_surfaces(root)
            public.write_text('#include "detail/messages.hpp"', encoding="utf-8")
            exports = root / "sdk/python/src/dokime"
            exports.mkdir(parents=True)
            (exports / "__init__.py").write_text("__all__ = ['Value']", encoding="utf-8")
            (exports / "client.py").write_text("__all__ = ['Value']", encoding="utf-8")
            with self.assertRaises(ValueError):
                check_public_surfaces(root)
            (exports / "client.py").write_text("__all__ = ['Client']", encoding="utf-8")
            check_public_surfaces(root)
        finally:
            scratch_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
