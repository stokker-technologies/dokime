# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Exact Pure Python Distribution Payloads."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations

import os
import tarfile
import tomllib
from collections.abc import Iterable
from email.message import Message
from email.parser import Parser
from pathlib import Path, PurePosixPath
from zipfile import ZipFile

# Third Party Imports
from packaging.requirements import Requirement
from packaging.specifiers import SpecifierSet

# Local Imports
from dokime_tools.check.wheel import verify_wheel as verify_wheel

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PACKAGES = {
    "dokime": {
        "distribution_name": "dokime",
        "module_paths": (
            "dokime/__init__.py",
            "dokime/py.typed",
            "dokime/client/__init__.py",
            "dokime/__init__.pyi",
            "dokime/contracts.json",
            "dokime/descriptors.bin",
            "dokime/_wire/_pool.py",
        ),
    },
}

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> int:
    dist = Path("dist/python")
    if not dist.exists():
        raise SystemExit("Missing dist/python/.")

    expected_version = _expected_version()
    project = tomllib.loads(Path("packages/dokime/pyproject.toml").read_text())[
        "project"
    ]
    required_dependencies = tuple(project["dependencies"])
    for package_name, expectations in PACKAGES.items():
        wheel_path = _artifact(
            dist,
            f"{package_name}-{expected_version}-*.whl",
            f"{package_name}-*.whl",
            package_name,
            "wheel",
        )
        sdist_path = _artifact(
            dist,
            (
                f"{package_name}-{expected_version}.tar.gz",
                f"{package_name.replace('_', '-')}-{expected_version}.tar.gz",
            ),
            (
                f"{package_name}-*.tar.gz",
                f"{package_name.replace('_', '-')}-*.tar.gz",
            ),
            package_name,
            "source distribution",
        )
        wheel_metadata = _read_wheel_metadata(wheel_path)
        sdist_metadata = _read_sdist_metadata(sdist_path)
        _verify_metadata(
            wheel_path,
            sdist_path,
            wheel_metadata,
            sdist_metadata,
            expectations["distribution_name"],
            required_dependencies,
        )
        for metadata in (wheel_metadata, sdist_metadata):
            requires_python = metadata.get_all("Requires-Python", [])
            if (
                len(requires_python) != 1
                or SpecifierSet(requires_python[0])
                != SpecifierSet(project["requires-python"])
                or metadata.get_all("Provides-Extra", [])
            ):
                raise SystemExit(
                    "Python runtime constraints differ from prepared source."
                )
            if metadata.get("License-Expression") != "Apache-2.0":
                raise SystemExit("Python distributions require Apache-2.0 metadata.")
        canonical_license = Path("LICENSE-APACHE").read_bytes()
        with ZipFile(wheel_path) as wheel:
            licenses = [
                name
                for name in wheel.namelist()
                if name.endswith("/licenses/LICENSE-APACHE")
            ]
            if len(licenses) != 1 or wheel.read(licenses[0]) != canonical_license:
                raise SystemExit("Wheel must preserve the canonical Apache license.")
        with tarfile.open(sdist_path) as archive:
            licenses = [
                item
                for item in archive.getmembers()
                if item.name.endswith("/LICENSE-APACHE")
            ]
            if len(licenses) != 1:
                raise SystemExit("Source archive must preserve one Apache license.")
            content = archive.extractfile(licenses[0])
            if content is None or content.read() != canonical_license:
                raise SystemExit(
                    "Source archive license differs from canonical source."
                )
        _verify_wheel_paths(wheel_path, expectations["module_paths"])
        _verify_wheel_payload(wheel_path, Path.cwd())
        _verify_sdist_build_inputs(sdist_path, Path.cwd())
        verify_wheel(wheel_path)

    return 0


# === Private Functions === #


def _latest_path(paths: Iterable[Path], package_name: str, artifact: str) -> Path:
    candidates = sorted(paths)
    if not candidates:
        raise SystemExit(f"Missing {package_name} {artifact} in dist/python/.")
    if len(candidates) > 1:
        names = ", ".join(path.name for path in candidates)
        raise SystemExit(
            f"Multiple {package_name} {artifact} artifacts found: {names}."
        )
    return candidates[0]


def _artifact(
    dist: Path,
    expected_patterns: str | tuple[str, ...],
    all_patterns: str | tuple[str, ...],
    package_name: str,
    artifact: str,
) -> Path:
    expected = _glob_many(dist, expected_patterns)
    stale = sorted(set(_glob_many(dist, all_patterns)) - set(expected))
    if stale:
        names = ", ".join(path.name for path in stale)
        raise SystemExit(f"Stale {package_name} {artifact} artifacts found: {names}.")
    return _latest_path(expected, package_name, artifact)


def _glob_many(dist: Path, patterns: str | tuple[str, ...]) -> list[Path]:
    if isinstance(patterns, str):
        patterns = (patterns,)
    paths: list[Path] = []
    for pattern in patterns:
        paths.extend(dist.glob(pattern))
    return sorted(set(paths))


def _verify_metadata(
    wheel_path: Path,
    sdist_path: Path,
    wheel_metadata: Message,
    sdist_metadata: Message,
    distribution_name: str,
    required_dependencies: tuple[str, ...],
) -> None:
    wheel_name = wheel_metadata.get("Name")
    sdist_name = sdist_metadata.get("Name")
    if _normalize_name(wheel_name) != _normalize_name(distribution_name):
        raise SystemExit(
            f"Wheel metadata name {wheel_name!r} does not match "
            f"{distribution_name!r} in {wheel_path}."
        )
    if _normalize_name(sdist_name) != _normalize_name(distribution_name):
        raise SystemExit(
            f"Source distribution metadata name {sdist_name!r} does not match "
            f"{distribution_name!r} in {sdist_path}."
        )

    wheel_version = _metadata_version(wheel_metadata, wheel_path)
    sdist_version = _metadata_version(sdist_metadata, sdist_path)
    if wheel_version != sdist_version:
        raise SystemExit(
            f"Wheel version {wheel_version} does not match source distribution "
            f"version {sdist_version} for {distribution_name}."
        )

    expected_version = os.environ.get("EXPECTED_PACKAGE_VERSION")
    if expected_version and wheel_version != expected_version:
        raise SystemExit(
            f"Wheel version {wheel_version} does not match expected {expected_version}."
        )

    _verify_required_dependencies(
        wheel_metadata, distribution_name, wheel_version, required_dependencies
    )
    _verify_required_dependencies(
        sdist_metadata, distribution_name, sdist_version, required_dependencies
    )


def _read_wheel_metadata(wheel_path: Path) -> Message:
    with ZipFile(wheel_path) as wheel:
        metadata_names = [
            name for name in wheel.namelist() if name.endswith(".dist-info/METADATA")
        ]
        if len(metadata_names) != 1:
            raise SystemExit(f"Missing wheel metadata in {wheel_path}.")
        metadata = wheel.read(metadata_names[0]).decode()

    return Parser().parsestr(metadata)


def _read_sdist_metadata(sdist_path: Path) -> Message:
    with tarfile.open(sdist_path, "r:gz") as sdist:
        pkg_info_members = [
            member for member in sdist.getmembers() if member.name.endswith("/PKG-INFO")
        ]
        if len(pkg_info_members) != 1:
            raise SystemExit(f"Missing source distribution metadata in {sdist_path}.")
        pkg_info = sdist.extractfile(pkg_info_members[0])
        if pkg_info is None:
            raise SystemExit(
                f"Cannot read source distribution metadata in {sdist_path}."
            )
        metadata = pkg_info.read().decode()

    return Parser().parsestr(metadata)


def _metadata_version(metadata: Message, path: Path) -> str:
    version = metadata.get("Version")
    if not version:
        raise SystemExit(f"Missing version metadata in {path}.")
    return version


def _expected_version() -> str:
    explicit_version = os.environ.get("EXPECTED_PACKAGE_VERSION")
    if explicit_version:
        return explicit_version

    data = tomllib.loads(
        Path("packages/dokime/pyproject.toml").read_text(encoding="utf-8")
    )
    project = data.get("project")
    if not isinstance(project, dict) or not isinstance(project.get("version"), str):
        raise SystemExit("pyproject.toml is missing [project].version.")
    return project["version"]


def _verify_required_dependencies(
    metadata: Message,
    distribution_name: str,
    version: str,
    required_dependencies: tuple[str, ...],
) -> None:
    expected = {
        str(Requirement(value.format(version=version)))
        for value in required_dependencies
    }
    values = metadata.get_all("Requires-Dist", [])
    actual = [str(Requirement(value)) for value in values]
    if len(actual) != len(set(actual)) or set(actual) != expected:
        raise SystemExit(f"{distribution_name} metadata dependency inventory mismatch.")


def _normalize_name(name: str | None) -> str:
    if not name:
        return ""
    return name.replace("_", "-").lower()


def _verify_wheel_paths(wheel_path: Path, required_paths: tuple[str, ...]) -> None:
    with ZipFile(wheel_path) as wheel:
        names = set(wheel.namelist())

    for required_path in required_paths:
        if required_path not in names:
            raise SystemExit(
                f"Missing packaged file in {wheel_path.name}: {required_path}."
            )


def _python_inputs(source_root: Path) -> list[Path]:
    python_root = source_root / "packages/dokime/src/dokime"
    files = sorted(
        path
        for path in python_root.rglob("*")
        if path.is_file()
        and (
            path.suffix in {".py", ".pyi"}
            or path.name in {"py.typed", "contracts.json", "descriptors.bin"}
        )
    )
    if not files:
        raise SystemExit("Missing Python sources for package verification.")
    return files


def _verify_wheel_payload(wheel_path: Path, source_root: Path) -> None:
    python_root = source_root / "packages/dokime/src"
    expected = {
        path.relative_to(python_root).as_posix(): path
        for path in _python_inputs(source_root)
    }
    with ZipFile(wheel_path) as wheel:
        names = wheel.namelist()
        if len(names) != len(set(names)):
            raise SystemExit(f"Duplicate wheel entries in {wheel_path.name}.")
        if any(name.startswith("dokime_client/") for name in names):
            raise SystemExit(f"Obsolete dokime_client package in {wheel_path.name}.")
        actual = {
            name
            for name in names
            if name.startswith("dokime/")
            and (
                name.endswith((".py", ".pyi"))
                or name.rsplit("/", 1)[-1]
                in {"py.typed", "contracts.json", "descriptors.bin"}
            )
        }
        if actual != expected.keys():
            raise SystemExit(
                f"Wheel source inventory mismatch in {wheel_path.name}: "
                f"missing={sorted(expected.keys() - actual)}, "
                f"unexpected={sorted(actual - expected.keys())}."
            )
        if any(name.endswith((".so", ".pyd", ".dll", ".dylib")) for name in names):
            raise SystemExit(f"Native payload in pure client wheel {wheel_path.name}.")

        for name, source in expected.items():
            if wheel.read(name) != source.read_bytes():
                raise SystemExit(f"Stale wheel source in {wheel_path.name}: {name}.")


def _verify_sdist_build_inputs(sdist_path: Path, source_root: Path) -> None:
    """Verify the prepared client source inventory without backend build inputs."""
    package = source_root / "packages/dokime"
    inputs = _python_inputs(source_root)
    inputs.extend(
        package / name for name in ("pyproject.toml", "README.md", "LICENSE-APACHE")
    )
    inputs.extend(
        sorted(path for path in (package / "examples").rglob("*.py") if path.is_file())
    )
    prefix = sdist_path.name.removesuffix(".tar.gz")
    with tarfile.open(sdist_path, "r:gz") as archive:
        entries = archive.getmembers()
        names = [entry.name for entry in entries]
        if len(names) != len(set(names)):
            raise SystemExit("Duplicate source distribution entries.")
        members = {entry.name: entry for entry in entries}
        for entry in entries:
            relative = PurePosixPath(entry.name)
            if (
                relative.is_absolute()
                or relative.as_posix() != entry.name
                or "\\" in entry.name
                or ".." in relative.parts
                or not relative.parts
                or relative.parts[0] != prefix
                or not (entry.isfile() or entry.isdir())
                or relative.name in {"Cargo.toml", "Cargo.lock"}
                or relative.suffix in {".rs", ".so", ".pyd", ".dll", ".dylib"}
            ):
                raise SystemExit("Invalid or native source distribution entry.")
        for source in inputs:
            relative = source.relative_to(package).as_posix()
            if not source.is_file() or source.is_symlink():
                raise SystemExit(f"Missing source build input: {relative}.")
            member = members.get(f"{prefix}/{relative}")
            if member is None or not member.isfile():
                raise SystemExit(
                    f"Missing source distribution build input: {relative}."
                )
            content = archive.extractfile(member)
            if content is None or content.read() != source.read_bytes():
                raise SystemExit(f"Stale source distribution build input: {relative}.")
        actual = {
            entry.name.removeprefix(prefix + "/")
            for entry in entries
            if entry.isfile() and entry.name.startswith(prefix + "/src/dokime/")
        }
        expected = {
            path.relative_to(package).as_posix() for path in _python_inputs(source_root)
        }
        if actual != expected:
            raise SystemExit("Source distribution Python inventory mismatch.")
        expected_files = {path.relative_to(package).as_posix() for path in inputs}
        expected_files.add("PKG-INFO")
        ignore = source_root / ".gitignore"
        if ignore.is_file():
            # Hatch Preserves The Repository Ignore Rules In Its Source Archive.
            expected_files.add(".gitignore")
            member = members.get(f"{prefix}/.gitignore")
            content = archive.extractfile(member) if member is not None else None
            if content is None or content.read() != ignore.read_bytes():
                raise SystemExit("Stale source distribution ignore rules.")
        actual_files = {
            entry.name.removeprefix(prefix + "/") for entry in entries if entry.isfile()
        }
        if actual_files != expected_files:
            raise SystemExit("Source distribution build inventory mismatch.")


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
