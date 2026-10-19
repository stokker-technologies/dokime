# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Build Every Development Output Once And Stamp The Staged Fixtures

    python -m dokime_tools.build.run <rust|typescript|python|stamp> [--profile dev|release|bench]

`rust` compiles the one workspace nextest archive, stages the product executables, packages the
OBC plugin bundle from the archive's compiler output, and exports the locked Cargo source closure
and starter wheelhouse for the generated-project story. `typescript` runs protoc-gen-es over the
dokime-server contract export, verifies the tracked GUI client, stages the generated message
modules and builds the live frontend. `python` verifies the tracked client, builds its distributions
into `target/wheels` and stages the canonical remote examples. `stamp` names and hashes the fixtures
the lanes consume. Every staged file lives under `dist/build/<platform>` or `target/wheels`, the
paths a lane runner downloads; each step records its files under `target/build/<profile>/` so later
steps never compile again.
"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import dependencies, frontend, outputs, wheel
from dokime_tools.package import examples, fixtures
from dokime_tools.package.staging import clear_output, stage_file, sync_output
from dokime_tools.prepare import bindings, generated
from dokime_tools.test import archive, policy

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT
PLATFORM = "windows" if sys.platform == "win32" else "linux"
STAGED = ROOT / "dist/build" / PLATFORM
EXECUTABLES = ("dokime", "dokime-server", "cargo-dokime", "dokime-gui")
STEPS = ("rust", "typescript", "python")

# === Types === #

# === Classes === #

# === Public Functions === #


def rust(profile: str) -> list[str]:
    """Compile Once, Then Stage Executables, The Plugin Bundle And The Offline Dependency Closure."""
    # Development And Bench Archives Carry The Runtime's Story Hooks; Release Binaries Never Do.
    stories = [] if profile == "release" else ["--features", "dokime-runtime/fault-injection"]
    messages = archive.build(ROOT, profile, stories).with_name(archive.MESSAGES)
    binaries = STAGED / "bin"
    clear_output(binaries)
    staged = []
    for path in outputs.executables(messages, EXECUTABLES).values():
        stage_file(path, binaries / path.name)
        staged.append(binaries / path.name)
    tool = binaries / f"cargo-dokime{_exe()}"
    staged += fixtures.prepare(ROOT, messages, profile, tool=tool, output=STAGED / "plugins")
    # The Generated-Project Story Builds Offline From The Locked Source Closure And Wheelhouse.
    staged += dependencies.cargo(ROOT, STAGED / "dependencies")
    staged += dependencies.wheelhouse(ROOT, STAGED / "dependencies")
    return _relative(staged)


def typescript(profile: str) -> list[str]:
    """Generate The GUI Message Modules From The Server Export And Build The Live Frontend."""
    messages = _messages(profile)
    subprocess.run(["node", str(frontend.WEB_ROOT / "tools/bindings.mjs"), str(messages), "--check"], cwd=ROOT, check=True)
    generated.stage()
    built = frontend.build(profile)
    destination = STAGED / "frontend"
    sync_output(built, destination)
    return _relative([path for path in destination.rglob("*") if path.is_file()])


def python(profile: str) -> list[str]:
    """Require Current Client Code, Build Distributions Offline And Stage The Canonical Examples."""
    bindings.generate(_messages(profile), check=True)
    distributions = wheel.build(ROOT, profile)
    staged = examples.stage(ROOT, STAGED / "examples/remote")
    return _relative([*distributions, *staged])


def stamp(profile: str) -> None:
    """Name And Hash Every Staged Fixture The Lanes Consume."""
    files: list[str] = []
    for step in STEPS:
        files += json.loads(_record(profile, step).read_text(encoding="utf-8"))["files"]
    archive.stamp(ROOT, profile, files, {"platform": PLATFORM})


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build Every Development Output Once")
    parser.add_argument("step", choices=(*STEPS, "stamp"))
    parser.add_argument("--profile", choices=(*policy.PROFILES, "bench"), default="dev")
    options = parser.parse_args(arguments)
    try:
        if options.step == "stamp":
            stamp(options.profile)
            return 0
        files = {"rust": rust, "typescript": typescript, "python": python}[options.step](options.profile)
    except (policy.TestError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build {options.step}: {error}", file=sys.stderr)
        return 1
    path = _record(options.profile, options.step)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({"files": files}, indent=2) + "\n", encoding="utf-8")
    return 0


# === Private Functions === #


def _exe() -> str:
    return ".exe" if sys.platform == "win32" else ""


def _messages(profile: str) -> Path:
    return archive.location(ROOT, profile).with_name(archive.MESSAGES)


def _record(profile: str, step: str) -> Path:
    return ROOT / "target/build" / profile / f"{step}.json"


def _relative(paths: list[Path]) -> list[str]:
    return sorted(path.resolve().relative_to(ROOT.resolve()).as_posix() for path in paths)


# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
