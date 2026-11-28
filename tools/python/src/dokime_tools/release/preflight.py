# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Rehearse Release / Prepare and the Tag Gate Before a Release, One Actionable Line per Failure."""

# === Imports === #

# Standard Library Imports
import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Callable, Generator
from contextlib import contextmanager
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.ci.review
from dokime_tools.build import wheel

from .. import policy
from ..process import spawn_flags
from . import context, pins, prepare, registry, version

# Type Checking Imports

# === Public Exports === #

# === Constants === #

FAILURES = (policy.ToolError, OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError)

# === Types === #

# === Classes === #

# === Public Functions === #


def packaging(directory: Path) -> list[str]:
    """Prepare HEAD in a Throwaway Worktree, Then Run Every Check Release / Prepare and Publication Depend On."""
    failures: list[str] = []

    def step(label: str, action: Callable[[], object], hint: str = "") -> bool:
        print(f"== Preflight / {label}", flush=True)
        try:
            action()
        except FAILURES as error:
            failures.append(f"{label}: {hint or error}")
            return False
        return True

    with worktree(directory) as tree:
        settings = policy.policy(tree, "release")
        current = context.release_tag(tree)[1:]
        if not step("Prepare", lambda: prepare.refresh(tree, settings, prepare.planned_version(tree, current))):
            return failures
        step("Version", lambda: version.run(tree, "check"))
        for lock in prepare.tracked(tree, "*uv.lock"):
            command = ["uv", "lock", "--check", "--project", Path(lock).parent.as_posix()]
            step(f"Lock {lock}", lambda command=command: policy.execute(tree, command), stale(lock))
        for lock in prepare.tracked(tree, "*Cargo.lock"):
            command = [
                "cargo",
                "metadata",
                "--locked",
                "--format-version",
                "1",
                "--manifest-path",
                (Path(lock).parent / "Cargo.toml").as_posix(),
            ]
            step(f"Lock {lock}", lambda command=command: policy.execute(tree, command, capture=True), stale(lock))
        findings: list[str] = []
        step("Manifests", lambda: findings.extend(registry.crates.audit(tree)))
        failures.extend(f"Manifests: {finding}" for finding in findings)
        # A Git Pin Refuses the Strict Dry Run Only; Packaging Without Verification Still Proves Every Manifest.
        if step("Pins", lambda: pins.check(tree)):
            step("Crates", lambda: registry.crates.rehearse(tree, skip_published=False))
        else:
            for name in registry.crates.publication_order(registry.crates.inventory(tree)):
                command = ["cargo", "package", "--no-verify", "--exclude-lockfile", "--allow-dirty", "--package", name]
                step(
                    f"Package {name}",
                    lambda command=command: policy.execute(tree, command),
                    f"{name} does not package; fix the error above",
                )
        step("Python SDK", lambda: wheel.build(tree, "dev"))
    return failures


def tag_gate(directory: Path, number: int) -> list[str]:
    """Run the Release / Review and Release / Tag Evidence Checks Read-Only Against One Merged Release Review."""
    settings = policy.policy(directory, "release")
    if settings.get("provider") != "github":
        return ["Tag gate: the emulation reads GitHub evidence; run Release / Tag on GitLab"]
    failures: list[str] = []
    with environment(GH_TOKEN=os.environ.get("GH_TOKEN") or _gh_token()):
        pull = registry.provider.document(settings, f"pulls/{number}") or {}
        branch = pull.get("head", {}).get("ref", "")
        sha = pull.get("merge_commit_sha") or ""
        if (
            pull.get("merged") is not True
            or pull.get("base", {}).get("ref") != "dev"
            or not branch.startswith(registry.reviews.RELEASE_BRANCH + "v")
        ):
            return [f"Review: #{number} is not a merged release/v<version> review into dev; pass the number of one"]
        name = branch.removeprefix(registry.reviews.RELEASE_BRANCH)
        policy.execute(directory, ["git", "fetch", "--quiet", "--no-tags", "origin", sha])
        found = registry.reviews.find(settings, sha, branch)
        if found is None or found["number"] != number:
            failures.append(f"Review lookup: merge commit {sha} does not identify #{number} alone; Release / Tag would refuse it")
        existing = registry.provider.remote_tag(settings, name)
        if existing not in (None, sha):
            failures.append(f"Tag: {name} already identifies {existing}; a release tag never moves")
        packages, builds = _review_evidence(directory)
        evidence = {
            "GITHUB_ACTIONS": "true",
            "GITHUB_REPOSITORY": settings["repository"],
            "GITHUB_REF": "refs/tags/" + name,
            # The Ruleset Protects the Tag Once It Exists; the Emulation Asserts Only the Review Evidence.
            "GITHUB_REF_PROTECTED": "true",
            "REVIEW_RELEASE": "true",
            "REVIEW_PACKAGES": packages,
            "REVIEW_BUILDS": builds,
        }
        with environment(**evidence):
            try:
                dokime_tools.ci.review.lookup(sha)
            except FAILURES as error:
                failures.append(f"CI evidence: {error}")
        if found is not None:
            try:
                registry.reviews.rehearsed(directory, settings, found, sha)
            except FAILURES as error:
                failures.append(f"Rehearsal evidence: {error}")
    return failures


def stale(lock: str) -> str:
    return f"{lock} fails its lock check after Prepare; if it is stale, add its refresh to [release] lock_commands in tools/project.toml"


@contextmanager
def worktree(directory: Path) -> Generator[Path]:
    """A Detached Worktree of HEAD Under target/, Removed Afterwards; Its Cargo Builds Share This Checkout's Target."""
    (directory / "target").mkdir(exist_ok=True)
    tree = Path(tempfile.mkdtemp(prefix="release-preflight-", dir=directory / "target")) / "tree"
    policy.execute(directory, ["git", "worktree", "add", "--quiet", "--detach", str(tree), "HEAD"])
    try:
        with environment(CARGO_TARGET_DIR=os.environ.get("CARGO_TARGET_DIR") or str(directory / "target")):
            yield tree
    finally:
        for command in (["git", "worktree", "remove", "--force", str(tree)], ["git", "worktree", "prune"]):
            subprocess.run(command, cwd=directory, check=False, creationflags=spawn_flags())
        shutil.rmtree(tree.parent, ignore_errors=True)


@contextmanager
def environment(**values: str) -> Generator[None]:
    previous = {}
    for name in values:
        previous[name] = os.environ.get(name)
    os.environ.update(values)
    try:
        yield
    finally:
        for name, value in previous.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--review", type=int, help="Emulate the Tag Gate for This Merged Release Review Instead")
    options = parser.parse_args(arguments)
    directory = policy.root()
    try:
        failures = packaging(directory) if options.review is None else tag_gate(directory, options.review)
    except FAILURES as error:
        failures = [f"Preflight: {error}"]
    for failure in failures:
        print(failure, file=sys.stderr)
    print(f"Preflight: {len(failures)} failure(s)" if failures else "Preflight: ready to release", flush=True)
    return int(bool(failures))


# === Private Functions === #


def _review_evidence(directory: Path) -> tuple[str, str]:
    """The Package and Build Evidence Release / Review Requires, Read From the Release Workflow."""
    text = (directory / ".github/workflows" / context.RELEASE_WORKFLOW).read_text(encoding="utf-8")
    values = []
    for name in ("REVIEW_PACKAGES", "REVIEW_BUILDS"):
        match = re.search(rf"^\s*{name}: '(\[.*\])'\s*$", text, re.MULTILINE)
        if match is None:
            raise policy.ToolError(f"Tag gate: {context.RELEASE_WORKFLOW} declares no {name} for Release / Review")
        values.append(match.group(1))
    return values[0], values[1]


def _gh_token() -> str:
    result = subprocess.run(["gh", "auth", "token"], check=False, capture_output=True, text=True, creationflags=spawn_flags())
    if result.returncode or not result.stdout.strip():
        raise policy.ToolError("Tag gate: set GH_TOKEN or sign in with gh auth login to read the review evidence")
    return result.stdout.strip()


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
