# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Publish Operations."""

# === Imports === #

# Standard Library Imports
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..package.changelog import notes
from . import context, layout, pins, registry, staging, version

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def publish(directory: Path, settings: dict) -> None:
    """Stage And Publish Exactly The Manifest's Downloads From The Protected Release Tag."""
    tag = context.release_tag(directory)
    sha = context.revision(directory)
    version.run(directory, "check")
    registry.provider.require_tag(directory, tag)
    pins.check(directory)
    context.refuse_overrides(directory)
    targets = list(settings.get("targets", []))
    if not targets:
        raise policy.ToolError("Publish Release: release.targets must declare the required OS/architecture targets")
    root = context.root(directory)
    staging.stage(directory, settings, tag, targets, signed=True)
    files = layout.uploads(directory, staged=True)
    state = {"sha": sha, "tag": tag, "name": tag, "files": {}}
    if settings["provider"] == "github":
        release = registry.provider.draft(settings, tag, sha, notes(directory, tag, tagged=True))
        state["release_id"] = release["id"]
        for item in files:
            data = staging.verified_bytes(directory, item)
            state["files"][item["name"]] = {**item, "endpoint": registry.provider.upload(settings, release["id"], item, data)}
            policy.write_json(root / context.SHARED / context.STATE, state)
    else:
        if registry.provider.document(settings, "releases/" + context.quote(tag)):
            raise policy.ToolError("Publish Release: published release cannot be replaced; use a new patch version")
        registry.provider.verify_duplicate_policy(settings)
        for item in files:
            state["files"][item["name"]] = {
                **item,
                **registry.provider.upload_generic(settings, context.package(settings), tag, item, staging.verified_bytes(directory, item)),
            }
            policy.write_json(root / context.SHARED / context.STATE, state)
    policy.write_json(root / context.SHARED / context.STATE, state)


# === Private Functions === #

# === Entry Point === #
