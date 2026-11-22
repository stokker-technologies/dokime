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
import tarfile
import tempfile
import unittest
from io import BytesIO
from pathlib import Path
from tarfile import TarInfo
from unittest import mock

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import protocol
from dokime_tools.build.protocol import cpp_dependency, library_dependencies, library_protocol
from dokime_tools.build.protocol_callers import check_public_surfaces

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class GenerationContracts(unittest.TestCase):
    def test_library_protocol_replaces_the_inventory_and_rejects_drift(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        archive = BytesIO()
        files = tarfile.open(fileobj=archive, mode="w:gz")
        try:
            for name, data in {
                "library/proto/harmos/plugin/v1/plugins/state.proto": b"state contract",
                "library/proto/other/ignored.proto": b"other contract",
            }.items():
                entry = TarInfo(name)
                entry.size = len(data)
                files.addfile(entry, BytesIO(data))
        finally:
            files.close()
        try:
            stale = scratch / "proto/harmos/removed.proto"
            stale.parent.mkdir(parents=True)
            stale.write_bytes(b"removed contract")
            library_protocol(scratch, archive.getvalue())
            self.assertFalse(stale.exists())
            self.assertFalse((scratch / "proto/other").exists())
            library_protocol(scratch, archive.getvalue(), check=True)
            state = scratch / "proto/harmos/plugin/v1/plugins/state.proto"
            self.assertEqual(state.read_bytes(), b"state contract")
            state.write_bytes(b"changed contract")
            self.assertRaises(ValueError, library_protocol, scratch, archive.getvalue(), check=True)
            library_protocol(scratch, archive.getvalue())
            library_protocol(scratch, archive.getvalue(), check=True)
            self.assertRaises(tarfile.ReadError, library_protocol, scratch, b"not an archive")
        finally:
            shutil.rmtree(scratch)

    def test_pin_projection_and_lock_refresh_precede_locked_generation(self) -> None:
        root = Path.cwd()
        with (
            mock.patch.object(protocol.sys, "argv", ["protocol", "--language", "python"]),
            mock.patch.object(protocol, "library_dependencies", return_value=True) as projection,
            mock.patch.object(protocol, "locks") as locks,
            mock.patch.object(protocol.subprocess, "run") as run,
        ):
            order = mock.Mock()
            order.attach_mock(projection, "project")
            order.attach_mock(locks, "locks")
            order.attach_mock(run, "reload")
            protocol.main()
            self.assertEqual([call[0] for call in order.mock_calls], ["project", "locks", "reload"])
            projection.assert_called_once_with(root, check=False)
            locks.assert_called_once_with(root)
            self.assertEqual(run.call_args.args[0][-2:], ["--language", "python"])
        with mock.patch.object(protocol.subprocess, "run") as run:
            protocol.locks(root)
        commands = [call.args[0] for call in run.call_args_list]
        self.assertEqual(commands[0], ["cargo", "update", "--package", "harmos"])
        self.assertEqual(commands[-1], ["uv", "sync", "--locked", "--all-groups"])
        self.assertLess(commands.index(["uv", "lock"]), commands.index(commands[-1]))

    def test_manifest_pins_follow_workspace_and_reject_each_stale_consumer(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        try:
            source = "https://example.test/library"
            manifests = {
                "Cargo.toml": (
                    f'[workspace.dependencies]\nharmos = {{ git = "{source}", rev = "next" }}\n'
                    f'library-runtime = {{ git = "{source}", rev = "previous" }}\n'
                ),
                "sdk/rust/tests/consumer/Cargo.toml": f'[dev-dependencies]\nlibrary-runtime = {{ git = "{source}", rev = "previous" }}\n',
                "tools/python/pyproject.toml": f'[tool.uv.sources]\nlibrary-tools = {{ git = "{source}", rev = "previous" }}\n',
                "sdk/python/pyproject.toml": f'[project]\ndependencies = ["harmos-sdk @ git+{source}@previous#subdirectory=sdk/python"]\n',
            }
            for relative, text in manifests.items():
                path = scratch / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text, encoding="utf-8")
            archive = scratch / "sdk/cpp/cmake/Dependencies.cmake"
            archive.parent.mkdir(parents=True)
            archive.write_text(f"URL {source}/archive/next.tar.gz\n", encoding="utf-8")
            library_dependencies(scratch)
            library_dependencies(scratch, check=True)
            for relative in manifests:
                with self.subTest(manifest=relative):
                    path = scratch / relative
                    current = path.read_text(encoding="utf-8")
                    if relative == "Cargo.toml":
                        stale = current.replace(
                            f'library-runtime = {{ git = "{source}", rev = "next"',
                            f'library-runtime = {{ git = "{source}", rev = "previous"',
                        )
                    else:
                        stale = current.replace("next", "previous")
                    path.write_text(stale, encoding="utf-8")
                    self.assertRaises(ValueError, library_dependencies, scratch, check=True)
                    library_dependencies(scratch)
                    self.assertEqual(path.read_text(encoding="utf-8"), current)
            archive.write_text(f"URL {source}/archive/previous.tar.gz\n", encoding="utf-8")
            self.assertRaises(ValueError, library_dependencies, scratch, check=True)
        finally:
            shutil.rmtree(scratch)

    def test_cpp_archive_follows_the_workspace_pin_and_refuses_stale_bytes(self) -> None:
        scratch = Path(tempfile.mkdtemp())
        try:
            manifest = scratch / "Cargo.toml"
            dependency = scratch / "sdk/cpp/cmake/Dependencies.cmake"
            dependency.parent.mkdir(parents=True)
            archive = b"First Archive"
            manifest.write_text('[workspace.dependencies.harmos]\ngit = "https://example.test/library"\nrev = "first"\n')
            with mock.patch("urllib.request.urlopen", side_effect=lambda url: BytesIO(archive)) as download:
                cpp_dependency(scratch)
                cpp_dependency(scratch, check=True)
                first = dependency.read_bytes()
                self.assertEqual(download.call_args.args[0], "https://example.test/library/archive/first.tar.gz")
                manifest.write_text(manifest.read_text().replace('"first"', '"second"'))
                self.assertRaises(ValueError, cpp_dependency, scratch, check=True)
                cpp_dependency(scratch)
                self.assertNotEqual(first, dependency.read_bytes())
                self.assertEqual(download.call_args.args[0], "https://example.test/library/archive/second.tar.gz")
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
