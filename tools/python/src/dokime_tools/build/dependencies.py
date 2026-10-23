# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Capture Offline Inputs For The Generated Project Acceptance Story"""

# === Imports === #

# Standard Library Imports
import hashlib
import subprocess
import sys
import tomllib
import urllib.parse
import urllib.request
from pathlib import Path

# Third Party Imports
from packaging.requirements import Requirement
from packaging.tags import sys_tags
from packaging.utils import parse_wheel_filename

# Local Imports
from dokime_tools import temporary
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def cargo(root: Path, directory: Path) -> list[Path]:
    """Export The Locked Source Closure Without Compiling Another Target."""
    root = root.resolve()
    vendor = directory / "vendor"
    config = directory / "cargo-config.toml"
    clear_output(vendor)
    clear_output(config)
    with config.open("w", encoding="utf-8") as output:
        subprocess.run(
            ["cargo", "vendor", "--locked", "--versioned-dirs", str(vendor)],
            cwd=root,
            stdout=output,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    # The Configuration Names The Closure; Its Thousands Of Vendored Files Stay Out Of The Stamp.
    return [config]


def wheelhouse(root: Path, directory: Path) -> list[Path]:
    """Export The Starter's Actual Locked Backend And Tool Wheels."""
    root = root.resolve()
    tools = directory / "starter-tools"
    templates = root / "crates/cargo-dokime/templates"
    clear_output(tools)
    for relative in ("pyproject.toml", "uv.lock", "tools/python/pyproject.toml"):
        stage_file(templates / f"{relative}.tmpl", tools / relative)
    requirements = directory / "starter-requirements.txt"
    wheels = directory / "wheels"
    clear_output(requirements)
    clear_output(wheels)
    with requirements.open("w", encoding="utf-8") as output:
        subprocess.run(
            ["uv", "export", "--locked", "--all-groups", "--no-emit-project", "--project", str(tools / "tools/python")],
            cwd=root,
            stdout=output,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    # uv Owns Resolution; Fetch Only Its Locked, Hash-Verified Native Wheels.
    locked = tomllib.loads((tools / "uv.lock").read_text(encoding="utf-8"))["package"]
    selected = requirements.read_text(encoding="utf-8").replace("\\\n", " ")
    for line in selected.splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        requirement = Requirement(line.split("--hash=", 1)[0].strip())
        if requirement.marker is not None and not requirement.marker.evaluate():
            continue
        packages = [package for package in locked if package["name"] == requirement.name and package["version"] in requirement.specifier]
        if len(packages) != 1:
            raise ValueError(f"expected one locked starter dependency: {requirement}")
        entry = _compatible(packages[0].get("wheels", []))
        _fetch(entry["url"], entry["hash"], wheels)

    # Editable Backend Support Is Explicitly Pinned Separately From uv's Runtime Lock.
    support = (root / "tools/python/editables.txt").read_text(encoding="utf-8").strip()
    declaration, digest = support.split(" --hash=", 1)
    requirement = Requirement(declaration)
    if not requirement.url:
        raise ValueError("editable backend support requires one pinned wheel URL")
    _compatible([{"url": requirement.url, "hash": digest}])
    _fetch(requirement.url, digest, wheels)
    return [requirements, *sorted(wheels.glob("*.whl"))]


# === Private Functions === #


def _compatible(wheels: list[dict]) -> dict:
    tags = {tag: index for index, tag in enumerate(sys_tags())}
    choices = []
    for wheel in wheels:
        name = Path(urllib.parse.urlsplit(wheel["url"]).path).name
        _name, _version, _build, supported = parse_wheel_filename(name)
        ranks = [tags[tag] for tag in supported if tag in tags]
        if ranks:
            choices.append((min(ranks), name, wheel))
    if not choices:
        raise ValueError("the starter dependency has no compatible locked binary wheel")
    return min(choices, key=lambda item: (item[0], item[1]))[2]


def _fetch(url: str, expected: str, output: Path) -> None:
    parsed = urllib.parse.urlsplit(url)
    name = Path(parsed.path).name
    if parsed.scheme != "https" or not name.endswith(".whl") or not expected.startswith("sha256:"):
        raise ValueError("starter wheels require HTTPS and a locked SHA-256")
    digest = hashlib.sha256()
    with temporary.directory() as scratch:
        candidate = scratch / name
        with urllib.request.urlopen(url, timeout=120) as response, candidate.open("wb") as written:
            while block := response.read(1024 * 1024):
                digest.update(block)
                written.write(block)
        if f"sha256:{digest.hexdigest()}" != expected:
            raise ValueError(f"starter wheel hash differs from its lock: {name}")
        stage_file(candidate, output / name)


# === Entry Point === #
