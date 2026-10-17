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
from collections.abc import Callable, Sequence
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.release.run import _override_source

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
VC_TOOLSET_COMPONENT = "Microsoft.VisualStudio.Component.VC.Tools.x86.x64"
VSWHERE_RELATIVE_PATH = Path("Microsoft Visual Studio/Installer/vswhere.exe")

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
    system = platform.system()
    _step(f"host system is {system}")
    _create_temporary_directory()
    for step in platform_steps(system, gui_packages=arguments.gui_packages):
        step()
    for task in arguments.tasks:
        _run_mise_task(task)
    if arguments.role:
        prepare_tools(arguments.role)
    _step("bootstrap complete")
    return 0


def parse_arguments(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="bootstrap.py",
        description="Prepare a CI runner and run the requested mise sync tasks.",
    )
    parser.add_argument(
        "--gui-packages",
        action="store_true",
        help="install the Debian GUI development packages (Linux jobs only).",
    )
    parser.add_argument("--harmos-key", action="store_true")
    parser.add_argument("--harmos-tools", action="store_true")
    parser.add_argument("--cargo-config", type=Path)
    parser.add_argument(
        "--role",
        choices=(
            "check",
            "contracts",
            "build",
            "test",
            "coverage",
            "catalog",
            "typescript",
            "publish",
        ),
    )
    parser.add_argument(
        "tasks",
        nargs="*",
        help="mise tasks to run once the host is prepared, in order.",
    )
    return parser.parse_args(argv)


def platform_steps(system: str, *, gui_packages: bool) -> list[Callable[[], None]]:
    """Name the host-specific steps for a system, in the order they run."""
    if system == "Linux":
        return [install_gui_packages] if gui_packages else []
    if system == "Windows":
        # The GUI packages have no Windows counterpart; the MSVC toolset does,
        # and every Rust job needs it.
        return [verify_visual_cpp_toolset]
    raise SystemExit(f"bootstrap.py supports Linux and Windows CI hosts, not {system}.")


def selected_tools(role: str, system: str) -> list[str]:
    pnpm = "npm:pnpm" if system == "Windows" else "pnpm"
    common = ["python", "uv", "rust", "node", pnpm]
    if role in {"check", "build"}:
        return []  # Complete Maintained-Source Checks Use The Pinned Tool Set.
    if role in {"catalog", "typescript"}:
        return [*common, "sccache"]
    if role == "publish":
        return [
            *common,
            "sccache",
            "gh",
            *(["aqua:jedisct1/minisign"] if system == "Linux" else []),
        ]
    if role == "contracts":
        return [*common, "sccache"]
    if role in {"test", "coverage"}:
        tools = [*common, "sccache", "aqua:nextest-rs/nextest/cargo-nextest"]
        if role == "coverage":
            tools.append("aqua:taiki-e/cargo-llvm-cov")
        return tools
    raise ValueError(f"unknown CI tool role: {role}")


def prepare_tools(role: str) -> None:
    _run(["mise", "install", "--locked", *selected_tools(role, platform.system())])
    _run(["git", "diff", "--exit-code", "--", "mise.lock"])
    if role == "build":
        _run(["mise", "run", "--skip-tools", "sync"])
    else:
        _run(["mise", "run", "--skip-tools", "prepare:tools"])
        if role == "check":
            _run(
                [
                    "mise",
                    "exec",
                    "--",
                    "pnpm",
                    "--dir",
                    "crates/dokime-gui",
                    "install",
                    "--frozen-lockfile",
                ]
            )
        elif role in {"test", "coverage"}:
            # Generated-Plugin And Incremental Scenarios Retain Their Own Compilation.
            _run(["mise", "exec", "--", "cargo", "fetch", "--locked"])
    if role in {"build", "contracts"}:
        prepare_harmos(Path.cwd(), allow_install=role == "contracts")
    if role == "typescript":
        _run(
            [
                "mise",
                "exec",
                "--",
                "pnpm",
                "--dir",
                "crates/dokime-gui",
                "install",
                "--frozen-lockfile",
            ]
        )
    for tool in ("mise", "rustc", "cargo", "python", "uv", "node", "pnpm"):
        _run(["mise", "exec", "--", tool, "--version"])


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
    identity = {
        **source_identity,
        "profile": "dev",
        "platform": platform.system(),
        "architecture": platform.machine(),
        "rustc": subprocess.check_output(["mise", "exec", "--", "rustc", "-vV"], text=True, encoding="utf-8").strip(),
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
        with tempfile.TemporaryDirectory(prefix="harmos-tool-", dir=destination.parent) as temporary:
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
            version = subprocess.check_output([str(executable), "--version"], text=True, encoding="utf-8").strip()
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


def install_gui_packages() -> None:
    """Install the Debian GUI development packages when they are missing.

    A CI host that is missing them and cannot install them is broken, and the
    linker error it would eventually produce says nothing useful, so only a
    host with no dpkg at all -- a developer machine, never the CI image --
    gets to skip quietly.
    """
    if shutil.which("dpkg") is None:
        _step("skipping GUI packages: no dpkg here, so this is not a Debian host")
        return
    if all(_package_installed(package) for package in GUI_PACKAGES):
        _step("skipping GUI packages: all prerequisites are installed")
        return
    prefix = []
    if os.geteuid() != 0 and os.environ.get("GITHUB_ACTIONS") == "true" and shutil.which("sudo"):
        prefix = ["sudo", "-n"]
    elif os.geteuid() != 0:
        raise SystemExit(
            f"{GUI_MARKER_PACKAGE} is missing and apt-get needs root; install the GUI development packages by hand or run this as root."
        )
    _run([*prefix, "apt-get", "update"])
    _run([*prefix, "apt-get", "install", "-y", "--no-install-recommends", *GUI_PACKAGES])


def verify_visual_cpp_toolset() -> None:
    """Fail early when the MSVC toolset rustc links through is not installed.

    The Windows setup script initializes the developer environment before
    installing Cargo-based tools, then persists linker inputs for later jobs.
    This check verifies that the requested native toolset is installed.
    """
    program_files = os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)")
    vswhere = Path(program_files) / VSWHERE_RELATIVE_PATH
    if not vswhere.is_file():
        raise SystemExit(
            f"vswhere.exe not found at '{vswhere}'; install Visual Studio with the 'Desktop development with C++' workload on this runner."
        )
    _step(f"resolving the MSVC toolset through {vswhere}")
    completed = subprocess.run(
        [
            str(vswhere),
            "-utf8",
            "-latest",
            "-products",
            "*",
            "-requires",
            VC_TOOLSET_COMPONENT,
            "-property",
            "installationPath",
        ],
        capture_output=True,
        check=True,
        text=True,
        encoding="utf-8",
    )
    installation = completed.stdout.strip()
    if not installation:
        raise SystemExit(
            f"no Visual Studio instance provides {VC_TOOLSET_COMPONENT}; install the "
            "'Desktop development with C++' workload on this runner."
        )
    _step(f"MSVC toolset found at {installation}")


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


def _create_temporary_directory() -> None:
    temporary = os.environ.get("TMPDIR")
    if not temporary:
        _step("skipping temporary directory: TMPDIR is unset")
        return
    _step(f"creating {temporary}")
    Path(temporary).mkdir(parents=True, exist_ok=True)


def _package_installed(name: str) -> bool:
    return (
        subprocess.run(
            ["dpkg", "-s", name],
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=False,
        ).returncode
        == 0
    )


def _run_mise_task(task: str) -> None:
    _run(["mise", "run", task])


def _run(command: Sequence[str]) -> None:
    _step(" ".join(command))
    subprocess.run(command, check=True)


def _step(message: str) -> None:
    print(f"[ci:bootstrap] {message}", flush=True)


# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
