# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Packageable Products Without Test Harnesses Or Story Features"""

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import frontend, outputs, paths, wheel
from dokime_tools.package import examples, plugins
from dokime_tools.package.staging import clear_output, stage_file
from dokime_tools.prepare import bindings, generated

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT
EXECUTABLES = ("dokime", "dokime-server", "cargo-dokime", "dokime-gui")
PACKAGES = (
    "dokime",
    "dokime-server",
    "cargo-dokime",
    "dokime-gui",
    "dokime-domain",
    "dokime-runtime",
    "dokime-client",
    "dokime-packaging",
    "dokime-host",
    "dokime-macros",
    "obc-plugin",
    "obc-runtime",
)
PROFILES = ("dev", "release")

# === Types === #

# === Classes === #

# === Public Functions === #


def rust(profile: str) -> list[Path]:
    """Compile Shipping Targets And Stage Their Binaries, Libraries And Plugin Bundle."""
    if profile not in PROFILES:
        raise ValueError("product builds require dev or release")
    if not (frontend.WEB_ROOT / "dist/index.html").is_file():
        raise ValueError("the packaged GUI requires compiled frontend assets; run `mise run build` first")
    output = paths.output(ROOT, "build", profile)
    messages = output / "product/compiler.json"
    messages.parent.mkdir(parents=True, exist_ok=True)
    compiler = subprocess.check_output(["rustc", "-vV"], cwd=ROOT, text=True)
    hosts = [line.removeprefix("host: ") for line in compiler.splitlines() if line.startswith("host: ")]
    host = hosts[0]
    triple = os.environ.get("CARGO_BUILD_TARGET", host)
    command = ["cargo", "build", "--locked", "--profile", profile, "--message-format=json", "--features", "tauri/custom-protocol"]
    for package in PACKAGES:
        command += ["--package", package]
    with messages.open("w", encoding="utf-8") as written:
        subprocess.run(command, cwd=ROOT, stdout=written, check=True)
    compiled = outputs.executables(messages, EXECUTABLES)
    targets = {
        path.parent.parent.name
        for path in compiled.values()
        if "-linux-" in path.parent.parent.name
        or "-windows-" in path.parent.parent.name
        or path.parent.parent.name.endswith("-apple-darwin")
    }
    if len(targets) > 1:
        raise ValueError("Product Build exported more than one Rust target")
    actual = next(iter(targets), triple)
    if "CARGO_BUILD_TARGET" in os.environ and actual != triple:
        raise ValueError("Product Build target differs from CARGO_BUILD_TARGET")
    (messages.parent / "target.txt").write_text(actual + "\n", encoding="utf-8")

    binaries = output / "bin"
    libraries = output / "lib"
    clear_output(binaries)
    clear_output(libraries)
    staged = []
    for source in compiled.values():
        destination = binaries / source.name
        stage_file(source, destination)
        staged.append(destination)
    for source in outputs.libraries(messages, PACKAGES):
        destination = libraries / source.name
        stage_file(source, destination)
        staged.append(destination)
    tool = binaries / f"cargo-dokime{'.exe' if sys.platform == 'win32' else ''}"
    staged += plugins.prepare(ROOT, messages, profile, tool=tool, output=output / "plugins")
    return staged


def typescript(profile: str) -> list[Path]:
    """Resolve Shipping Server Contracts Before Compiling The Embedded Frontend."""
    frontend.ensure_report()
    messages = paths.output(ROOT, "build", profile) / "product/contracts.jsonl"
    messages.parent.mkdir(parents=True, exist_ok=True)
    command = ["cargo", "check", "--locked", "--package", "dokime-server", "--profile", profile, "--message-format=json"]
    with messages.open("w", encoding="utf-8") as written:
        subprocess.run(command, cwd=ROOT, stdout=written, check=True)

    subprocess.run(["node", str(frontend.WEB_ROOT / "tools/bindings.mjs"), str(messages), "--check"], cwd=ROOT, check=True)
    modules = generated.stage(profile)
    built = frontend.build(profile)
    return [path for root in (built, modules) for path in root.rglob("*") if path.is_file()]


def python(profile: str) -> list[Path]:
    """Build The Client Distribution And Its Matching Portable Tutorial."""
    messages = paths.output(ROOT, "build", profile, "any") / "product/contracts.jsonl"
    messages.parent.mkdir(parents=True, exist_ok=True)
    command = ["cargo", "check", "--locked", "--package", "dokime-server", "--profile", profile, "--message-format=json"]
    with messages.open("w", encoding="utf-8") as written:
        subprocess.run(command, cwd=ROOT, stdout=written, check=True)
    bindings.generate(messages, check=True)

    distributions = wheel.build(ROOT, profile)
    wheels = [path for path in distributions if path.suffix == ".whl"]
    if len(wheels) != 1:
        raise ValueError("the tutorial requires exactly one matching built wheel")

    destination = paths.output(ROOT, "build", profile, "any") / "python/examples/remote"
    staged = examples.stage(ROOT, destination, wheel=wheels[0])
    return [*distributions, *staged]


def files(profile: str) -> list[Path]:
    """Require Matching Product Hosts And Enumerate The Outputs Tests Consume."""
    if profile not in PROFILES:
        raise ValueError("product outputs require dev or release")
    native = paths.output(ROOT, "build", profile)
    independent = paths.output(ROOT, "build", profile, "any")
    suffix = ".exe" if sys.platform == "win32" else ""
    required = [native / "bin" / f"{name}{suffix}" for name in EXECUTABLES]
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
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("step", choices=("rust", "typescript", "python"))
    parser.add_argument("--profile", choices=PROFILES, default="dev")
    options = parser.parse_args(arguments)
    try:
        {"rust": rust, "typescript": typescript, "python": python}[options.step](options.profile)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build product {options.step}: {error}", file=sys.stderr)
        return 1
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
