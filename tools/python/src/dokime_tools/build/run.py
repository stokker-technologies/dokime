# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Build Every Development Output Once And Stamp The Staged Fixtures

    python -m dokime_tools.build.run <rust|typescript|python|stamp> [--profile dev|release|bench]

`rust` captures the embedded report viewer, compiles the one workspace nextest archive with
that report as its declared parent input, stages the product executables and packages the
OBC plugin bundle from the archive's compiler output. `typescript` generates the GUI bindings
from the server's contract export and captures the live frontend. `python` verifies the
tracked client bindings and captures the client wheel. `stamp` binds every staged file under
`dist/build/<platform>` to its original producer. Each step records its producers under
`target/build/<profile>/` so later steps never compile again.
"""

# === Imports === #

# Standard Library Imports
import argparse
import importlib.util
import json
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import frontend, outputs, wheel
from dokime_tools.package import fixtures
from dokime_tools.package.contracts import locate
from dokime_tools.package.staging import clear_output, stage_verified
from dokime_tools.prepare import bindings
from dokime_tools.test import archive, policy

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT
PLATFORM = "windows" if sys.platform == "win32" else "linux"
STAGED = ROOT / "dist/build" / PLATFORM
EXECUTABLES = ("dokime", "dokime-server", "cargo-dokime", "dokime-gui")
STEPS = ("rust", "typescript", "python")

# The Hatchling backend closure the offline, nonisolated wheel build imports.
BACKEND = ("hatchling", "packaging", "pathspec", "pluggy", "trove_classifiers")

# === Types === #

# === Classes === #

# === Public Functions === #


def rust(profile: str) -> dict:
    """Compile Once, Then Stage Executables And The Plugin Bundle From That Output."""
    report = frontend.capture(profile, report=True)
    compiled = archive.build(ROOT, profile, parents=[report], inputs=[frontend.REPORT])
    binaries = STAGED / "bin"
    clear_output(binaries)
    selected = outputs.executables(ROOT, compiled, EXECUTABLES)
    files = stage_verified(ROOT, compiled, {path: binaries / path.name for path in selected.values()})
    packaged, plugins = fixtures.prepare(ROOT, compiled, tool=selected["cargo-dokime"], output=STAGED / "plugins")
    return {"producers": [report, compiled, *packaged], "files": {**files, **plugins}}


def typescript(profile: str) -> dict:
    """Generate The GUI Bindings From The Server Export And Capture The Live Frontend."""
    compiled = _archive(profile)
    export = locate(ROOT / compiled["messages"]["path"])
    bindings.typescript(export)
    live = frontend.capture(profile, report=False, parents=[compiled], inputs=[frontend.WEB_ROOT / ".generated"])
    destination = STAGED / "frontend"
    clear_output(destination)
    files = stage_verified(ROOT, live, outputs.exports(live, ROOT / "target/build" / profile / "frontend", destination))
    return {"producers": [live], "files": files}


def python(profile: str) -> dict:
    """Require Current Tracked Client Bindings And Capture The Wheel Offline."""
    compiled = _archive(profile)
    bindings.generate(ROOT / compiled["messages"]["path"], check=True)
    built = wheel.capture(ROOT, compiled, backend())
    destination = STAGED / "python"
    clear_output(destination)
    files = stage_verified(ROOT, built, outputs.exports(built, ROOT / "target/build" / profile / "python", destination))
    return {"producers": [built], "files": files}


def stamp(profile: str) -> dict:
    """Bind Every Staged File To Its Original Producer For The Test Lanes."""
    producers: list[dict] = []
    files: dict[str, str] = {}
    for step in STEPS:
        record = json.loads(_record(profile, step).read_text(encoding="utf-8"))
        producers.extend(record["producers"])
        files.update(record["files"])
    return archive.stamp(ROOT, producers, files, {"platform": PLATFORM})


def backend() -> list[Path]:
    """The Installed Backend Packages The Root Environment Imports For The Wheel."""
    paths = []
    for name in BACKEND:
        spec = importlib.util.find_spec(name)
        if spec is None or not spec.submodule_search_locations:
            raise ValueError(f"the locked root environment lacks the wheel backend package {name}")
        paths.append(Path(next(iter(spec.submodule_search_locations))))
    return paths


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build Every Development Output Once")
    parser.add_argument("step", choices=(*STEPS, "stamp"))
    parser.add_argument("--profile", choices=(*policy.PROFILES, "bench"), default="dev")
    options = parser.parse_args(arguments)
    try:
        if options.step == "stamp":
            stamp(options.profile)
            return 0
        record = {"rust": rust, "typescript": typescript, "python": python}[options.step](options.profile)
    except (policy.TestError, OSError, ValueError) as error:
        print(f"build {options.step}: {error}", file=sys.stderr)
        return 1
    path = _record(options.profile, options.step)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    return 0


# === Private Functions === #


def _archive(profile: str) -> dict:
    record = json.loads(_record(profile, "rust").read_text(encoding="utf-8"))
    return next(producer for producer in record["producers"] if producer.get("messages"))


def _record(profile: str, step: str) -> Path:
    return ROOT / "target/build" / profile / f"{step}.json"


# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
