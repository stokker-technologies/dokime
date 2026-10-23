# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Produce Shipping Outputs, Test Fixtures And Benchmark Inputs In Their Own Scopes"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import bench, cargo, dependencies, frontend, outputs, paths, run, wheel
from dokime_tools.package import fixtures
from dokime_tools.package.staging import clear_output, stage_file, sync_output
from dokime_tools.prepare import bindings, generated
from dokime_tools.test import archive, policy

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT
EXECUTABLES = ("dokime", "dokime-server", "cargo-dokime")
STEPS = ("rust", "typescript", "desktop", "python", "tests", "bench")

# === Types === #

# === Classes === #

# === Public Functions === #


def rust(profile: str) -> list[str]:
    """Build Shipping Native Bytes Before Compiling The Frontend-Embedding Desktop."""
    output = paths.output(ROOT, "build", profile)
    messages = output / "product/compiler.json"
    messages.parent.mkdir(parents=True, exist_ok=True)
    arguments = [
        "--locked",
        "--workspace",
        "--exclude",
        "dokime-gui",
        "--exclude",
        "dokime-fixtures",
        "--profile",
        profile,
        "--features",
        "dokime-server-cli/media-native",
    ]
    cargo.clippy(ROOT, arguments)
    command = ["cargo", "build", *arguments, "--message-format=json-render-diagnostics"]
    with messages.open("w", encoding="utf-8") as written:
        subprocess.run(
            command,
            cwd=ROOT,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    binaries = output / "bin"
    clear_output(binaries)
    staged = []
    for path in outputs.executables(messages, EXECUTABLES).values():
        stage_file(path, binaries / path.name)
        staged.append(binaries / path.name)
    tool = binaries / f"cargo-dokime{_exe()}"
    staged += fixtures.prepare(ROOT, messages, profile, tool=tool, output=output / "plugins")
    run.record_target(ROOT, messages, profile)
    return _relative(staged)


def typescript(profile: str) -> list[str]:
    """Generate The GUI Message Modules From The Server Export And Build The Live Frontend."""
    messages = _messages(profile)
    subprocess.run(
        ["node", str(frontend.WEB_ROOT / "tools/bindings.mjs"), str(messages), "--check"],
        cwd=ROOT,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    generated.stage(profile)
    built = frontend.build(profile)
    return _relative([path for path in built.rglob("*") if path.is_file()])


def python(profile: str) -> list[str]:
    """Require Current Tracked Client Code And Build The Wheel Offline."""
    bindings.generate(_messages(profile), check=True)
    return _relative([wheel.build(ROOT, profile)])


def desktop(profile: str) -> list[str]:
    """Embed The Already-Built Frontend In The Shipping Desktop Executable."""
    output = paths.output(ROOT, "build", profile)
    if not (output / "web/index.html").is_file():
        raise ValueError("build the matching frontend before compiling the desktop")
    sync_output(output / "web", frontend.WEB_ROOT / "dist")
    messages = output / "product/desktop-compiler.json"
    arguments = [
        "--locked",
        "--package",
        "dokime-gui",
        "--profile",
        profile,
        "--features",
        "tauri/custom-protocol",
    ]
    cargo.clippy(ROOT, arguments)
    command = ["cargo", "build", *arguments, "--message-format=json-render-diagnostics"]
    with messages.open("w", encoding="utf-8") as written:
        subprocess.run(
            command,
            cwd=ROOT,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    source = outputs.executables(messages, ("dokime-gui",))["dokime-gui"]
    destination = output / "bin" / source.name
    stage_file(source, destination)
    run.record_target(ROOT, messages, profile)
    return _relative([destination])


def tests(profile: str) -> list[str]:
    """Compile One Archive And Capture Inputs Needed By Transported Test Lanes."""
    output = paths.output(ROOT, "build", profile)
    if not (output / "product/target.txt").is_file():
        raise ValueError("build the matching Product before its Tests producer")
    _compatibility(profile)
    messages = archive.build(ROOT, profile, ["--features", "dokime-runtime/fault-injection"]).with_name(archive.MESSAGES)
    metadata = messages.with_name("metadata.json")
    with metadata.open("w", encoding="utf-8") as written:
        subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            cwd=ROOT,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    directory = output / "fixtures"
    staged = dependencies.cargo(ROOT, directory / "dependencies")
    staged += dependencies.wheelhouse(ROOT, directory / "dependencies")
    sync_output(output / "generated/typescript", directory / "typescript")
    staged += [path for path in (directory / "typescript").rglob("*") if path.is_file()]
    staged += [archive.location(ROOT, profile), messages, metadata]
    files = _relative(staged)
    archive.stamp(ROOT, profile, files)
    return files


def benchmarks() -> list[str]:
    """Build Benchmarks And Package Their Matching Plugin Without Recompiling It."""
    messages = bench.build(ROOT, ["--features", "dokime-runtime/fault-injection"])
    output = paths.output(ROOT, "build", "bench")
    tool = output / "bin" / f"cargo-dokime{_exe()}"
    staged = fixtures.prepare(ROOT, messages, "bench", tool=tool, output=output / "plugins")
    record = json.loads((output / "fixtures/stamp.json").read_text(encoding="utf-8"))
    files = sorted(set(record["files"]) | set(_relative(staged)))
    archive.stamp(ROOT, "bench", files)
    return files


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build Every Development Output Once")
    parser.add_argument("step", choices=STEPS)
    parser.add_argument("--profile", choices=policy.PROFILES, default="dev")
    options = parser.parse_args(arguments)
    try:
        if options.step == "bench":
            benchmarks()
        else:
            {
                "rust": rust,
                "typescript": typescript,
                "desktop": desktop,
                "python": python,
                "tests": tests,
            }[options.step](options.profile)
    except (policy.TestError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build {options.step}: {error}", file=sys.stderr)
        return 1
    return 0


# === Private Functions === #


def _compatibility(profile: str) -> None:
    """Compile Every Declared Feature And Browser Target In The Optional Tests Producer."""
    compiler_profile = "release-tests" if profile == "release" else profile
    for selected in (
        ["--workspace", "--all-targets", "--all-features"],
        ["--workspace", "--lib", "--bins", "--no-default-features"],
        ["--package", "dokime-domain", "--target", "wasm32-unknown-unknown"],
    ):
        cargo.clippy(ROOT, ["--locked", "--profile", compiler_profile, *selected])


def _exe() -> str:
    return ".exe" if sys.platform == "win32" else ""


def _messages(profile: str) -> Path:
    return paths.output(ROOT, "build", profile) / "product/compiler.json"


def _relative(paths: list[Path]) -> list[str]:
    return sorted(path.resolve().relative_to(ROOT.resolve()).as_posix() for path in paths)


# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
