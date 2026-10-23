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
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import cargo, frontend, outputs, paths, run, wheel
from dokime_tools.package import examples, plugins
from dokime_tools.package.staging import clear_output, stage_file, sync_output
from dokime_tools.prepare import bindings, generated
from dokime_tools.test import policy

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT
EXECUTABLES = ("dokime", "dokime-server", "cargo-dokime")
PROFILES = policy.PROFILES
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
        "--profile",
        profile,
    ]
    if profile == "release":
        arguments += ["--features", "dokime-server-cli/media-native"]
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
    libraries = output / "lib"
    clear_output(libraries)
    for source in outputs.libraries(
        messages,
        (
            "dokime",
            "dokime-domain",
            "dokime-runtime",
            "dokime-client",
            "dokime-host",
            "dokime-macros",
            "dokime-packaging",
            "obc-plugin",
            "obc-runtime",
        ),
    ):
        destination = libraries / source.name
        stage_file(source, destination)
        staged.append(destination)
    tool = binaries / f"cargo-dokime{_exe()}"
    staged += plugins.prepare(ROOT, messages, profile, tool=tool, output=output / "plugins")
    run.record_target(ROOT, messages, profile)
    return _relative(staged)


def typescript(profile: str) -> list[str]:
    """Generate The GUI Message Modules From The Server Export And Build The Live Frontend."""
    messages = _messages(profile)
    bindings.generate(messages, check=True)
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
    """Build Tracked Client Distributions And Their Exact Portable Tutorial."""
    distributions = wheel.build(ROOT, profile)
    wheels = [path for path in distributions if path.suffix == ".whl"]
    if len(wheels) != 1:
        raise ValueError("the tutorial requires exactly one matching built wheel")
    destination = paths.output(ROOT, "build", profile, "any") / "python/examples/remote"
    staged = examples.stage(ROOT, destination, wheel=wheels[0])
    return _relative([*distributions, *staged])


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
    """Compile Tests And Starter Inputs Through Their Optional Build Owner."""
    from dokime_tools.build import tests as producer

    return producer.build(profile)


def benchmarks() -> list[str]:
    """Produce The Optional Benchmark Hosts And Their Matching OBC Suite."""
    from dokime_tools.build import benchmarks as producer

    producer.build()
    return []


def files(profile: str) -> list[Path]:
    """Require Matching Product Hosts And Enumerate The Outputs Tests Consume."""
    if profile not in PROFILES:
        raise ValueError("product outputs require dev or release")
    native = paths.output(ROOT, "build", profile)
    independent = paths.output(ROOT, "build", profile, "any")
    suffix = ".exe" if sys.platform == "win32" else ""
    required = [native / "bin" / f"{name}{suffix}" for name in (*EXECUTABLES, "dokime-gui")]
    required += [
        native / "plugins/obc-plugin.dokime",
        native / "product/compiler.json",
        native / "product/target.txt",
        native / "web/index.html",
    ]
    wheel_directory = independent / "python"
    wheels = list(wheel_directory.glob("dokime_client-*.whl"))
    complete = all(path.is_file() for path in required)
    if len(wheels) != 1 or not complete:
        raise ValueError(f"tests require matching {profile} Product outputs; run `mise run build --profile {profile}` first")

    kinds = ("bin", "lib", "plugins", "web", "generated/typescript")
    roots = [native / kind for kind in kinds]
    roots += [independent / "python"]
    staged = {path for root in roots for path in root.rglob("*") if path.is_file()}
    return sorted(staged)


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
