# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Prepare Operations."""

# === Imports === #

# Standard Library Imports
import os
import re
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.screenshots.run

from .. import policy
from . import context, registry, version

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def planned_version(directory: Path, current: str) -> str:
    """The Current Version Before Its First Tag, Else git-cliff's Configured Automatic Version."""
    if not policy.execute(directory, ["git", "tag", "--list", "v" + current], capture=True).strip():
        return current
    # Breaking changes bump the minor and everything else the patch while the version is 0.x, as the cliff.toml [bump] declares.
    command = ["git-cliff", "--config", "tools/config/cliff.toml", "--bumped-version"]
    planned = policy.execute(directory, command, capture=True).strip().removeprefix("v")
    if planned == current:
        since = policy.execute(directory, ["git", "log", "--format=%h %s", f"v{current}..HEAD"], capture=True).strip()
        listing = "\n".join("  " + line for line in since.splitlines()) or "  (no commits)"
        raise policy.ToolError(
            f"Prepare Release: v{current} is released and no Conventional Commit since warrants a new version; commits since:\n{listing}"
        )
    return planned


def prepare(directory: Path, settings: dict) -> None:
    """Bump Every Version Carrier And Regenerate Release Inputs In The Working Tree."""
    if policy.execute(directory, ["git", "status", "--porcelain"], capture=True).strip():
        raise policy.ToolError("Prepare Release: prepare requires clean committed source")
    require_publication(directory, settings)
    current = context.release_tag(directory)[1:]
    planned = planned_version(directory, current)
    if re.fullmatch(context.STABLE, planned) is None:
        raise policy.ToolError(f"Prepare Release: the next version is not a stable version: {planned}")
    version.run(directory, "sync", value=planned)
    policy_version(directory, planned)
    commands = settings.get("lock_commands")
    if commands is None:
        commands = []
        if (directory / "Cargo.toml").is_file():
            # Cold runners start with an empty Cargo home, so the workspace-only update may fetch the index.
            commands.append(["cargo", "update", "--workspace"])
        for lock in tracked(directory, "*package-lock.json"):
            commands.append(["npm", "install", "--package-lock-only", "--ignore-scripts", "--prefix", str(directory / Path(lock).parent)])
        for lock in tracked(directory, "*pnpm-lock.yaml"):
            commands.append(["pnpm", "install", "--lockfile-only", "--ignore-scripts", "--dir", str(directory / Path(lock).parent)])
        for lock in tracked(directory, "*uv.lock"):
            commands.append(["uv", "lock", "--offline", "--project", str(directory / Path(lock).parent)])
    for command in commands:
        policy.execute(directory, policy.command(command, "release.lock_commands"))
    if policy.policy(directory, "screenshots").get("scenes"):
        dokime_tools.screenshots.run.capture(directory)
    print(f"Prepared v{planned}; review the changes, or let Release / Prepare open the release review")


def policy_version(directory: Path, value: str) -> None:
    """Write `[version] value` In The Project Policy, Keeping Every Other Line."""
    path = directory / "tools/project.toml"
    text = path.read_text(encoding="utf-8")
    match = re.search(r"(?ms)(^\[version\]\s*\n)(.*?)(?=^\[|\Z)", text)
    if match is None:
        raise policy.ToolError("Prepare Release: prepare requires [version] policy")
    body = re.sub(r"(?m)^value\s*=.*$", f'value = "{value}"', match.group(2), count=1)
    path.write_text(text[: match.start()] + match.group(1) + body + text[match.end() :], encoding="utf-8")


def require_publication(directory: Path, settings: dict) -> None:
    """Refuse A Release Whose Crates Cannot Publish: A First Publication Needs The Bootstrap Token."""
    if not (directory / "Cargo.toml").is_file():
        return
    missing = []
    for missing_name in registry.crates.inventory(directory):
        if registry.crates.published(missing_name):
            continue
        missing.append(missing_name)

    if not missing:
        return
    available = os.environ.get("CRATES_BOOTSTRAP_AVAILABLE") == "true" or bool(os.environ.get("CARGO_REGISTRY_TOKEN"))
    message = f"crates not yet on crates.io need the bootstrap token for their first publication: {', '.join(missing)}"
    if context.continuous_integration() and not available:
        raise policy.ToolError(message + "; configure the [release] crates_bootstrap_secret environment secret before preparing")
    if not available:
        print("warning: " + message)


def tracked(directory: Path, *patterns: str) -> list[str]:
    """Tracked paths matching Git pathspec patterns, where `*` also crosses directories."""
    listed = policy.execute(directory, ["git", "ls-files", "-z", "--", *patterns], capture=True)
    return sorted(filter(None, listed.split("\0")))


# === Private Functions === #

# === Entry Point === #
