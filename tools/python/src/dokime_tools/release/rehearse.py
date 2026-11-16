# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Rehearse Operations."""

# === Imports === #

# Standard Library Imports
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from . import context, dry, registry

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def rehearse(
    directory: Path,
    settings: dict,
    *,
    target: str | None = None,
    profile: str = "release",
    stage_only: bool = False,
    channel: str = "stable",
) -> None:
    """Verify Selected Local Packages or Every Hosted Target Without Publishing Credentials."""
    hosted = context.continuous_integration()
    if profile not in {"dev", "release"}:
        raise policy.ToolError("Rehearse Release: rehearsal profile must be dev or release")
    if hosted and (profile != "release" or stage_only):
        raise policy.ToolError("Rehearse Release: hosted rehearsal requires complete release-profile verification")
    target = target or "native"
    selected = context.native_target(settings) if target == "native" else target
    if selected != "all" and selected not in settings.get("targets", []):
        raise policy.ToolError(f"Rehearse Release: undeclared release target: {selected}")
    targets = list(settings.get("targets", [])) if selected == "all" else [selected]
    with policy.credential_free(settings, profile=profile):
        context.refuse_overrides(directory)
        if not hosted and not stage_only and targets == [context.native_target(settings)]:
            policy.execute(directory, ["mise", "run", "build", "--", "--language", "python", "--profile", profile])
            policy.execute(directory, ["mise", "run", "build", "--", "--profile", profile])
            policy.execute(directory, ["mise", "run", "package", "--", "--profile", profile, "--target", targets[0]])
        revision = context.revision(directory) if stage_only else None
        dry.dry(directory, settings, profile=profile, require_all=True, source_revision=revision, targets=targets, channel=channel)
        if not stage_only and channel == "stable" and context.task_defined(directory, "release:tag"):
            registry.crates.rehearse(directory, skip_published=False, channel=channel)
        if not stage_only and channel == "stable" and context.task_defined(directory, "release:publish:python"):
            registry.python.publish(directory, settings, dry_run=True)
    print(f"Release {'Staging' if stage_only else 'Rehearsal'} Passed: {', '.join(targets)} ({profile} Profile, {channel} Channel)")
    uncovered = sorted(set(settings.get("targets", [])) - set(targets))
    if uncovered:
        print(f"Targets Not Covered: {', '.join(uncovered)}; Use --target all With Those Packages Available to Rehearse Them")


# === Private Functions === #

# === Entry Point === #
