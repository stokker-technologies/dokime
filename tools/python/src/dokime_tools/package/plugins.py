# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Package Product Plugins From Their Existing Compiler Outputs"""

# === Imports === #

# Standard Library Imports
import json
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import paths
from dokime_tools.package.staging import clear_output
from dokime_tools.release import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

FIXTURES = ("plugin-host", "plugin-obc")

# === Types === #

# === Classes === #

# === Public Functions === #


def prepare(root: Path, messages: Path, profile: str, *, tool: Path, output: Path, packages: tuple[str, ...] | None = None) -> list[Path]:
    """Collect The Product Build's Plugin Executables Without Another Compiler."""
    root = root.resolve()
    declarations = policy.config(root).get("release", {}).get("plugins", [])
    target = paths.target()
    if packages is None:
        packages = tuple(item["package"] for item in declarations if target in item["targets"])

    metadata = messages.with_name("metadata.json")
    with metadata.open("w", encoding="utf-8") as written:
        subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            cwd=root,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    clear_output(output)
    output.mkdir(parents=True)
    staged: list[Path] = []
    for package in sorted(set(packages)):
        build_json = output / f"{package}.build.json"
        collector = [str(tool), "dokime", "sidecar", "build", "--from-cargo-json", str(messages), "--metadata-json", str(metadata)]
        collector += ["--profile", profile, "--package", package, "--json"]
        with build_json.open("w", encoding="utf-8") as written:
            subprocess.run(
                collector,
                cwd=root,
                stdout=written,
                check=True,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                stdin=sys.stdin,
                stderr=sys.stderr,
            )
        packager = [str(tool), "dokime", "sidecar", "package", "--build-json", str(build_json), "--output", str(output), "--json"]
        packaged = _capture(packager, cwd=root)
        for item in json.loads(packaged):
            path = Path(item["path"])
            path = path if path.is_absolute() else root / path
            declaration = next((item for item in declarations if item["package"] == package and target in item["targets"]), None)
            if declaration is not None and "path" in declaration:
                selected = path.with_name(Path(declaration["path"]).name)
                if selected != path:
                    path.rename(selected)
                path = selected
            staged.append(path)
    return staged


def archive(root: Path, build: Path, output: Path, version: str, target: str) -> Path | None:
    """Archive Exactly The Declared Shipping Sidecars From Existing Build Bytes."""
    members = shipping(root, build, target)
    if not members:
        return None

    extension = ".zip" if "windows" in target else ".tar.gz"
    destination = output / f"dokime-sidecars-{version}-{target}{extension}"
    output.mkdir(parents=True, exist_ok=True)
    if extension == ".zip":
        with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for name, source in sorted(members.items()):
                archive.write(source, name)
    else:
        with tarfile.open(destination, "w:gz") as archive:
            for name, source in sorted(members.items()):
                archive.add(source, arcname=name, recursive=False)
    return destination


def shipping(root: Path, build: Path, target: str) -> dict[str, Path]:
    """Resolve Unversioned Bundle Names From Product Declarations."""
    declarations = policy.config(root).get("release", {}).get("plugins", [])
    selected = [item for item in declarations if target in item["targets"].values() and item.get("kind", "sidecar") == "sidecar"]
    members = {}
    for item in selected:
        name = Path(item["path"]).name
        if name.casefold() in {old.casefold() for old in members} or Path(name).suffix != ".dokime":
            raise ValueError("shipping sidecars require distinct .dokime member names")
        source = build / "plugins" / name
        if source.is_symlink() or not source.is_file():
            raise ValueError("shipping sidecar must be an existing regular bundle")
        members[name] = source
    return members


# === Private Functions === #


def _capture(command: list[str], *, cwd: Path | None = None) -> str:
    try:
        result = subprocess.run(
            command,
            cwd=cwd,
            capture_output=True,
            text=True,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
        )
    except subprocess.CalledProcessError as error:
        print(f"Command failed with exit code {error.returncode}: {subprocess.list2cmdline(command)}", file=sys.stderr)
        outputs = filter(None, (error.stderr, error.stdout))
        diagnostics = "\n".join(outputs)
        print(diagnostics, file=sys.stderr)
        raise
    return result.stdout


# === Entry Point === #
