# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Regression tests for static PyO3 declaration and stub validation."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from pathlib import Path
import shutil
from tempfile import TemporaryDirectory
import textwrap
import unittest

# Third Party Imports

# Local Imports
from dokime_tools.check.bindings import _configured_paths, check_static

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class BindingCheckerTests(unittest.TestCase):
    def case_repository_stub(self):
        root = Path(__file__).resolve().parents[4]
        self.assertEqual(
            check_static(
                root / "crates/dokime-python/src",
                root / "crates/dokime-python/python/dokime/_native/__init__.pyi",
            ),
            [],
        )

    def case_nested_registration(self):
        repository = Path(__file__).resolve().parents[4]
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            rust = root / "src"
            stubs = root / "stubs"
            shutil.copytree(repository / "crates/dokime-python/src", rust)
            shutil.copytree(
                repository / "crates/dokime-python/python/dokime/_native", stubs
            )
            client_stub = stubs / "client.pyi"
            original = client_stub.read_text()
            client_stub.write_text(
                original.replace("def verify(", "def renamed_verify(")
            )
            errors = check_static(rust, stubs / "__init__.pyi")
            self.assertTrue(
                any("Client.verify is missing" in error for error in errors)
            )

            client_stub.write_text(original)
            client_rust = rust / "client/mod.rs"
            client_rust.write_text(
                client_rust.read_text().replace(
                    'module.add("ClientError", exception)?;', ""
                )
            )
            errors = check_static(rust, stubs / "__init__.pyi")
            self.assertTrue(
                any(
                    "exception ClientError is not registered" in error
                    for error in errors
                )
            )

    def case_module_path(self):
        with TemporaryDirectory() as temporary:
            config = Path(temporary) / "stkr.toml"
            config.write_text(
                '[[pyo3.modules]]\nroot = true\nrust = "src"\nstub = "pkg/__init__.pyi"\nimport = "pkg.native"\n'
            )
            rust, stub, module, *_ = _configured_paths(config)
            self.assertEqual(rust, Path(temporary) / "src")
            self.assertEqual(stub, Path(temporary) / "pkg/__init__.pyi")
            self.assertEqual(module, "pkg.native")
            for text in [
                '[pyo3]\nrust = "src"\n',
                "[[pyo3.modules]]\nroot = false\n",
                "[[pyo3.modules]]\nroot = true\n[[pyo3.modules]]\nroot = true\n",
            ]:
                config.write_text(text)
                with self.assertRaises(ValueError):
                    _configured_paths(config)

    def case_missing_reported(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            rust = root / "src"
            rust.mkdir()
            (rust / "lib.rs").write_text(
                textwrap.dedent(
                    """
                #[pyclass(name = "Thing", module = "dokime.runtime")]
                #[derive(Debug)]
                /// A fixture wrapper with intervening attributes.
                pub struct PyThing;
                #[pymethods]
                impl PyThing {
                    pub fn run(&self) {}
                }
                #[pyfunction]
                pub(crate) fn ping() {}
                #[pymodule]
                fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
                    module.add_class::<PyThing>()?;
                    module.add_function(wrap_pyfunction!(ping, module)?)?;
                    Ok(())
                }
                """
                ),
                encoding="utf-8",
            )
            stub = root / "_native.pyi"
            stub.write_text(
                textwrap.dedent(
                    """
                class Thing: ...
                def ping() -> None: ...
                """
                ),
                encoding="utf-8",
            )

            errors = check_static(rust, stub)

        self.assertTrue(
            any("Thing.run is missing from the stub" in error for error in errors)
        )

    def case_unmatched_reported(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            rust = root / "src"
            rust.mkdir()
            (rust / "lib.rs").write_text(
                textwrap.dedent(
                    """
                #[pyclass]
                /// This marker has no wrapper declaration.
                fn not_a_class() {}
                #[pymodule]
                fn _native(_module: &Bound<'_, PyModule>) -> PyResult<()> { Ok(()) }
                """
                ),
                encoding="utf-8",
            )
            stub = root / "_native.pyi"
            stub.write_text("", encoding="utf-8")

            errors = check_static(rust, stub)

        self.assertTrue(any("unmatched #[pyclass]" in error for error in errors))

    def case_unregistered_reported(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            rust = root / "src"
            rust.mkdir()
            (rust / "lib.rs").write_text(
                textwrap.dedent(
                    """
                #[pyclass]
                pub(crate) struct PyThing;
                #[pymodule]
                fn _native(_module: &Bound<'_, PyModule>) -> PyResult<()> { Ok(()) }
                """
                ),
                encoding="utf-8",
            )
            stub = root / "_native.pyi"
            stub.write_text("class Thing: ...", encoding="utf-8")

            errors = check_static(rust, stub)

        self.assertTrue(any("explicit Python name" in error for error in errors))
        self.assertTrue(any("is not registered" in error for error in errors))

    def test_contract(self):
        for scenario in (
            "case_repository_stub",
            "case_nested_registration",
            "case_module_path",
            "case_missing_reported",
            "case_unmatched_reported",
            "case_unregistered_reported",
        ):
            with self.subTest(scenario=scenario):
                type(self)(scenario).debug()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
