# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Verify Operations."""

# === Imports === #

# Standard Library Imports
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.package.installation
import dokime_tools.screenshots.run

from .. import policy, temporary
from . import context, layout, registry, staging

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def verify(directory: Path, settings: dict, *, channel: str = "stable") -> None:
    """After Publication: Download Every Published Asset, Match SHA256SUMS And Signatures, Install On This Platform."""
    tag = context.release_tag(directory) if channel == "stable" else channel
    sha = context.revision(directory)
    if registry.provider.remote_tag(settings, tag) != sha:
        raise policy.ToolError(f"Verify Release: {tag} does not identify the checked-out revision")
    inventory = {item["name"]: item for item in layout.uploads(directory)}
    quoted = context.quote(tag)
    if settings["provider"] == "github":
        remote = registry.provider.document(settings, "releases/tags/" + quoted, channel=channel != "stable")
        if not isinstance(remote, dict) or remote.get("draft") is not False or remote.get("prerelease") is not (channel != "stable"):
            raise policy.ToolError("Verify Release: release is not published")
        if remote.get("target_commitish") != sha:
            raise policy.ToolError("Verify Release: published release identifies another revision")
        downloads = {
            item["name"]: f"releases/assets/{item['id']}"
            for item in registry.provider.pages(settings, f"releases/{remote['id']}/assets", channel=channel != "stable")
        }
    else:
        remote = registry.provider.document(settings, "releases/" + quoted)
        if not isinstance(remote, dict):
            raise policy.ToolError("Verify Release: release is not published")
        prefix = f"packages/generic/{context.quote(context.package(settings))}/{quoted}/"
        downloads = {item["name"]: prefix + context.quote(item["name"]) for item in remote["assets"]["links"]}
    if set(downloads) != set(inventory):
        raise policy.ToolError("Verify Release: published downloads differ from upload.json")
    root_directory = temporary.Directory()
    root = root_directory.path
    try:
        for name, item in inventory.items():
            data = registry.provider.verify_download(settings, downloads[name], item, channel=channel != "stable")
            policy.inside(root, name).write_bytes(data)
        staging.checksums(root / context.SUMS, inventory)
        if channel == "stable":
            registry.provider.verify_signatures(directory, settings, root, list(inventory))
        native = context.native_target(settings)
        dokime_tools.package.installation.installed(directory, {**settings, "tag": tag}, root, native)
    finally:
        root_directory.close()
    policy.write_json(directory / "target/state/release-publication.json", {"sha": sha, "tag": tag, "verified": True})


# === Private Functions === #

# === Entry Point === #
