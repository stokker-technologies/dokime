# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Registry Crates Operations."""

# === Imports === #

# Standard Library Imports
import json
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Callable
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ... import policy
from ...package import changelog
from ...process import spawn_flags
from .. import notices, pins
from . import provider

# Type Checking Imports

# === Public Exports === #

# === Constants === #

CRATES_INDEX = "https://index.crates.io"
RATE_INTERVAL_S = 600
RATE_BUDGET_S = 5400

# === Types === #

# === Classes === #

# === Public Functions === #


def publish(directory: Path, *, sleep: Callable[[float], None] = time.sleep, clock: Callable[[], float] = time.monotonic) -> None:
    """Publish Workspace Crates One At A Time In Dependency Order, Resuming Past Published Versions."""

    if policy.execute(directory, ["git", "status", "--porcelain"], capture=True).strip():
        raise policy.ToolError("Publish Crates: crate publication requires clean source before generated CHANGELOG staging")
    pins.check(directory)
    packages = inventory(directory)
    deadline = clock() + RATE_BUDGET_S
    with changelog.staged(directory, packages):
        _publish_crates(directory, packages, deadline, sleep=sleep, clock=clock)


def rehearse(directory: Path, *, skip_published: bool = True, channel: str = "stable") -> None:
    """Dry-Run Crates In Publication Order Without Publishing Credentials."""
    if not pins.check(directory, warn=channel == "nightly"):
        return
    with policy.credential_free(policy.policy(directory, "release")):
        packages = inventory(directory)
        metadata = policy.execute(directory, ["cargo", "metadata", "--format-version", "1", "--no-deps"], capture=True)
        target = Path(json.loads(metadata)["target_directory"])
        lock = directory / "Cargo.lock"
        original = lock.read_bytes() if lock.is_file() else None
        staged = []
        with changelog.staged(directory, packages):
            _rehearse_crates(directory, packages, target, lock, original, staged, skip_published=skip_published)


def inventory(directory: Path) -> dict:
    """Publishable Workspace Members By Name."""
    metadata = json.loads(policy.execute(directory, ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"], capture=True))
    return policy.publishable(directory, metadata)


def published(name: str, version: str | None = None) -> bool:
    """Whether The Sparse Index Lists This Crate, Or This Exact Crate Version."""
    lower = name.lower()
    prefix = {1: "1", 2: "2", 3: "3/" + lower[0]}.get(len(lower), f"{lower[0:2]}/{lower[2:4]}")
    try:
        lines = notices.download(f"{CRATES_INDEX}/{prefix}/{lower}", operation="crates.io index read").decode("utf-8").splitlines()
    except policy.ToolError as error:
        cause = error.__cause__
        if isinstance(cause, urllib.error.HTTPError) and cause.code == 404:
            return False
        if isinstance(cause, urllib.error.URLError) and isinstance(cause.reason, FileNotFoundError):
            return False
        raise
    entries = []
    for entries_line in lines:
        if not (entries_line.strip()):
            continue
        entries.append(json.loads(entries_line))

    return bool(entries) if version is None else any(entry.get("vers") == version for entry in entries)


def _publish_crates(
    directory: Path, packages: dict, deadline: float, *, sleep: Callable[[float], None], clock: Callable[[], float]
) -> None:
    for name in _publication_order(packages):
        version = packages[name]["version"]
        if published(name, version):
            print(f"{name} {version}: already published")
            continue
        while True:
            result = subprocess.run(
                ["cargo", "publish", "--locked", "--allow-dirty", "--package", name],
                creationflags=spawn_flags(),
                cwd=directory,
                capture_output=True,
                text=True,
                encoding="utf-8",
                check=False,
            )
            output = (result.stdout or "") + (result.stderr or "")
            if result.returncode == 0:
                print(f"{name} {version}: published")
                break
            if "already exists" in output:
                print(f"{name} {version}: already published")
                break
            if "429" not in output and "Too Many Requests" not in output:
                raise policy.ToolError(f"Publish Crates: cargo publish {name} failed with exit code {result.returncode}: {output}")
            wait = provider._retry_after(output)
            if clock() + wait > deadline:
                raise policy.ToolError(f"Publish Crates: {name} {version}: crates.io rate limit outlasted the publication budget")
            print(f"{name} {version}: rate limited; retrying in {wait}s")
            sleep(wait)


def _rehearse_crates(
    directory: Path, packages: dict, target: Path, lock: Path, original: bytes | None, staged: list, *, skip_published: bool
) -> None:
    try:
        for name in _publication_order(packages):
            version = packages[name]["version"]
            if skip_published and published(name, version):
                print(f"{name} {version}: already published; the real leaf skips it")
                continue
            # A sibling resolves from its packaged sources only once the real leaf would have published it, so a
            # crate ordered before something its published manifest requires fails here, before any tag exists.
            command = ["cargo", "publish", "--dry-run", "--allow-dirty", "--package", name]
            for sibling, release in staged:
                command += ["--config", f'patch.crates-io.{sibling}.path="{(target / "package" / f"{sibling}-{release}").as_posix()}"']
            policy.execute(directory, command)
            staged.append((name, version))
    finally:
        # The patches record themselves in the lock file; the rehearsal leaves the committed lock untouched.
        if original is not None:
            lock.write_bytes(original)


def _publication_order(packages: dict) -> list[str]:
    """Publishable Crates After Every Publishable Crate Their Published Manifest Requires, Ties By Name.

    A dev-dependency that carries a version stays in the published manifest and must resolve from the registry; a
    path-only one is stripped on packaging.
    """
    requires = {}
    for requires_name, requires_package in packages.items():
        requires[requires_name] = sorted(
            {
                dependency["name"]
                for dependency in requires_package["dependencies"]
                if (dependency.get("kind") != "dev" or dependency.get("req", "*") != "*")
                and dependency["name"] in packages
                and (dependency["name"] != requires_name)
            }
        )

    order = []
    visiting = set()

    def visit(name: str) -> None:
        if name in order:
            return
        if name in visiting:
            cycle = ", ".join(sorted(visiting))
            raise policy.ToolError(
                f'Publish Crates: publishable crates {cycle} require each other at {name}; a dev-dependency on a sibling that carries a version stays in the published manifest, so declare it path-only ({{ path = "../sibling" }}, no version) and cargo strips it on packaging'
            )
        visiting.add(name)
        for dependency in requires[name]:
            visit(dependency)
        visiting.discard(name)
        order.append(name)

    for name in sorted(packages):
        visit(name)
    return order


# === Private Functions === #

# === Entry Point === #
