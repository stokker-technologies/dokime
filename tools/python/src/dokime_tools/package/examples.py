# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Stage And Package The Canonical Remote Examples"""

# === Imports === #

# Standard Library Imports
import argparse
import hashlib
import stat
import tomllib
from pathlib import Path

# Third Party Imports
from packaging.specifiers import SpecifierSet
from packaging.utils import parse_wheel_filename

# Local Imports
from dokime_tools import temporary
from dokime_tools.build import paths
from dokime_tools.package.run import assemble, definitions
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Only Public Canonical Inputs Travel; Local Keys, Dotenv Files And Evidence Never Do.
FILES = (
    "LICENSE-APACHE",
    ".env.example",
    "README.md",
    "cli/README.md",
    "cli/bootstrap.ps1",
    "cli/bootstrap.bat",
    "cli/control-device.sh",
    "cli/control-test.sh",
    "cli/control-prompt.sh",
    "cli/control-device.ps1",
    "cli/control-test.ps1",
    "cli/control-prompt.ps1",
    "cli/control-device.bat",
    "cli/control-test.bat",
    "cli/control-prompt.bat",
    "python/control-device.py",
    "python/control-test.py",
    "python/control-prompt.py",
    "python/README.md",
    "python/pyproject.toml",
    "python/uv.lock",
)

# === Types === #

# === Classes === #

# === Public Functions === #


def stage(root: Path, destination: Path, *, wheel: Path) -> list[Path]:
    """Stage The Tutorial And Its Exact Locked Candidate Wheel."""
    package_root = root / "packages/dokime-client"
    source_root = package_root / "examples/remote"
    sources = [package_root / relative if relative == "LICENSE-APACHE" else source_root / relative for relative in FILES]
    for source in sources:
        _regular(source)
    _environment(source_root / "python", wheel)
    if wheel.resolve().is_relative_to(destination.resolve()):
        raise ValueError("the candidate wheel must be outside the staged examples")

    clear_output(destination)
    staged = []
    for relative, source in zip(FILES, sources, strict=True):
        output = destination / relative
        stage_file(source, output)
        staged.append(output)
    candidate = destination / "wheels" / wheel.name
    stage_file(wheel, candidate)
    _environment(destination / "python", candidate)
    staged.append(candidate)
    return staged


def package(root: Path, profile: str) -> None:
    """Package Built Distributions And The Declared Tutorial Without Reading Source."""
    source = paths.output(root, "build", profile, "any") / "python/examples/remote"
    if source.is_symlink() or not source.is_dir():
        raise ValueError("packaging requires the built remote examples; run `mise run build` first")
    for relative in FILES:
        _regular(source / relative)
    wheel = _candidate(source / "python")
    _environment(source / "python", wheel)
    built = paths.output(root, "build", profile, "any") / "python"
    wheels = sorted(built.glob("dokime_client-*.whl"))
    if len(wheels) != 1:
        raise ValueError("packaging requires exactly one matching built client wheel")
    _environment(source / "python", wheels[0])
    distributions = [wheels[0]]
    if profile == "release":
        version = parse_wheel_filename(wheels[0].name)[1]
        distribution = built / f"dokime_client-{version}.tar.gz"
        _regular(distribution)
        distributions.append(distribution)

    destination = paths.output(root, "package", profile, "any") / "python"
    names = [path.name for path in distributions]
    names += [item["output"] for item in definitions(root, profile, "any") if item["language"] == "python"]
    with temporary.directory() as scratch:
        assembled = scratch / "assembled"
        assembled.mkdir()
        assemble(root, "python", profile, "any", assembled)

        # Select Public Distributions And Declared Bundles
        selected = scratch / "selected"
        for name in names:
            source = assembled / name
            _regular(source)
            stage_file(source, selected / name)
        paths.replace(root, selected, destination, origin=scratch)
    print(f"Packaged {destination.relative_to(root).as_posix()}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    parser.add_argument("--target", choices=("any",), default="any")
    options = parser.parse_args()
    package(Path.cwd(), options.profile)


# === Private Functions === #


def _regular(path: Path) -> None:
    if any(parent.is_symlink() for parent in path.parents) or not stat.S_ISREG(path.lstat().st_mode):
        raise ValueError(f"example must be a regular file with an unlinked parent: {path}")


def _candidate(project: Path) -> Path:
    _regular(project / "pyproject.toml")
    manifest = tomllib.loads((project / "pyproject.toml").read_text(encoding="utf-8"))
    source = manifest.get("tool", {}).get("uv", {}).get("sources", {}).get("dokime-client", {})
    path = source.get("path")
    if not isinstance(path, str) or path != f"../wheels/{Path(path).name}" or "\\" in path:
        raise ValueError("the tutorial requires one portable bundled Dokime wheel source")
    return project.parent / "wheels" / Path(path).name


def _environment(project: Path, wheel: Path) -> None:
    _regular(wheel)
    _regular(project / "uv.lock")
    candidate = _candidate(project)
    if candidate.name != wheel.name:
        raise ValueError("the candidate wheel differs from the tutorial source")
    name, version, _build, _tags = parse_wheel_filename(wheel.name)
    if name != "dokime-client":
        raise ValueError("the candidate must be a Dokime wheel")

    manifest = tomllib.loads((project / "pyproject.toml").read_text(encoding="utf-8"))
    declaration = manifest.get("project", {})
    if declaration.get("dependencies") != [f"dokime-client=={version}"]:
        raise ValueError("the tutorial must pin its bundled Dokime version")
    lock = tomllib.loads((project / "uv.lock").read_text(encoding="utf-8"))
    if SpecifierSet(declaration.get("requires-python", "")) != SpecifierSet(lock.get("requires-python", "")):
        raise ValueError("the tutorial Python requirement differs from its lock")
    packages = [package for package in lock.get("package", []) if package.get("name") == "dokime-client"]
    if len(packages) != 1:
        raise ValueError("the tutorial lock must name exactly one Dokime candidate")
    package = packages[0]
    expected_source = {"path": f"../wheels/{wheel.name}"}
    if package.get("version") != str(version) or package.get("source") != expected_source:
        raise ValueError("the locked Dokime version or source differs from the candidate")
    digest = hashlib.sha256(wheel.read_bytes()).hexdigest()
    expected_wheels = [{"filename": wheel.name, "hash": f"sha256:{digest}"}]
    if package.get("wheels") != expected_wheels:
        raise ValueError("the candidate wheel bytes differ from the tutorial lock; regenerate it from this candidate")


# === Entry Point === #

if __name__ == "__main__":
    main()
