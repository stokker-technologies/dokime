# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Policy and Native Execution for Managed Repository Tools"

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import subprocess
import sys
import tomllib
from collections.abc import Callable, Generator, Sequence
from contextlib import contextmanager
from pathlib import Path
from typing import Any

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ..process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #


LIBRARY_TARGETS = frozenset({"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"})

# === Types === #

# === Classes === #


class ToolError(Exception):
    """Invalid policy or a failed native operation."""


# === Public Functions === #


def root() -> Path:
    current = Path.cwd().resolve()
    for candidate in (current, *current.parents):
        if (candidate / "tools/project.toml").is_file():
            return candidate
    raise ToolError("Configure Project Tooling: no project policy file found")


def config(directory: Path) -> dict[str, Any]:
    return tomllib.loads((directory / "tools/project.toml").read_text(encoding="utf-8"))


def policy(directory: Path, name: str) -> dict[str, Any]:
    value = config(directory).get(name, {})
    if not isinstance(value, dict):
        raise ToolError(f"Configure Project Tooling: [{name}] must be a table")
    return value


def publishable(directory: Path, metadata: dict) -> dict:
    """Select Library Workspace Members And Refuse Publishable Artifact-Only Packages."""
    settings = config(directory)
    plugin_project = settings.get("project", {}).get("profile") in {"plugin", "sidecar", "guest"}
    plugins = {plugin["package"] for plugin in settings.get("release", {}).get("plugins", [])}
    members = set(metadata["workspace_members"])
    packages = {}
    for package in metadata["packages"]:
        if package["id"] not in members:
            continue
        if package.get("publish") == []:
            continue
        name = package["name"]
        manifest = Path(package["manifest_path"]).resolve()
        relative = manifest.relative_to(directory.resolve()) if manifest.is_relative_to(directory.resolve()) else manifest
        plugin = plugin_project or name in plugins or (not relative.is_absolute() and "plugins" in relative.parts[:-1])
        library = any(LIBRARY_TARGETS.intersection(target["kind"]) for target in package["targets"])
        if plugin or not library:
            role = "plugin" if plugin else "non-library"
            raise ToolError(
                f"Configure Project Tooling: {relative.as_posix()}: {name} is a {role} package; packages without library targets and plugin packages never publish to registries; declare publish = false"
            )
        packages[name] = package
    return packages


def profile(default: str = "release") -> str:
    """The Package Profile Passed By The Rendered Job Or A Local Invocation."""
    selected = os.environ.get("RELEASE_PROFILE") or default
    if selected not in ("dev", "release"):
        raise ToolError("Configure Project Tooling: RELEASE_PROFILE must be dev or release")
    return selected


def inside(directory: Path, relative: str) -> Path:
    candidate = directory / relative
    if Path(relative).is_absolute() or not candidate.resolve().is_relative_to(directory.resolve()):
        raise ToolError(f"Configure Project Tooling: path must stay inside the repository: {relative}")
    return candidate


def command(value: object, name: str) -> list[str]:
    if not isinstance(value, list) or not value or any(not isinstance(part, str) or not part for part in value):
        raise ToolError(f"Configure Project Tooling: {name} must be a nonempty argument array")
    return value


def execute(directory: Path, arguments: Sequence[str], *, capture: bool = False) -> str:
    result = subprocess.run(
        list(arguments),
        creationflags=spawn_flags(),
        cwd=directory,
        check=False,
        text=True,
        encoding="utf-8",
        stdin=sys.stdin,
        stdout=subprocess.PIPE if capture else sys.stdout,
        stderr=subprocess.PIPE if capture else sys.stderr,
    )
    if result.returncode:
        raise ToolError(f"Configure Project Tooling: {arguments[0]} failed with exit code {result.returncode}: {result.stderr or ''}")
    return result.stdout or ""


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.is_symlink():
        raise ToolError(f"Configure Project Tooling: refusing linked output: {path}")
    temporary = path.with_suffix(path.suffix + ".tmp")
    if temporary.is_symlink():
        raise ToolError(f"Configure Project Tooling: refusing linked staging file: {temporary}")
    temporary.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


@contextmanager
def credential_free(settings: dict, *, profile: str = "release") -> Generator[None]:
    """Temporarily Remove Publishing Credentials And Select The Requested Package Profile."""
    declared = set()
    for declared_lane in settings.get("package_signing", []):
        for declared_name in declared_lane.get("secrets", []):
            declared.add(declared_name)

    names = set()
    for names_name in os.environ:
        if not (names_name.endswith(("_TOKEN", "_PASSWORD", "_SECRET")) or names_name in declared):
            continue
        names.add(names_name)

    names.add("ACTIONS_ID_TOKEN_REQUEST_URL")
    previous = {}
    for previous_name in names:
        if previous_name not in os.environ:
            continue
        previous[previous_name] = os.environ.pop(previous_name)

    previous_profile = os.environ.get("RELEASE_PROFILE")
    os.environ["RELEASE_PROFILE"] = profile
    try:
        yield
    finally:
        os.environ.update(previous)
        if previous_profile is None:
            os.environ.pop("RELEASE_PROFILE", None)
        else:
            os.environ["RELEASE_PROFILE"] = previous_profile


def main(
    name: str,
    verbs: Sequence[str],
    operation: Callable[[Path, str], None],
    arguments: Sequence[str] | None = None,
) -> int:
    parser = argparse.ArgumentParser(description=f"Managed {name.title()} Tooling")
    parser.add_argument("verb", choices=verbs)
    options = parser.parse_args(arguments)
    try:
        operation(root(), options.verb)
    except (ToolError, OSError, ValueError, KeyError, TypeError) as error:
        print(f"{name}: {error}", file=sys.stderr)
        return 1
    return 0


# === Private Functions === #

# === Entry Point === #
