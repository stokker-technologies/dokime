# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Finalize Operations."""

# === Imports === #

# Standard Library Imports
import json
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy, temporary
from ..package.changelog import notes
from . import context, layout, registry

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def finalize(directory: Path, settings: dict) -> None:
    """Verify Every Manifest Download And Signature Before Making The Release Visible."""
    root = context.root(directory)
    state = json.loads((root / context.SHARED / context.STATE).read_text(encoding="utf-8"))
    if state.get("sha") != context.revision(directory) or state.get("tag") != context.release_tag(directory):
        raise policy.ToolError("Finalize Release: the staged publication belongs to another revision or version")
    inventory = {item["name"]: item for item in layout.uploads(directory, staged=True)}
    if set(inventory) != set(state["files"]) or any(
        any(entry.get(key) != value for key, value in inventory[name].items()) for name, entry in state["files"].items()
    ):
        raise policy.ToolError("Finalize Release: publication state differs from upload.json")
    downloads_directory = temporary.Directory()
    downloads = downloads_directory.path
    try:
        for item in state["files"].values():
            (downloads / item["name"]).write_bytes(registry.provider.verify_download(settings, item["endpoint"], item))
        registry.provider.verify_signatures(directory, settings, downloads, list(inventory))
    finally:
        downloads_directory.close()
    if registry.provider.remote_tag(settings, state["tag"]) != state["sha"]:
        raise policy.ToolError("Finalize Release: the release tag is absent or identifies another revision")
    if settings["provider"] == "github":
        current = registry.provider.document(settings, f"releases/{state['release_id']}")
        if not isinstance(current, dict) or not current.get("draft") or current.get("tag_name") != state["tag"]:
            raise policy.ToolError("Finalize Release: the release is no longer the staged draft; verify it manually")
        listed = {item["name"] for item in registry.provider.pages(settings, f"releases/{state['release_id']}/assets")}
        if listed != set(inventory):
            raise policy.ToolError("Finalize Release: draft assets differ from upload.json")
        registry.provider.document(settings, f"releases/{state['release_id']}", "PATCH", {"draft": False})
        visible = registry.provider.document(settings, "releases/tags/" + context.quote(state["tag"]))
        if not isinstance(visible, dict) or visible.get("draft") or visible.get("prerelease"):
            raise policy.ToolError("Finalize Release: published release visibility cannot be verified")
        return
    created = registry.provider.document(
        settings,
        "releases",
        "POST",
        {
            "tag_name": state["tag"],
            "ref": state["sha"],
            "name": state["tag"],
            "description": notes(directory, state["tag"], tagged=True),
            "assets": {"links": [item["link"] for item in state["files"].values()]},
        },
    )
    if not isinstance(created, dict) or created.get("tag_name") != state["tag"] or (created.get("commit") or {}).get("id") != state["sha"]:
        raise policy.ToolError("Finalize Release: published release tag identity cannot be verified")
    remote = registry.provider.document(settings, "releases/" + context.quote(state["tag"])) or {}
    if {link.get("name") for link in (remote.get("assets") or {}).get("links", [])} != set(inventory):
        raise policy.ToolError("Finalize Release: release links differ from upload.json")


# === Private Functions === #

# === Entry Point === #
