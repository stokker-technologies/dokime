# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify expected Rust-era Python package build artifacts exist."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
import os
import tarfile
import tomllib
from collections.abc import Iterable
from email.message import Message
from email.parser import Parser
from pathlib import Path
from zipfile import ZipFile

# Third Party Imports

# Local Imports
from dokime_tools.check.native_wheel import verify_native_wheel

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PACKAGES = {
    "dokime": {
        "distribution_name": "dokime",
        "module_paths": (
            "dokime/__init__.py",
            "dokime/core/__init__.py",
            "dokime/_native/__init__.pyi",
            "dokime/py.typed",
            "dokime/client/__init__.py",
            "dokime/client/_generated/dokime/v1/execution_pb2.pyi",
        ),
        "requires_dist": ("protobuf>=7.35.1,<8",),
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
            expectations["requires_dist"],
        )
        for metadata in (wheel_metadata, sdist_metadata):
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
        verify_native_wheel(wheel_path)

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
        if not metadata_names:
            raise SystemExit(f"Missing wheel metadata in {wheel_path}.")
        metadata = wheel.read(metadata_names[0]).decode()

    return Parser().parsestr(metadata)


def _read_sdist_metadata(sdist_path: Path) -> Message:
    with tarfile.open(sdist_path, "r:gz") as sdist:
        pkg_info_members = [
            member for member in sdist.getmembers() if member.name.endswith("/PKG-INFO")
        ]
        if not pkg_info_members:
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

    data = tomllib.loads(Path("pyproject.toml").read_text(encoding="utf-8"))
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
    if not required_dependencies:
        return

    actual = set(metadata.get_all("Requires-Dist", []))
    for dependency in required_dependencies:
        expected = dependency.format(version=version)
        if expected not in actual:
            raise SystemExit(
                f"{distribution_name} metadata is missing Requires-Dist: {expected}."
            )


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
    python_root = source_root / "crates/dokime-python/python/dokime"
    files = sorted(
        path
        for path in python_root.rglob("*")
        if path.is_file()
        and (path.suffix in {".py", ".pyi"} or path.name == "py.typed")
    )
    if not files:
        raise SystemExit("Missing Python sources for package verification.")
    return files


def _verify_wheel_payload(wheel_path: Path, source_root: Path) -> None:
    python_root = source_root / "crates/dokime-python/python"
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
            and (name.endswith((".py", ".pyi")) or name.endswith("/py.typed"))
        }
        if actual != expected.keys():
            raise SystemExit(
                f"Wheel source inventory mismatch in {wheel_path.name}: "
                f"missing={sorted(expected.keys() - actual)}, "
                f"unexpected={sorted(actual - expected.keys())}."
            )
        native = [
            name
            for name in names
            if name.startswith("dokime/_native.") and name.endswith((".so", ".pyd"))
        ]
        if len(native) != 1 or wheel.getinfo(native[0]).file_size == 0:
            raise SystemExit(
                f"Expected one nonempty native extension in {wheel_path.name}."
            )

        for name, source in expected.items():
            if wheel.read(name) != source.read_bytes():
                raise SystemExit(f"Stale wheel source in {wheel_path.name}: {name}.")


def _verify_sdist_build_inputs(sdist_path: Path, source_root: Path) -> None:
    """Verify Python payloads and crate-local inputs consumed by Rust build scripts."""
    inputs = sorted(
        (source_root / "crates/dokime-protocol/proto/dokime/v1").glob("*.proto")
    )
    if not inputs:
        raise SystemExit("Missing source protobuf definitions for sdist verification.")
    inputs.append(source_root / "crates/dokime-app/dist-report/report.html")
    bundle = source_root / "branding"
    inputs.append(bundle / "brand.toml")
    for folder in ("theme", "fonts", "logos"):
        directory = bundle / folder
        if not directory.is_dir() or directory.is_symlink():
            raise SystemExit(f"Missing source branding directory: {folder}.")
        inputs.extend(sorted(path for path in directory.iterdir() if path.is_file()))
    python_root = source_root / "crates/dokime-python"
    inputs.extend(_python_inputs(source_root))
    prefix = sdist_path.name.removesuffix(".tar.gz")

    with tarfile.open(sdist_path, "r:gz") as sdist:
        members = {member.name: member for member in sdist.getmembers()}
        for source in inputs:
            relative = source.relative_to(
                python_root if source.is_relative_to(python_root) else source_root
            ).as_posix()
            if not source.is_file():
                raise SystemExit(f"Missing source build input: {relative}.")
            member = members.get(f"{prefix}/{relative}")
            if member is None or not member.isfile():
                raise SystemExit(
                    f"Missing source distribution build input in {sdist_path.name}: "
                    f"{relative}."
                )
            archived = sdist.extractfile(member)
            if archived is None or archived.read() != source.read_bytes():
                raise SystemExit(
                    f"Stale source distribution build input in {sdist_path.name}: "
                    f"{relative}."
                )


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
