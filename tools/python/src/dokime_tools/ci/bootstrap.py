# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare Native CI Platform Prerequisites"""

# === Imports === #

# Standard Library Imports
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import tomllib
import urllib.parse
from collections.abc import Sequence
from pathlib import Path
from typing import Any

# Third Party Imports
# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Debian Build And Installer Prerequisites
GUI_MARKER_PACKAGE = "libwebkit2gtk-4.1-dev"
GUI_PACKAGES = (
    "git",
    "xz-utils",
    "unzip",
    "build-essential",
    "pkg-config",
    "libssl-dev",
    "patchelf",
    "rpm",
    "file",
    GUI_MARKER_PACKAGE,
    "libgtk-3-dev",
    "libsoup-3.0-dev",
    "libjavascriptcoregtk-4.1-dev",
    "librsvg2-dev",
    # Native Media Capture (dokime-app `media-native`): ALSA for audio input,
    # V4L2 kernel headers for cameras, and libclang for the bindings V4L2
    # generates at build time. Screenshots use the desktop portal over D-Bus.
    "libasound2-dev",
    "linux-libc-dev",
    "libclang-dev",
)
HARMOS_REPOSITORY = "https://github.com/stokker-technologies/harmos.git"
GUI_RUNTIME_PACKAGES = (
    "libwebkit2gtk-4.1-0",
    "libgtk-3-0t64",
    "libsoup-3.0-0",
    "libjavascriptcoregtk-4.1-0",
    "librsvg2-2",
    "libasound2t64",
    "libssl3t64",
)
DESKTOP_PACKAGES = ("webkit2gtk-driver", "xvfb", "desktop-file-utils", "xdg-utils")

# === Types === #

# === Classes === #

# === Public Functions === #


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parse_arguments(argv)
    if arguments.harmos_key:
        print(harmos_pin(Path.cwd())["rev"])
        return 0
    if arguments.harmos_tools:
        prepare_harmos(
            Path.cwd(),
            allow_install=os.environ.get("DOKIME_CI_ROLE") in {None, "contracts"},
            cargo_config=arguments.cargo_config,
        )
        return 0
    if arguments.gui_packages or arguments.gui_runtime:
        install_gui_packages(runtime=arguments.gui_runtime, desktop=arguments.desktop)
    if arguments.desktop:
        if sys.platform != "linux":
            raise ValueError("the declared WebKit desktop bootstrap requires Linux")
        if not os.environ.get("DISPLAY"):
            github_environment = os.environ.get("GITHUB_ENV")
            if not github_environment:
                raise ValueError("the desktop job needs DISPLAY or GitHub's environment file")
            subprocess.Popen(
                ["Xvfb", ":99", "-screen", "0", "1920x1080x24"],
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )
            with Path(github_environment).open("a", encoding="utf-8") as written:
                written.write("DISPLAY=:99\n")
    _step("bootstrap complete")
    return 0


def parse_arguments(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="bootstrap.py",
        description="Prepare explicitly selected native prerequisites or the pinned Harmos tool.",
    )
    gui = parser.add_mutually_exclusive_group()
    gui.add_argument(
        "--gui-packages",
        action="store_true",
        help="install the Debian GUI development packages (Linux jobs only).",
    )
    gui.add_argument(
        "--gui-runtime",
        action="store_true",
        help="install only Ubuntu GUI runtime libraries for prebuilt artifacts.",
    )
    parser.add_argument("--harmos-key", action="store_true")
    parser.add_argument("--harmos-tools", action="store_true")
    parser.add_argument("--cargo-config", type=Path)
    parser.add_argument("--desktop", action="store_true")
    return parser.parse_args(argv)


def harmos_pin(root: Path) -> dict[str, str]:
    """Resolve the one Harmos dependency to its exact public Git revision.

    A `rev` source is exact as declared; a `branch` source resolves to the
    revision Cargo.lock locked it to, so the prepared tool stays exact.
    """
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    dependencies = manifest["workspace"]["dependencies"]
    if any(name in dependencies for name in ("harmos-contract", "harmos-signal")):
        raise ValueError("Dokime depends on the harmos crate alone")
    dependency = dependencies["harmos"]
    url, revision, branch = (
        dependency.get("git"),
        dependency.get("rev"),
        dependency.get("branch"),
    )
    if url != HARMOS_REPOSITORY:
        raise ValueError("Harmos tools require the public Harmos Git source")
    if branch is None and _is_revision(revision):
        return {"git": url, "rev": revision, "package": "cargo-harmos"}
    if revision is None and isinstance(branch, str) and branch:
        # Cargo Percent-Encodes The Branch In Its Lockfile Source
        locked = _locked_revision(root, f"git+{url}?branch={urllib.parse.quote(branch, safe='')}#")
        return {"git": url, "branch": branch, "rev": locked, "package": "cargo-harmos"}
    raise ValueError("Harmos tools require the exact declared public Git revision or a branch")


def prepare_harmos(
    root: Path,
    *,
    allow_install: bool,
    cargo_config: Path | None = None,
) -> Path:
    local_source = None
    if cargo_config is not None:
        if os.environ.get("CI") or os.environ.get("GITHUB_ACTIONS") or os.environ.get("DOKIME_CI_ROLE"):
            raise ValueError("CI refuses local Harmos tool overrides")
        config = cargo_config.resolve(strict=True)
        patch = tomllib.loads(config.read_text(encoding="utf-8"))["patch"][HARMOS_REPOSITORY]["harmos"]
        crate = Path(patch["path"])
        if not crate.is_absolute():
            raise ValueError("local Harmos tool configuration requires an absolute patch path")
        crate = crate.resolve(strict=True)
        package = tomllib.loads((crate / "Cargo.toml").read_text(encoding="utf-8"))["package"]
        if package["name"] != "harmos":
            raise ValueError("local Harmos patch does not identify the harmos package")
        local_source = crate.parent.parent
        tool = local_source / "crates/cargo-harmos"
        if tomllib.loads((tool / "Cargo.toml").read_text(encoding="utf-8"))["package"]["name"] != "cargo-harmos":
            raise ValueError("local Harmos checkout has no canonical tool package")
        source_identity = {
            "local_source": str(local_source),
            "source": _override_source(local_source),
            "cargo_config_sha256": hashlib.sha256(config.read_bytes()).hexdigest(),
            "package": "cargo-harmos",
        }
    else:
        source_identity = harmos_pin(root)
    identity: dict[str, Any] = {
        **source_identity,
        "profile": "dev",
        "platform": platform.system(),
        "architecture": platform.machine(),
        "rustc": subprocess.check_output(
            ["mise", "exec", "--", "rustc", "-vV"],
            text=True,
            encoding="utf-8",
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        ).strip(),
        "environment": {
            name: os.environ.get(name)
            for name in sorted(
                {
                    "RUSTC",
                    "RUSTC_WRAPPER",
                    "RUSTC_WORKSPACE_WRAPPER",
                    "RUSTFLAGS",
                    "CARGO_ENCODED_RUSTFLAGS",
                    "CARGO_BUILD_TARGET",
                    "CARGO_BUILD_RUSTFLAGS",
                }
                | {name for name in os.environ if name.startswith("CARGO_PROFILE_")}
            )
        },
    }
    destination = root / ".cache/harmos-tool"
    binary = destination / "bin" / ("cargo-harmos.exe" if os.name == "nt" else "cargo-harmos")
    receipt = destination / "receipt.json"
    available = False
    if binary.is_file() and not binary.is_symlink() and receipt.is_file() and not receipt.is_symlink():
        try:
            value = json.loads(receipt.read_text(encoding="utf-8"))
            available = value.get("identity") == identity and value.get("sha256") == hashlib.sha256(binary.read_bytes()).hexdigest()
        except (OSError, ValueError):
            available = False
    if not available:
        if not allow_install:
            raise ValueError("exact prepared Harmos tool artifact is missing or incompatible; consumers never rebuild it")
        destination.parent.mkdir(parents=True, exist_ok=True)
        source_arguments = (
            ["--path", str(local_source / "crates/cargo-harmos")]
            if local_source is not None
            else ["--git", identity["git"], "--rev", identity["rev"]]
        )
        with tempfile.TemporaryDirectory(prefix="harmos-tool-") as temporary:
            staged = Path(temporary)
            _run(
                [
                    "mise",
                    "exec",
                    "--",
                    "cargo",
                    "install",
                    "--debug",
                    *source_arguments,
                    "--locked",
                    "--root",
                    str(staged),
                    "--target-dir",
                    str(root / "target/tooling/harmos"),
                    identity["package"],
                ]
            )
            if local_source is not None and _override_source(local_source) != identity["source"]:
                raise ValueError("Harmos source changed while preparing the local tool")
            executable = staged / "bin" / binary.name
            if executable.is_symlink() or not executable.is_file():
                raise ValueError("published Harmos package omitted its required CLI")
            version = subprocess.check_output(
                [str(executable), "--version"],
                text=True,
                encoding="utf-8",
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                stdin=sys.stdin,
                stderr=sys.stderr,
            ).strip()
            value = {
                "identity": identity,
                "sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
                "version": version,
            }
            (staged / "receipt.json").write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
            for path in staged.iterdir():
                if path.name not in {"bin", "receipt.json"}:
                    path.unlink()
            if destination.exists():
                shutil.rmtree(destination)
            shutil.copytree(staged, destination)
    if os.name != "nt":
        binary.chmod(0o755)
    github_path = os.environ.get("GITHUB_PATH")
    if github_path:
        with Path(github_path).open("a", encoding="utf-8") as output:
            output.write(str(binary.parent) + "\n")
    selected = identity.get("rev", identity.get("source"))
    _step(f"Harmos CLI uses {selected} with verified installed bytes")
    return binary


def install_gui_packages(*, runtime: bool = False, desktop: bool = False) -> None:
    """Install the declared GUI runtime or development packages when missing.

    A CI host that is missing them and cannot install them is broken, and the
    linker error it would eventually produce says nothing useful, so only a
    host with no dpkg at all -- a developer machine, never the CI image --
    gets to skip quietly.
    """
    if sys.platform != "linux" or shutil.which("dpkg") is None:
        _step("skipping GUI packages: no dpkg here, so this is not a Debian host")
        return
    selected = GUI_RUNTIME_PACKAGES if runtime else GUI_PACKAGES
    packages = (*selected, *(DESKTOP_PACKAGES if desktop else ()))
    if all(_package_installed(package) for package in packages):
        _step("skipping GUI packages: all prerequisites are installed")
        return
    prefix = []
    if os.geteuid() != 0 and os.environ.get("GITHUB_ACTIONS") == "true" and shutil.which("sudo"):
        prefix = ["sudo", "-n"]
    elif os.geteuid() != 0:
        raise SystemExit("GUI prerequisites are missing and apt-get needs root; install the selected packages or run this as root.")
    _run([*prefix, "apt-get", "update"])
    _run([*prefix, "apt-get", "install", "-y", "--no-install-recommends", *packages])


# === Private Functions === #


def _is_revision(value: object) -> bool:
    return isinstance(value, str) and len(value) == 40 and all(character in "0123456789abcdef" for character in value)


def _locked_revision(root: Path, source: str) -> str:
    # A Local Path Override Leaves No Git Source In The Lockfile
    lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
    for package in lock.get("package", []):
        locked = package.get("source", "")
        if package.get("name") == "harmos" and locked.startswith(source):
            revision = locked.removeprefix(source)
            if _is_revision(revision):
                return revision
    raise ValueError("the Harmos branch has no locked public revision; a local path override is active")


def _override_source(directory: Path) -> str:
    """Identify The Exact Local Harmos Checkout Used By An Explicit Development Override."""
    root = Path(
        subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=directory,
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
        ).stdout.strip()
    )
    files = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root,
        capture_output=True,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
    ).stdout.split(b"\0")
    digest = hashlib.sha256()
    for name in sorted(set(files) - {b""}):
        path = root / os.fsdecode(name)
        digest.update(name + b"\0")
        if path.is_symlink():
            digest.update(b"link\0" + os.fsencode(os.readlink(path)))
        elif path.is_file():
            digest.update(b"file\0" + hashlib.sha256(path.read_bytes()).digest())
        else:
            digest.update(b"missing\0")
    return digest.hexdigest()


def _package_installed(name: str) -> bool:
    return (
        subprocess.run(
            ["dpkg", "-s", name],
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=False,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
        ).returncode
        == 0
    )


def _run(command: Sequence[str]) -> None:
    _step(" ".join(command))
    subprocess.run(
        command, check=True, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0), stdin=sys.stdin, stdout=sys.stdout, stderr=sys.stderr
    )


def _step(message: str) -> None:
    print(f"[ci:bootstrap] {message}", flush=True)


# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
