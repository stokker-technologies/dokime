# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Select Rust Test Targets From Changed Paths And Cargo's Workspace Graph."""

# === Imports === #

# Standard Library Imports
import json
import shlex
import subprocess
import sys
from collections.abc import Sequence
from pathlib import Path

# Third Party Imports
# Local Imports
from .. import temporary
from ..build import cargo
from ..process import spawn_flags
from . import nextest, policy, report

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def display(operation: str, arguments: Sequence[str]) -> str:
    command = ["cargo", "nextest", "run"] if operation == "test" else ["cargo", operation]
    return shlex.join([*command, *arguments])


def run(directory: Path, arguments: Sequence[str], *, plan: bool = False) -> int:
    """Compile Only The Requested Cargo Targets, With The Canonical Story Guards."""
    rules = policy.load(directory)
    values = list(arguments)
    if "--" in values:
        split = values.index("--")
        if values[split + 1 :] == ["--exact"]:
            name = values.pop(split - 1)
            values.append(name)
    names = [values[index + 1] for index, value in enumerate(values[:-1]) if value == "--test"]
    packages = {values[index + 1] for index, value in enumerate(values[:-1]) if value in ("--package", "-p")}
    canonical: set[str] = set()
    if "--tests" in values:
        # Names Drive Deadline Overrides Only, Never Widen The Cargo Selection.
        metadata = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--locked", "--format-version", "1"],
            cwd=directory,
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=True,
            creationflags=spawn_flags(),
        )
        targets = [
            (package["name"], target)
            for package in json.loads(metadata.stdout)["packages"]
            if not packages or package["name"] in packages
            for target in package["targets"]
            if "test" in target["kind"]
        ]
        names = [target["name"] for _, target in targets]
        canonical.update(f"{name}/{feature}" for name, target in targets for feature in target.get("required-features", []))
    if canonical:
        split = values.index("--") if "--" in values else len(values)
        values[split:split] = ["--features", ",".join(sorted(canonical))]
    if plan:
        print(display("test", values))
        return 0
    destination = report.native_directory(directory, Path("target/test-reports/focused")) / "junit.xml"
    destination.unlink(missing_ok=True)
    binaries = {name: name for name in names}
    overrides = nextest.overrides(rules, None, coverage=False, binaries=binaries)
    with temporary.directory() as scratch:
        profile_file = scratch / "nextest.toml"
        profile_file.write_text(nextest.config(str(scratch / "store"), rules.deadline("unit"), overrides), encoding="utf-8")
        command = [
            "cargo",
            "nextest",
            "run",
            "--config-file",
            str(profile_file),
            "--profile",
            nextest.PROFILE,
            "--no-tests",
            "fail",
            *values,
        ]
        result = subprocess.run(
            command,
            cwd=directory,
            env=dict(cargo.compiler_environment("test"), CARGO_BUILD_WARNINGS="deny", TEST_PROFILE="dev"),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            check=False,
            creationflags=spawn_flags(),
        )
        junit = scratch / "store" / nextest.PROFILE / "junit.xml"
        if junit.is_file():
            report.write(destination, junit.read_text(encoding="utf-8"))
        return result.returncode


def selections(
    directory: Path, metadata: dict, paths: Sequence[str], layer: str | None, canonical_features: Sequence[str] = ()
) -> list[list[str]]:
    """Test Edits Select Their Harness; Source Edits Include Transitive Consumers."""
    directory = directory.resolve()
    packages = {package["name"]: package for package in metadata["packages"]}
    roots = {name: Path(package["manifest_path"]).resolve().parent for name, package in packages.items()}
    selected: dict[str, set[str] | None] = {}
    changed_source = set()
    for value in paths:
        path = (directory / value).resolve()
        if not path.is_relative_to(directory):
            raise ValueError(f"changed path escapes the repository: {value}")
        if path in (directory / "Cargo.toml", directory / "Cargo.lock"):
            selected.update(dict.fromkeys(packages))
            changed_source.update(packages)
            continue
        owners = [name for name, root in roots.items() if path.is_relative_to(root)]
        if not owners:
            raise ValueError(f"no Rust package owns {value}; select that language's focused task explicitly")
        owner = max(owners, key=lambda name: len(roots[name].parts))
        relative = path.relative_to(roots[owner])
        if not relative.parts:
            selected[owner] = None
            changed_source.add(owner)
            continue
        if relative.parts[0] == "benches":
            raise ValueError(f"{value} belongs to a benchmark; use mise run bench with its binary selection")
        if relative.parts[0] == "tests":
            all_tests = {target["name"] for target in packages[owner]["targets"] if "test" in target["kind"]}
            targets = {
                target["name"]
                for target in packages[owner]["targets"]
                if "test" in target["kind"] and path.is_relative_to(Path(target["src_path"]).resolve().parent)
            }
            current = selected.get(owner, set())
            if current is not None:
                selected[owner] = current | (targets or all_tests)
        else:
            selected[owner] = None
            changed_source.add(owner)

    # Follow Path Dependencies, Including Renamed And Development Dependencies.
    affected_roots = {roots[name] for name in changed_source}
    while True:
        consumers = {
            name
            for name, package in packages.items()
            if any(Path(dependency["path"]).resolve() in affected_roots for dependency in package["dependencies"] if dependency.get("path"))
        }
        updated = affected_roots | {roots[name] for name in consumers}
        if updated == affected_roots:
            break
        affected_roots = updated
        selected.update(dict.fromkeys(consumers))

    groups: dict[tuple[str, ...], tuple[list[str], set[str]]] = {}
    for name, names in sorted(selected.items()):
        package = packages[name]
        targets = []
        for target in package["targets"]:
            if not target["test"] or "example" in target["kind"]:
                continue
            integration = "test" in target["kind"]
            category = "unit"
            if integration:
                category = next((value for value in ("acceptance", "system") if target["name"].split("-", 1)[0] == value), "integration")
            if layer and layer != category or names is not None and (not integration or target["name"] not in names):
                continue
            targets.append(target)
        if not targets:
            continue
        features = {f"{name}/{feature}" for target in targets for feature in target.get("required-features", [])}
        features.update(feature for feature in canonical_features if feature.split("/", 1)[0] == name)
        selectors = []
        for target in targets:
            if "test" in target["kind"]:
                selectors += ["--test", target["name"]]
            elif "bin" in target["kind"]:
                selectors += ["--bin", target["name"]]
            else:
                selectors += ["--lib"]
        if names is None and layer is None:
            selectors = ["--tests"]
        owners, shared_features = groups.setdefault(tuple(selectors), ([], set()))
        owners += ["--package", name]
        shared_features.update(features)
    commands = []
    for selectors, (owners, features) in groups.items():
        command = ["--locked", *owners]
        if features:
            command += ["--features", ",".join(sorted(features))]
        commands.append([*command, *selectors])
    if not commands:
        raise ValueError("no test targets match the changed paths and layer")
    return commands


def changed(directory: Path, paths: Sequence[str], layer: str | None, *, plan: bool, no_run: bool, timings: bool) -> int:
    # Read Cargo's Declared Ownership Without Resolving Or Compiling Dependencies.
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--locked", "--format-version", "1"],
        cwd=directory,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=True,
        creationflags=spawn_flags(),
    )
    commands = selections(directory, json.loads(result.stdout), paths, layer, [])

    # Run Only The Selected Targets, Preserving Cargo's Compiler Parallelism.
    for command in commands:
        if timings:
            command += ["--timings"]
        if no_run:
            command += ["--no-run"]
        print(display("test", command), flush=True)
        if plan:
            continue
        status = cargo.focused(directory, "test", command)
        if status:
            return status
    return 0


# === Private Functions === #

# === Entry Point === #
