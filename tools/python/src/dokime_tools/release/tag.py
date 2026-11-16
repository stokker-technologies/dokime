# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Tag Operations."""

# === Imports === #

# Standard Library Imports
import os
import subprocess
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..process import spawn_flags
from . import context, git, pins, registry

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def tag(directory: Path, settings: dict) -> None:
    """Tag An Exactly Rehearsed Merged Release Review, Then Fast-Forward main."""
    sha = context.revision(directory)
    event = tag_event(directory, settings, sha)
    pins.check(directory)
    review = registry.reviews.find(settings, sha, registry.reviews.RELEASE_BRANCH + context.release_tag(directory))
    if review is None:
        raise policy.ToolError("Tag Release: the revision has no merged release review")
    if event is not None and (review["number"] != event["number"] or review["head"] != event["head"]["sha"]):
        raise policy.ToolError("Tag Release: the merged release review differs from the GitHub event")
    name = context.release_tag(directory)
    existing = registry.provider.remote_tag(settings, name)
    if existing and existing != sha:
        raise policy.ToolError(f"Tag Release: {name} already identifies another revision; never move a release tag")
    # Only products that render the rehearsal can have rehearsed their review; for them no tag exists without it.
    if context.task_defined(directory, "release:rehearse"):
        registry.reviews.rehearsed(directory, settings, review, sha)
    else:
        registry.reviews._same_tree(directory, review["head"], sha)
        print("note: this product renders no rehearsal; Release / Dry verified the review")
    token = os.environ.get("RELEASE_TOKEN", "")
    if not token:
        raise policy.ToolError("Tag Release: tagging requires the workflow-triggering RELEASE_TOKEN secret")
    # Reject known divergence before any immutable tag or registry publication can occur.
    remote = git.push_remote(settings, token)
    git.run(directory, ["git", "fetch", "--quiet", remote, "refs/heads/main"], token)
    if subprocess.run(
        ["git", "merge-base", "--is-ancestor", "FETCH_HEAD", sha],
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
        creationflags=spawn_flags(),
        cwd=directory,
        check=False,
    ).returncode:
        raise policy.ToolError(f"Tag Release: main is not an ancestor of {name}; it must only ever fast-forward to release tags")
    if existing is None:
        # The release token's tag push starts the tag pipeline; the native job token's would not.
        registry.provider.create_tag(settings, name, sha)
    if registry.provider.remote_tag(settings, name) != sha:
        raise policy.ToolError(f"Tag Release: {name} was not created at {sha}")
    registry.crates.publish(directory)
    # main is the last released commit: a fast-forward to the tag, never a merge or a force.
    git.run(directory, ["git", "push", remote, f"{sha}:refs/heads/main"], token)
    print(f"{name}: tagged at {sha}; main fast-forwarded")
    branch = registry.reviews.RELEASE_BRANCH + name
    if registry.provider.delete_branch(settings, branch):
        print(f"{branch}: deleted after its squash")


def tag_event(directory: Path, settings: dict, sha: str) -> dict | None:
    if settings["provider"] == "gitlab":
        source = os.environ.get("PIPELINE_SOURCE", os.environ.get("CI_PIPELINE_SOURCE"))
        if source != "web" or os.environ.get("CI_COMMIT_BRANCH") != "dev" or os.environ.get("CI_COMMIT_SHA") != sha:
            raise policy.ToolError("Tag Release: GitLab tagging requires a web-started dev release pipeline at the checkout revision")
        return None
    if os.environ.get("GITHUB_EVENT_NAME") not in {"workflow_run", "workflow_dispatch"} or os.environ.get("GITHUB_REF") != "refs/heads/dev":
        raise policy.ToolError("Tag Release: GitHub tagging requires a dev validation completion or dispatch")
    event_sha = os.environ.get("GITHUB_SHA", "")
    if not event_sha:
        raise policy.ToolError("Tag Release: GitHub tagging requires the dev event revision")
    if sha != event_sha:
        policy.execute(directory, ["git", "merge-base", "--is-ancestor", sha, event_sha])
    workflow = f"{settings['repository']}/.github/workflows/{context.RELEASE_WORKFLOW}@"
    if not os.environ.get("GITHUB_WORKFLOW_REF", "").startswith(workflow):
        raise policy.ToolError("Tag Release: GitHub tagging runs only in release.yml")
    merged = registry.reviews.find(settings, sha, registry.reviews.RELEASE_BRANCH + context.release_tag(directory))
    if merged is None:
        raise policy.ToolError("Tag Release: the revision has no merged release review")
    review = registry.provider.document(settings, f"pulls/{merged['number']}")
    if (
        not isinstance(review, dict)
        or review.get("merged") is not True
        or review.get("merge_commit_sha") != sha
        or type(review.get("number")) is not int
        or review.get("base", {}).get("ref") != "dev"
        or review.get("head", {}).get("ref") != registry.reviews.RELEASE_BRANCH + context.release_tag(directory)
        or any(review.get(side, {}).get("repo", {}).get("full_name") != settings["repository"] for side in ("base", "head"))
    ):
        raise policy.ToolError("Tag Release: GitHub review is not the merged release review for this version and revision")
    return review


# === Private Functions === #

# === Entry Point === #
