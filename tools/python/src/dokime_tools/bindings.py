# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prepare Shared Harmos Python Contracts From Selected Cargo Outputs"""

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

# Third Party Imports

# Local Imports
# isort: split
from .ci.bootstrap import prepare_harmos
from .contracts import locate, restored

# Type Checking Imports

# === Public Exports === #

# === Constants === #
ROOT = Path(__file__).resolve().parents[4]
_FILES = {
    "__init__.py",
    "__init__.pyi",
    "operation.py",
    "dynamic.py",
    "py.typed",
    "contracts.json",
    "descriptors.bin",
}

# === Types === #

# === Classes === #

# === Public Functions === #


def generate(
    messages: Path | None,
    root: Path = ROOT,
    *,
    check: bool = False,
    export: Path | None = None,
) -> None:
    prepare_harmos(root, allow_install=False)
    if export is None:
        if messages is None:
            raise ValueError("a selected contract build is required")
        export = locate(messages)
    project = tomllib.loads((root / "packages/dokime/pyproject.toml").read_text())[
        "project"
    ]
    destination = root / "packages/dokime/src/dokime"
    with tempfile.TemporaryDirectory(prefix="dokime-bindings-") as directory:
        package = Path(directory) / "package"
        subprocess.run(
            [
                "cargo",
                "harmos",
                "bindings",
                "python",
                "--export",
                str(export),
                "--output",
                str(package),
                "--distribution",
                project["name"],
                "--module",
                "dokime",
                "--package-version",
                project["version"],
                "--description",
                project["description"],
                "--license",
                project["license"],
            ],
            cwd=root,
            check=True,
        )
        source = package / "src/dokime"
        generated = {}
        for path in source.rglob("*"):
            if not path.is_file():
                continue
            relative = path.relative_to(source)
            if path.is_symlink() or (
                relative.as_posix() not in _FILES and relative.parts[0] != "_wire"
            ):
                raise ValueError(
                    f"exporter returned an unexpected SDK path: {relative}"
                )
            generated[relative] = path.read_bytes()
        for name in ("__init__.py", "__init__.pyi"):
            current = destination / name
            marker = b"# Product Client Exports\n"
            if current.is_file():
                contents = current.read_bytes()
                if marker in contents:
                    suffix = marker + contents.split(marker, 1)[1]
                    generated[Path(name)] = (
                        generated[Path(name)].rstrip() + b"\n\n" + suffix
                    )
        if not {Path(name) for name in _FILES | {"_wire/_pool.py"}}.issubset(generated):
            raise ValueError("shared exporter omitted required contract assets")
        _normalize(generated, source, root)
        for relative in generated:
            target = destination / relative
            ancestors = [target, *target.parents]
            if any(
                path.is_symlink()
                for path in ancestors
                if path == destination or destination in path.parents
            ):
                raise ValueError(
                    f"generated destination contains a symlink: {relative}"
                )
            if target.exists() and not target.is_file():
                raise ValueError(f"generated destination is not a file: {relative}")
        obsolete = [
            path
            for path in (destination / "_wire").rglob("*")
            if path.is_file() and path.relative_to(destination) not in generated
        ]
        if check:
            stale = [
                relative
                for relative, data in generated.items()
                if not (destination / relative).is_file()
                or (destination / relative).read_bytes() != data
            ]
            if stale or obsolete:
                raise ValueError(
                    "Python contracts are stale; regenerate from the selected Cargo build"
                )
        else:
            for relative, data in generated.items():
                target = destination / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                if not target.is_file() or target.read_bytes() != data:
                    target.write_bytes(data)
            for path in obsolete:
                path.unlink()
    print("Python contracts match the selected Cargo build.")


def _normalize(generated: dict[Path, bytes], source: Path, root: Path) -> None:
    selected = sorted(
        path
        for path in generated
        if path.as_posix() in _FILES and path.suffix in {".py", ".pyi"}
    )
    for relative in selected:
        (source / relative).write_bytes(generated[relative])
    paths = [str(source / relative) for relative in selected]
    configuration = str(root / "packages/dokime/pyproject.toml")
    subprocess.run(
        [
            sys.executable,
            "-m",
            "ruff",
            "check",
            "--select",
            "I",
            "--fix",
            "--config",
            configuration,
            *paths,
        ],
        cwd=root,
        check=True,
    )
    subprocess.run(
        [sys.executable, "-m", "ruff", "format", "--config", configuration, *paths],
        cwd=root,
        check=True,
    )
    for relative in selected:
        generated[relative] = (source / relative).read_bytes()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--language", choices=("python", "typescript"), default="python"
    )
    parser.add_argument(
        "--build-json", type=Path, default=os.environ.get("DOKIME_CONTRACT_BUILD_JSON")
    )
    parser.add_argument("--check", action="store_true")
    parser.add_argument(
        "--export", type=Path, default=os.environ.get("DOKIME_CONTRACT_EXPORT")
    )
    args = parser.parse_args()
    export = None
    if args.export is not None:
        revision = (
            os.environ.get("GITHUB_SHA")
            or subprocess.check_output(
                ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
            ).strip()
        )
        export = restored(args.export, revision)
    if args.build_json is None and export is None:
        parser.error(
            "--build-json or DOKIME_CONTRACT_BUILD_JSON must select a successful Cargo build"
        )
    if args.language == "typescript":
        if export is None:
            if args.build_json is None:
                parser.error("a selected Cargo build is required")
            export = locate(args.build_json)
        command = [
            "node",
            str(ROOT / "crates/dokime-gui/tools/bindings.mjs"),
            str(export),
        ]
        if args.check:
            command.append("--check")
        subprocess.run(command, cwd=ROOT, check=True)
    else:
        generate(args.build_json, check=args.check, export=export)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
