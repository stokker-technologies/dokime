# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"Own Package Definition Operations."

# === Imports === #

# Standard Library Imports
import re
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..build import paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def definitions(directory: Path, profile: str, target: str) -> list[dict]:
    """Resolve Declared Products Without Looking At Native Compiler Directories."""
    config = policy.config(directory)
    version = config.get("version", {}).get("value")
    result = []
    for item in config.get("package", {}).get("executables", []):
        triple = item["targets"].get(target)
        if triple is None:
            continue
        if not version or not re.fullmatch(r"[a-zA-Z0-9_.-]+", version):
            raise policy.ToolError("Package Product: executable packages require version.value")
        extension = ".exe" if "-windows-" in triple else ""
        archive = ".zip" if extension else ".tar.gz"
        name = f"{item['name']}-{version}-{triple}{archive}"
        result.append({**item, "language": "rust", "output": name, "triple": triple, "binary": item["command"] + extension})
    for item in config.get("package", {}).get("bundles", []):
        if profile in item["profiles"] and target in item["targets"]:
            fields = [item["name"], *(entry[key] for entry in item["inputs"] for key in ("path", "destination"))]
            if any("{version}" in field for field in fields) and (
                not isinstance(version, str) or re.fullmatch(r"[A-Za-z0-9_.-]+", version) is None or version in {".", ".."}
            ):
                raise policy.ToolError("Package Product: versioned bundles require a safe version.value")
            result.append({**item, "output": item["name"].format(version=version, target=target)})
    names = [item["output"].casefold() for item in result]
    if len(names) != len(set(names)):
        raise policy.ToolError("Package Product: package output names collide")
    for item in result:
        name = item["output"]
        if re.fullmatch(r"[A-Za-z0-9_.-]+", name) is None or name in {".", "..", "SHA256SUMS", "release.json", "platform.json"}:
            raise policy.ToolError("Package Product: unsafe or reserved package output name")
    return result


def packaged(directory: Path, profile: str, target: str) -> list[dict]:
    """Only Explicitly Shipped Tools Enter The Release; Bundles Need release.assets."""
    root = paths.output(directory, "package", profile, target)
    result = []
    for result_item in definitions(directory, profile, target):
        if "triple" not in result_item:
            continue
        if not result_item["release"]:
            continue
        result.append(
            {
                "target": target,
                "path": (root / result_item["language"] / result_item["output"]).relative_to(directory).as_posix(),
                "name": result_item["output"],
            }
        )

    return result


def regular(root: Path, relative: str) -> Path:
    parts = relative.split("/")
    if not relative or any(part in {"", ".", ".."} for part in parts) or "\\" in relative or ":" in relative:
        raise policy.ToolError(f"Package Product: unsafe package input: {relative}")
    source = policy.inside(root, relative)
    if not source.is_file() or any(path.is_symlink() for path in (source, *source.parents) if path.is_relative_to(root)):
        raise policy.ToolError(f"Package Product: missing or linked package input: {relative}")
    return source


# === Private Functions === #

# === Entry Point === #
