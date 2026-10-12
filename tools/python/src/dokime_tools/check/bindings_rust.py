# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Parse PyO3 declarations without compiling the native extension."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from dataclasses import dataclass
from pathlib import Path
import re

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

__all__ = ["parse_rust_surface"]

# === Constants === #

PYCLASS_RE = re.compile(
    r"#\[pyclass(?:\((?P<attributes>.*?)\))?\]\s*"
    r"(?:(?:#\[[^\]]+\]|///[^\n]*)\s*)*"
    r"(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum)\s+(?P<rust_name>[A-Za-z_]\w*)",
    re.DOTALL,
)
PYFUNCTION_RE = re.compile(
    r"#\[pyfunction(?:\([^\]]*\))?\]\s*"
    r"(?:(?:#\[[^\]]+\]|///[^\n]*)\s*)*"
    r"(?:pub(?:\([^)]*\))?\s+)?fn\s+(?P<name>[A-Za-z_]\w*)\s*\("
)
WRAPPED_FUNCTION_RE = re.compile(
    r"wrap_pyfunction!\(\s*(?P<path>[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*)\s*,"
)
PYMETHODS_RE = re.compile(
    r"#\[pymethods\]\s*"
    r"(?:(?:#\[[^\]]+\]|///[^\n]*)\s*)*"
    r"impl(?:<[^>]*>)?\s+(?P<type>[A-Za-z_]\w*)"
)
PYNAME_RE = re.compile(r"\bname\s*=\s*\"(?P<name>[^\"]+)\"")
MODULE_RE = re.compile(r"\bmodule\s*=\s*\"(?P<module>[^\"]+)\"")
PYMODULE_RE = re.compile(r"#\[pymodule(?:\([^\]]*\))?\]")
PYCLASS_MARKER_RE = re.compile(r"#\[pyclass(?:\(|\])")
PYFUNCTION_MARKER_RE = re.compile(r"#\[pyfunction(?:\(|\])")
PYMETHODS_MARKER_RE = re.compile(r"#\[pymethods\]")

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class RustClass:
    rust_name: str
    python_name: str
    explicit_name: bool
    module: str | None
    path: Path


@dataclass(frozen=True)
class RustMethods:
    rust_type: str
    names: frozenset[str]
    path: Path


@dataclass(frozen=True)
class RustSurface:
    classes: tuple[RustClass, ...]
    functions: frozenset[str]
    registered_functions: frozenset[str]
    registered_classes: frozenset[str]
    methods: tuple[RustMethods, ...]
    module_registration: bool
    declaration_errors: tuple[str, ...]
    submodules: frozenset[str]
    exceptions: frozenset[str]


# === Public Functions === #


def parse_rust_surface(root: Path, *, exclude: tuple[Path, ...] = ()) -> RustSurface:
    """Return PyO3 declarations and module registration from a Rust source tree."""

    classes: list[RustClass] = []
    functions: set[str] = set()
    registered_functions: set[str] = set()
    registered_classes: set[str] = set()
    methods: list[RustMethods] = []
    declaration_errors: list[str] = []
    module_registration = False
    submodules: set[str] = set()
    exceptions: set[str] = set()
    module_root = root / "lib.rs"
    if not module_root.exists():
        module_root = root / "mod.rs"

    for path in sorted(root.rglob("*.rs")):
        if any(path.is_relative_to(directory) for directory in exclude):
            continue
        text = path.read_text(encoding="utf-8")
        module_registration |= bool(PYMODULE_RE.search(text))
        class_matches = tuple(PYCLASS_RE.finditer(text))
        declaration_errors.extend(
            _unmatched_markers(
                path,
                text,
                PYCLASS_MARKER_RE,
                {match.start() for match in class_matches},
                "pyclass",
            )
        )
        for match in class_matches:
            attributes = match.group("attributes") or ""
            rust_name = match.group("rust_name")
            python_name = _attribute(PYNAME_RE, attributes) or rust_name.removeprefix(
                "Py"
            )
            classes.append(
                RustClass(
                    rust_name=rust_name,
                    python_name=python_name,
                    explicit_name=_attribute(PYNAME_RE, attributes) is not None,
                    module=_attribute(MODULE_RE, attributes),
                    path=path,
                )
            )
        function_matches = tuple(PYFUNCTION_RE.finditer(text))
        declaration_errors.extend(
            _unmatched_markers(
                path,
                text,
                PYFUNCTION_MARKER_RE,
                {match.start() for match in function_matches},
                "pyfunction",
            )
        )
        functions.update(match.group("name") for match in function_matches)
        if path == module_root:
            submodules.update(re.findall(r"wrap_pymodule!\(\s*(\w+)::", text))
            registered_classes.update(
                match.group("path").rsplit("::", 1)[-1]
                for match in re.finditer(
                    r"add_class::<\s*(?P<path>[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*)\s*>\s*\(",
                    text,
                )
            )
            registered_functions.update(
                match.group("path").rsplit("::", 1)[-1]
                for match in WRAPPED_FUNCTION_RE.finditer(text)
            )
        for exception in re.findall(r"create_exception!\(\s*\w+\s*,\s*(\w+)\s*,", text):
            exceptions.add(exception)
            if not re.search(rf'\.add\(\s*"{re.escape(exception)}"\s*,', text):
                declaration_errors.append(
                    f"{path}: exception {exception} is not registered"
                )
        method_matches = tuple(PYMETHODS_RE.finditer(text))
        declaration_errors.extend(
            _unmatched_markers(
                path,
                text,
                PYMETHODS_MARKER_RE,
                {match.start() for match in method_matches},
                "pymethods",
            )
        )
        methods.extend(_parse_methods(path, text, method_matches))

    return RustSurface(
        tuple(classes),
        frozenset(functions),
        frozenset(registered_functions),
        frozenset(registered_classes),
        tuple(methods),
        module_registration,
        tuple(declaration_errors),
        frozenset(submodules),
        frozenset(exceptions),
    )


# === Private Functions === #


def _parse_methods(
    path: Path, text: str, markers: tuple[re.Match[str], ...] | None = None
) -> list[RustMethods]:
    parsed: list[RustMethods] = []
    for marker in markers or tuple(PYMETHODS_RE.finditer(text)):
        open_brace = text.find("{", marker.end())
        if open_brace < 0:
            continue
        close_brace = _matching_brace(text, open_brace)
        body = text[open_brace + 1 : close_brace]
        names: set[str] = set()
        for method in re.finditer(
            r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+([A-Za-z_]\w*)\s*(?:<[^>]*>)?\s*\(",
            body,
        ):
            name = method.group(1)
            if name == "new":
                name = "__init__"
            if name.startswith("__"):
                continue
            names.add(_pyo3_method_name(body, method.start(), name))
        parsed.append(RustMethods(marker.group("type"), frozenset(names), path))
    return parsed


def _unmatched_markers(
    path: Path,
    text: str,
    marker_pattern: re.Pattern[str],
    matched_starts: set[int],
    label: str,
) -> list[str]:
    errors: list[str] = []
    for marker in marker_pattern.finditer(text):
        if marker.start() in matched_starts:
            continue
        line = text.count("\n", 0, marker.start()) + 1
        errors.append(f"{path}:{line}: unmatched #[{label}] declaration")
    return errors


def _pyo3_method_name(body: str, start: int, default: str) -> str:
    attributes: list[str] = []
    cursor = body.rfind("\n", 0, start)
    while cursor >= 0:
        line_start = body.rfind("\n", 0, cursor) + 1
        line = body[line_start:cursor].strip()
        if line.startswith("#["):
            attributes.append(line)
        elif line:
            break
        cursor = line_start - 1
    match = re.search(
        r"#\[pyo3\([^\]]*\bname\s*=\s*\"([^\"]+)\"",
        "\n".join(attributes),
    )
    return match.group(1) if match else default


def _matching_brace(text: str, opening: int) -> int:
    depth = 0
    quote: str | None = None
    escaped = False
    line_comment = False
    block_comment = False
    index = opening
    while index < len(text):
        char = text[index]
        next_char = text[index + 1] if index + 1 < len(text) else ""
        if line_comment:
            if char == "\n":
                line_comment = False
        elif block_comment:
            if char == "*" and next_char == "/":
                block_comment = False
                index += 1
        elif quote:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == quote:
                quote = None
        elif char == "/" and next_char == "/":
            line_comment = True
            index += 1
        elif char == "/" and next_char == "*":
            block_comment = True
            index += 1
        elif char == '"':
            quote = char
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
        index += 1
    raise ValueError("Unclosed Rust brace in #[pymethods] block")


def _attribute(pattern: re.Pattern[str], attributes: str) -> str | None:
    match = pattern.search(attributes)
    if not match:
        return None
    return next(iter(match.groupdict().values()))


# === Entry Point === #
