# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Select Rust Test Targets from Changed Paths and Cargo's Workspace Graph."

# === Imports === #

# Standard Library Imports
import json
import shlex
import subprocess
import sys
import xml.etree.ElementTree
from collections.abc import Sequence
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.test.credentials
import dokime_tools.test.scratch

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
    """Compile Only the Requested Cargo Targets, with the Canonical Story Guards."""
    rules = policy.load(directory)
    values = list(arguments)
    if "--" in values:
        split = values.index("--")
        if values[split + 1 :] == ["--exact"]:
            name = values.pop(split - 1)
            values.append(name)
    names = []
    for names_index, names_value in enumerate(values[:-1]):
        if names_value != "--test":
            continue
        names.append(values[names_index + 1])

    packages = set()
    for packages_index, packages_value in enumerate(values[:-1]):
        if packages_value not in ("--package", "-p"):
            continue
        packages.add(values[packages_index + 1])

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
        targets = []
        for targets_package in json.loads(metadata.stdout)["packages"]:
            if not (not packages or targets_package["name"] in packages):
                continue
            for targets_target in targets_package["targets"]:
                if "test" not in targets_target["kind"]:
                    continue
                targets.append((targets_package["name"], targets_target))

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
    scratch_directory = temporary.Directory()
    scratch = scratch_directory.path
    try:
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
        print(f"Run Rust Stories: {shlex.join(command)}", flush=True)
        environment = dict(cargo.compiler_environment("test"), CARGO_BUILD_WARNINGS="deny", TEST_PROFILE="dev")
        scratch_root = dokime_tools.test.scratch.Root(environment, destination.parent)
        try:
            scratch_environment = scratch_root.environment
            credential_store = dokime_tools.test.credentials.Store(scratch_environment) if names else None
            try:
                selected = credential_store.environment if credential_store is not None else scratch_environment
                result = subprocess.run(
                    command,
                    cwd=directory,
                    env=selected,
                    stdin=sys.stdin,
                    stdout=sys.stdout,
                    stderr=sys.stderr,
                    check=False,
                    creationflags=spawn_flags(),
                )
            finally:
                if credential_store is not None:
                    credential_store.close()
        finally:
            try:
                junit = scratch / "store" / nextest.PROFILE / "junit.xml"
                if junit.is_file():
                    report.write(destination, junit.read_text(encoding="utf-8"))
            finally:
                scratch_root.close()
        if result.returncode:
            detail = f"Story Diagnostics: {destination}" if destination.is_file() else "The Runner Produced No JUnit Report"
            print(f"Run Rust Stories: Exit {result.returncode}; {detail}", file=sys.stderr, flush=True)
            if destination.is_file():
                for case in xml.etree.ElementTree.parse(destination).iter("testcase"):
                    for failure in (*case.findall("failure"), *case.findall("error")):
                        print(f"{case.get('classname')}::{case.get('name')}: {failure.get('message', '')}", file=sys.stderr)
                        print(failure.text or "", file=sys.stderr, flush=True)
        return result.returncode
    finally:
        scratch_directory.close()


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
            raise ValueError(f"Run Focused Tests: changed path escapes the repository: {value}")
        if path in (directory / "Cargo.toml", directory / "Cargo.lock"):
            selected.update(dict.fromkeys(packages))
            changed_source.update(packages)
            continue
        owners = []
        for owners_name, owners_root in roots.items():
            if not (path.is_relative_to(owners_root)):
                continue
            owners.append(owners_name)

        if not owners:
            raise ValueError(f"Run Focused Tests: no Rust package owns {value}; select that language's focused task explicitly")

        def sort_key(name: str) -> int:
            return len(roots[name].parts)

        owner = max(owners, key=sort_key)
        relative = path.relative_to(roots[owner])
        if not relative.parts:
            selected[owner] = None
            changed_source.add(owner)
            continue
        if relative.parts[0] == "benches":
            raise ValueError(f"Run Focused Tests: {value} belongs to a benchmark; use mise run bench with its binary selection")
        if relative.parts[0] == "tests":
            all_tests = set()
            for all_tests_target in packages[owner]["targets"]:
                if "test" not in all_tests_target["kind"]:
                    continue
                all_tests.add(all_tests_target["name"])

            targets = set()
            for targets_target in packages[owner]["targets"]:
                if "test" not in targets_target["kind"]:
                    continue
                if not path.is_relative_to(Path(targets_target["src_path"]).resolve().parent):
                    continue
                targets.add(targets_target["name"])

            current = selected.get(owner, set())
            if current is not None:
                selected[owner] = current | (targets or all_tests)
        else:
            selected[owner] = None
            changed_source.add(owner)

    # Follow Path Dependencies, Including Renamed And Development Dependencies.
    affected_roots = {roots[name] for name in changed_source}
    while True:
        consumers = set()
        for consumers_name, consumers_package in packages.items():
            if not (
                any(
                    Path(dependency["path"]).resolve() in affected_roots
                    for dependency in consumers_package["dependencies"]
                    if dependency.get("path")
                )
            ):
                continue
            consumers.add(consumers_name)

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
            if not target["test"]:
                continue
            if "example" in target["kind"]:
                continue
            integration = "test" in target["kind"]
            category = "unit"
            if integration:
                category = next((value for value in ("acceptance", "system") if target["name"].split("-", 1)[0] == value), "integration")
            if layer and layer != category:
                continue
            if names is not None and (not integration or target["name"] not in names):
                continue
            targets.append(target)
        if not targets:
            continue
        features = set()
        for features_target in targets:
            for features_feature in features_target.get("required-features", []):
                features.add(f"{name}/{features_feature}")

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
        raise ValueError("Run Focused Tests: no test targets match the changed paths and layer")
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
