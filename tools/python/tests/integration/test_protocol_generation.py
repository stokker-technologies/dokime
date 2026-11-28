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
import hashlib
import runpy
import shutil
import tempfile
import unittest
from io import BytesIO
from pathlib import Path
from unittest import mock

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import protocol
from dokime_tools.build.protocol import Placement, cpp_dependency, place
from dokime_tools.build.protocol_callers import check_public_surfaces

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class GenerationContracts(unittest.TestCase):
    def test_python_source_inventory_tracks_content_additions_removals_and_renames(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, scratch)
        generator = Path(__file__).resolve().parents[4] / "sdk/python/tools/generate.py"
        source_inventory = runpy.run_path(str(generator))["source_inventory"]
        self.assertEqual(source_inventory(scratch), {})
        product = scratch / "dokime/model/v1/value.proto"
        product.parent.mkdir(parents=True)
        product.write_bytes(b"product")
        library = scratch / "harmos/model/v1/value.proto"
        library.parent.mkdir(parents=True)
        library.write_bytes(b"library")
        (scratch / ".revision").write_text("not a protocol")
        inventory = source_inventory(scratch)
        self.assertEqual(
            inventory,
            {
                "dokime/model/v1/value.proto": hashlib.sha256(b"product").hexdigest(),
                "harmos/model/v1/value.proto": hashlib.sha256(b"library").hexdigest(),
            },
        )
        product.write_bytes(b"changed")
        self.assertNotEqual(source_inventory(scratch)["dokime/model/v1/value.proto"], inventory["dokime/model/v1/value.proto"])
        product.rename(product.with_name("renamed.proto"))
        library.unlink()
        self.assertEqual(source_inventory(scratch), {"dokime/model/v1/renamed.proto": hashlib.sha256(b"changed").hexdigest()})

    def test_protocol_inputs_follow_the_compiled_local_library_and_prune_stale_sources(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, scratch)
        product = scratch / "proto/dokime/model/v1/value.proto"
        library = scratch / "library/proto/harmos/model/v1/value.proto"
        for path in (product, library):
            path.parent.mkdir(parents=True)
            path.write_text("first")
        # A Workspace Patch Points the Library at a Local Checkout Whose Repository Owns proto/harmos.
        (scratch / "Cargo.toml").write_text(
            '[workspace.dependencies]\nharmos = "1.0.0"\n[patch.crates-io]\nharmos = { path = "library/sdk/rust" }\n'
        )
        output = protocol.inputs(scratch)
        self.assertEqual((output / "harmos/model/v1/value.proto").read_text(), "first")
        library.write_text("second")
        product.unlink()
        protocol.inputs(scratch)
        self.assertEqual((output / "harmos/model/v1/value.proto").read_text(), "second")
        self.assertFalse((output / "dokime/model/v1/value.proto").exists())

    def test_placement_replaces_only_owned_files_and_reports_each_difference(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, scratch)
        rendered = scratch / "rendered"
        destination = scratch / "sdk/generated"
        for path, text in ((rendered / "client.py", "new"), (rendered / "proto/added_pb2.py", "added")):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        for path, text in (
            (destination / "client.py", "old"),
            (destination / "proto/obsolete_pb2.py", "old"),
            (destination / "authored.py", "kept"),
        ):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        owned = Placement("sdk/generated", ("client.py", "proto/**/*_pb2.py"))
        self.assertEqual(
            place(scratch, rendered, owned, check=True),
            [
                "Missing: sdk/generated/proto/added_pb2.py",
                "Obsolete: sdk/generated/proto/obsolete_pb2.py",
                "Changed: sdk/generated/client.py",
            ],
        )
        self.assertEqual(place(scratch, rendered, owned, check=False), [])
        self.assertEqual(place(scratch, rendered, owned, check=True), [])
        self.assertEqual((destination / "client.py").read_text(), "new")
        self.assertFalse((destination / "proto/obsolete_pb2.py").exists())
        self.assertEqual((destination / "authored.py").read_text(), "kept")

    def test_cpp_archive_follows_the_locked_library_release_and_refuses_stale_bytes(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        try:
            lock = scratch / "Cargo.lock"
            dependency = scratch / "sdk/cpp/cmake/Dependencies.cmake"
            dependency.parent.mkdir(parents=True)
            archive = b"First Archive"
            lock.write_text('[[package]]\nname = "harmos"\nversion = "1.0.0"\n')
            with mock.patch("urllib.request.urlopen", side_effect=lambda url: BytesIO(archive)) as download:
                cpp_dependency(scratch)
                cpp_dependency(scratch, check=True)
                first = dependency.read_bytes()
                self.assertEqual(download.call_args.args[0], f"{protocol.LIBRARY}/archive/refs/tags/v1.0.0.tar.gz")
                lock.write_text(lock.read_text().replace('"1.0.0"', '"1.1.0"'))
                self.assertRaises(ValueError, cpp_dependency, scratch, check=True)
                cpp_dependency(scratch)
                self.assertNotEqual(first, dependency.read_bytes())
                self.assertEqual(download.call_args.args[0], f"{protocol.LIBRARY}/archive/refs/tags/v1.1.0.tar.gz")
                archive = b"Changed Archive"
                self.assertRaises(ValueError, cpp_dependency, scratch, check=True)
                cpp_dependency(scratch)
                cpp_dependency(scratch, check=True)
        finally:
            shutil.rmtree(scratch)

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
                    self.assertRaises(ValueError, check_public_surfaces, root)
                finally:
                    scratch.cleanup()

    def test_three_public_homes_have_one_inventory(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        try:
            generator = scratch / "sdk/python/tools/generate.py"
            generator.parent.mkdir(parents=True)
            shutil.copy2(Path(__file__).resolve().parents[4] / "sdk/python/tools/generate.py", generator)
            homes = runpy.run_path(str(generator))["PUBLIC_HOMES"]
            package = scratch / "sdk/python/src/dokime_sdk"
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
            self.assertRaises(ValueError, check_public_surfaces, scratch)
            fourth.unlink()
            templates = package / "templates/__init__.py"
            templates.write_text(templates.read_text().replace("Context = object()", ""), encoding="utf-8")
            self.assertRaises(ValueError, check_public_surfaces, scratch)
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
