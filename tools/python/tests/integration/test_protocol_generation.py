# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prove Caller Projection and Public Surface Rejection at the Generator Boundary."""

# === Imports === #

# Standard Library Imports
import tempfile
import unittest
from pathlib import Path

# isort: split

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# isort: split

# Local Imports
from dokime_tools.build.protocol_callers import caller_options, check_public_surfaces, narrow_callers
from dokime_tools.build.resources.schema import table

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class GenerationContracts(unittest.TestCase):
    def test_composition_narrows_both_surfaces_and_refuses_widening(self) -> None:
        root = Path(__file__).resolve().parents[4]
        original = FileDescriptorSet.FromString((root / "crates/dokime-protocol/src/descriptors.bin").read_bytes())
        identity = next(identity for identity, audience in caller_options(original).items() if audience == "ALL")
        for audience, face in [("CLIENTS", "client"), ("PLUGINS", "context")]:
            with self.subTest(audience=audience):
                narrowed = FileDescriptorSet.FromString(original.SerializeToString())
                narrow_callers(narrowed, {identity: audience})
                selected = []
                for selected_method in table(narrowed):
                    if f"{selected_method['package']}.{selected_method['service']}/{selected_method['method']}" != identity:
                        continue
                    selected.append(selected_method)

                self.assertEqual({method["face"] for method in selected}, {face})
                with self.assertRaises(ValueError):
                    narrow_callers(narrowed, {identity: "ALL"})
        self.assertEqual(caller_options(original)[identity], "ALL")

    def test_private_wire_paths_are_refused_only_at_public_entry_points(self) -> None:
        scratch_directory = tempfile.TemporaryDirectory()
        scratch = scratch_directory.name
        try:
            root = Path(scratch)
            readme = root / "README.md"
            readme.write_text("/application.host.v1.Resource/Call", encoding="utf-8")
            internal = root / "sdk/cpp/include/detail/messages.hpp"
            internal.parent.mkdir(parents=True)
            internal.write_text("using Value = application::model::v1::Value;", encoding="utf-8")
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
