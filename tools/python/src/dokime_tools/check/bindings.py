# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Validate the static and built surfaces of the Dokime PyO3 extension."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
import argparse
import ast
from dataclasses import dataclass
import importlib
import inspect
from pathlib import Path
import re
import sys
import tomllib
from collections.abc import Mapping

# Third Party Imports

# Local Imports
from dokime_tools.check.bindings_rust import parse_rust_surface

# Type Checking Imports

# === Public Exports === #

__all__ = ["check_runtime", "check_static", "main"]

# === Constants === #

DEFAULT_RUST_ROOT = Path("crates/dokime-python/src")
DEFAULT_STUB = Path("crates/dokime-python/python/dokime/_native/__init__.pyi")
DEFAULT_MODULE = "dokime._native"
DEFAULT_NATIVE_CLASS_MODULES = {
    "Runtime": "dokime._native",
    "AuthoringContext": "dokime._native",
    "Artifacts": "dokime._native",
    "ArtifactWriter": "dokime._native",
}
DEFAULT_PUBLIC_CLASS_MODULES = {
    "Runtime": "dokime._native",
    "AuthoringContext": "dokime._native",
    "Artifacts": "dokime._native",
    "ArtifactWriter": "dokime._native",
    "ServerConfig": "dokime.runtime",
    "TlsIdentity": "dokime.runtime",
    "BearerGrant": "dokime.runtime",
    "Role": "dokime.runtime",
}

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class StubSurface:
    exports: frozenset[str]
    classes: frozenset[str]
    runtime_exports: frozenset[str]
    class_methods: dict[str, set[str]]


# === Public Functions === #


def check_static(
    rust_root: Path = DEFAULT_RUST_ROOT,
    stub: Path = DEFAULT_STUB,
    *,
    expected_module: str = DEFAULT_MODULE,
    expected_class_module: str = "dokime.runtime",
    native_class_modules: Mapping[str, str] = DEFAULT_NATIVE_CLASS_MODULES,
) -> list[str]:
    """Compare Rust PyO3 declarations and registration with the native stub."""

    module_path = rust_root / "lib.rs"
    if not module_path.exists():
        module_path = rust_root / "mod.rs"
    root_text = module_path.read_text(encoding="utf-8") if module_path.exists() else ""
    children = set(re.findall(r"wrap_pymodule!\(\s*(\w+)::", root_text))
    flattened = set(
        re.findall(r"(?:add_class::<|wrap_pyfunction!\()\s*(\w+)::", root_text)
    )
    nested = children - flattened
    surface = parse_rust_surface(
        rust_root, exclude=tuple(rust_root / name for name in nested)
    )
    stub_surface = _parse_stub(stub)
    errors: list[str] = list(surface.declaration_errors)

    if not surface.module_registration:
        errors.append(f"{rust_root / 'lib.rs'}: missing #[pymodule] registration")

    class_names = {item.python_name for item in surface.classes}
    function_names = set(surface.functions)
    rust_exports = (
        class_names | function_names | set(surface.submodules) | set(surface.exceptions)
    )

    for item in surface.classes:
        if not item.rust_name.startswith("Py"):
            errors.append(
                f"{item.path}: PyO3 wrapper {item.rust_name} must start with Py"
            )
        if not item.explicit_name:
            errors.append(
                f"{item.path}: {item.rust_name} needs an explicit Python name"
            )
        expected_item_module = native_class_modules.get(
            item.python_name, expected_class_module
        )
        if item.module != expected_item_module:
            errors.append(
                f"{item.path}: {item.rust_name} exports module {item.module!r}; "
                f"expected {expected_item_module!r}"
            )

    for missing in sorted(rust_exports - stub_surface.exports):
        errors.append(f"{stub}: missing Rust export {missing}")
    for extra in sorted(stub_surface.exports - rust_exports):
        errors.append(f"{stub}: no mapped Rust export for {extra}")

    missing_registration = sorted(surface.functions - surface.registered_functions)
    for name in missing_registration:
        errors.append(f"{rust_root}: #[pyfunction] {name} is not registered")
    unknown_registration = sorted(surface.registered_functions - surface.functions)
    for name in unknown_registration:
        errors.append(f"{rust_root}: registered function {name} has no #[pyfunction]")

    missing_class_registration = sorted(
        item.rust_name
        for item in surface.classes
        if item.rust_name not in surface.registered_classes
    )
    for name in missing_class_registration:
        errors.append(f"{rust_root}: #[pyclass] {name} is not registered")

    for methods in surface.methods:
        class_item = next(
            (item for item in surface.classes if item.rust_name == methods.rust_type),
            None,
        )
        if class_item is None:
            errors.append(
                f"{methods.path}: #[pymethods] target {methods.rust_type} "
                "has no #[pyclass] declaration"
            )
            continue
        stub_methods = stub_surface.class_methods.get(class_item.python_name, set())
        for missing in sorted(methods.names - stub_methods):
            errors.append(
                f"{stub}: {class_item.python_name}.{missing} is missing from the stub"
            )

    # The module argument is checked separately so a renamed extension remains
    # explicit in the configuration and cannot silently drift.
    if not module_path.is_file():
        errors.append(f"Missing Rust module root: {module_path}")
    elif not re.search(
        rf"\bfn\s+{re.escape(expected_module.rsplit('.', 1)[-1])}\s*\(",
        module_path.read_text(encoding="utf-8"),
    ):
        errors.append(
            f"{module_path}: missing #[pymodule] function for {expected_module}"
        )

    for name in sorted(nested):
        child_stub = stub.parent / f"{name}.pyi"
        if not child_stub.exists():
            errors.append(f"Missing Rust child module stub: {child_stub}")
            continue
        errors.extend(
            check_static(
                rust_root / name,
                child_stub,
                expected_module=f"{expected_module}.{name}",
                expected_class_module=f"{expected_module}.{name}",
                native_class_modules={},
            )
        )

    return errors


def check_runtime(
    stub: Path = DEFAULT_STUB,
    *,
    module_name: str = DEFAULT_MODULE,
    expected_class_module: str = "dokime.runtime",
    native_class_modules: Mapping[str, str] = DEFAULT_NATIVE_CLASS_MODULES,
    public_class_modules: Mapping[str, str] = DEFAULT_PUBLIC_CLASS_MODULES,
    package_modules: tuple[str, ...] = (
        "dokime",
        "dokime.client",
        "dokime.core",
        "dokime.host",
        "dokime.records",
        "dokime.runtime",
        "dokime.suites",
        "dokime.telemetry",
    ),
) -> list[str]:
    """Verify imports and that built native exports have matching stubs."""

    errors: list[str] = []
    for name in package_modules:
        try:
            importlib.import_module(name)
        except Exception as error:  # pragma: no cover - depends on built wheel
            errors.append(f"{name}: import failed: {type(error).__name__}: {error}")

    try:
        native = importlib.import_module(module_name)
    except Exception as error:  # pragma: no cover - depends on built wheel
        errors.append(f"{module_name}: import failed: {type(error).__name__}: {error}")
        return errors

    package_root = stub.parent.parent if stub.name == "__init__.pyi" else stub.parent
    host_stub = package_root / "host" / "__init__.pyi"
    if host_stub.is_file() and hasattr(native, "host_stream_contracts"):
        declarations = ast.parse(host_stub.read_text(encoding="utf-8"))
        written = {
            item.name: {
                field.target.id: ast.unparse(field.annotation)
                for field in item.body
                if isinstance(field, ast.AnnAssign)
                and isinstance(field.target, ast.Name)
                and not (
                    isinstance(field.annotation, ast.Subscript)
                    and isinstance(field.annotation.value, ast.Name)
                    and field.annotation.value.id == "ClassVar"
                )
            }
            for item in declarations.body
            if isinstance(item, ast.ClassDef) and not item.name.startswith("_")
        }
        expected = {
            name: definition["fields"]
            for name, definition in native.host_stream_contracts().items()
        }
        if written != expected:
            errors.append(
                f"{host_stub}: host row types differ from the canonical Rust field table"
            )

    stub_surface = _parse_stub(stub)
    actual = {name for name in dir(native) if not name.startswith("_")}
    missing = sorted(stub_surface.exports - actual)
    extra = sorted(actual - stub_surface.exports)
    if missing or extra:
        errors.append(
            f"{module_name}: runtime/stub mismatch; missing={missing}, extra={extra}"
        )

    for class_name in stub_surface.classes:
        exported = getattr(native, class_name, None)
        if exported is None:
            continue
        if not inspect.isclass(exported):
            errors.append(f"{module_name}.{class_name}: runtime export is not a class")
            continue
        expected_item_module = native_class_modules.get(
            class_name, expected_class_module
        )
        if exported.__module__ != expected_item_module:
            errors.append(
                f"{module_name}.{class_name}: runtime module is "
                f"{exported.__module__!r}, expected {expected_item_module!r}"
            )
        if exported.__module__ == expected_item_module:
            native_module = importlib.import_module(exported.__module__)
            if getattr(native_module, exported.__name__, None) is not exported:
                errors.append(
                    f"{module_name}.{class_name}: {exported.__module__}."
                    f"{exported.__name__} does not resolve to the native class"
                )
        public_module_name = public_class_modules.get(class_name, expected_class_module)
        try:
            public_module = importlib.import_module(public_module_name)
        except Exception as error:  # pragma: no cover - built wheel dependent
            errors.append(
                f"{class_name}: public module {public_module_name} import failed: "
                f"{type(error).__name__}: {error}"
            )
            continue
        public_class = getattr(public_module, class_name, None)
        if public_class is not exported:
            errors.append(
                f"{module_name}.{class_name}: public path "
                f"{public_module_name}.{class_name} does not resolve to the native class"
            )

    package_root = stub.parent.parent if stub.name == "__init__.pyi" else stub.parent
    for public_stub in sorted(package_root.rglob("*.pyi")):
        public_module = _module_name(package_root, public_stub)
        if public_module == module_name:
            continue
        try:
            loaded = importlib.import_module(public_module)
        except Exception as error:  # pragma: no cover - depends on built wheel
            errors.append(
                f"{public_module}: import failed: {type(error).__name__}: {error}"
            )
            continue
        expected = _parse_stub(public_stub).runtime_exports
        missing = sorted(expected - set(dir(loaded)))
        if missing:
            errors.append(
                f"{public_module}: stub exports missing at runtime: {missing}"
            )

    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--runtime",
        action="store_true",
        help="Import the built wheel and verify runtime exports against stubs.",
    )
    parser.add_argument("--rust-root", type=Path, default=DEFAULT_RUST_ROOT)
    parser.add_argument("--stub", type=Path, default=DEFAULT_STUB)
    parser.add_argument(
        "--config",
        type=Path,
        default=Path("stkr.toml"),
        help="stkr project file containing the [[pyo3.modules]] binding mappings.",
    )
    args = parser.parse_args(argv)
    (
        rust_root,
        stub,
        module_name,
        class_module,
        native_class_modules,
        public_class_modules,
    ) = _configured_paths(args.config)
    errors = (
        check_runtime(
            stub,
            module_name=module_name,
            expected_class_module=class_module,
            native_class_modules=native_class_modules,
            public_class_modules=public_class_modules,
        )
        if args.runtime
        else check_static(
            rust_root,
            stub,
            expected_module=module_name,
            expected_class_module=class_module,
            native_class_modules=native_class_modules,
        )
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


# === Private Functions === #


def _parse_stub(path: Path) -> StubSurface:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    exports: set[str] = set()
    classes: set[str] = set()
    typed_dicts: set[str] = set()
    class_methods: dict[str, set[str]] = {}

    for node in tree.body:
        if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
            if not node.name.startswith("_"):
                exports.add(node.name)
            if isinstance(node, ast.ClassDef):
                classes.add(node.name)
                if any(
                    isinstance(base, ast.Name) and base.id == "TypedDict"
                    for base in node.bases
                ):
                    typed_dicts.add(node.name)
                class_methods[node.name] = {
                    child.name
                    for child in node.body
                    if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef))
                    and not child.name.startswith("__")
                }
                class_methods[node.name].update(
                    child.target.id
                    for child in node.body
                    if isinstance(child, ast.AnnAssign)
                    and isinstance(child.target, ast.Name)
                    and not child.target.id.startswith("__")
                )
        elif isinstance(node, ast.ImportFrom):
            exports.update(
                alias.asname
                for alias in node.names
                if alias.asname is not None and not alias.asname.startswith("_")
            )
        elif isinstance(node, ast.Assign):
            if any(
                isinstance(target, ast.Name) and target.id == "__all__"
                for target in node.targets
            ):
                value = ast.literal_eval(node.value)
                if not isinstance(value, (list, tuple)) or not all(
                    isinstance(item, str) for item in value
                ):
                    raise ValueError(f"{path}: __all__ must be a static list of names")
                exports = set(value)

    return StubSurface(
        frozenset(exports),
        frozenset(classes),
        frozenset(exports - typed_dicts),
        class_methods,
    )


def _configured_paths(
    config_path: Path,
) -> tuple[Path, Path, str, str, Mapping[str, str], Mapping[str, str]]:
    if not config_path.is_file():
        return (
            DEFAULT_RUST_ROOT,
            DEFAULT_STUB,
            DEFAULT_MODULE,
            "dokime.runtime",
            DEFAULT_NATIVE_CLASS_MODULES,
            DEFAULT_PUBLIC_CLASS_MODULES,
        )

    config = tomllib.loads(config_path.read_text(encoding="utf-8"))
    pyo3 = config.get("pyo3", {})
    if not isinstance(pyo3, dict):
        raise ValueError(f"{config_path}: [pyo3] must be a table")

    modules = pyo3.get("modules")
    if not isinstance(modules, list) or any(
        not isinstance(entry, dict) for entry in modules
    ):
        raise ValueError(f"{config_path}: declare [[pyo3.modules]] mappings")
    roots = [entry for entry in modules if entry.get("root") is True]
    if len(roots) != 1:
        raise ValueError(f"{config_path}: declare exactly one root PyO3 module")
    pyo3 = roots[0]

    root = config_path.parent
    rust = pyo3.get("rust", str(DEFAULT_RUST_ROOT))
    stub = pyo3.get("stub", str(DEFAULT_STUB))
    module = pyo3.get("import", DEFAULT_MODULE)
    class_module = pyo3.get("class_module", "dokime.runtime")
    native_classes = pyo3.get("native_classes", DEFAULT_NATIVE_CLASS_MODULES)
    public_classes = pyo3.get("public_classes", DEFAULT_PUBLIC_CLASS_MODULES)
    if not all(isinstance(value, str) for value in (rust, stub, module, class_module)):
        raise ValueError(
            f"{config_path}: root PyO3 mapping paths and module names must be strings"
        )
    if not isinstance(native_classes, dict) or not all(
        isinstance(key, str) and isinstance(value, str)
        for key, value in native_classes.items()
    ):
        raise ValueError(
            f"{config_path}: [pyo3.modules.native_classes] must map names to modules"
        )
    if not isinstance(public_classes, dict) or not all(
        isinstance(key, str) and isinstance(value, str)
        for key, value in public_classes.items()
    ):
        raise ValueError(
            f"{config_path}: [pyo3.modules.public_classes] must map names to modules"
        )
    return (
        root / rust,
        root / stub,
        module,
        class_module,
        native_classes,
        public_classes,
    )


def _module_name(package_root: Path, stub: Path) -> str:
    relative = stub.relative_to(package_root)
    parts = list(relative.parts)
    if parts[-1] == "__init__.pyi":
        parts.pop()
    else:
        parts[-1] = Path(parts[-1]).stem
    return ".".join((package_root.name, *parts))


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
