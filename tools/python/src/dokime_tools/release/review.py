# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Review Operations."""

# === Imports === #

# Standard Library Imports
import base64
import json
import os
import subprocess
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..package.changelog import notes
from ..process import spawn_flags
from . import context, registry, version

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def review(directory: Path, settings: dict) -> None:
    """Commit the Prepared Tree to release/v<version> and Open Its Review Into dev with the Release Token."""
    token = os.environ.get("RELEASE_TOKEN", "")
    if not context.continuous_integration() or not token:
        raise policy.ToolError("Open Release Review: the release review opens from Release / Prepare with the RELEASE_TOKEN secret")
    tag = context.release_tag(directory)
    version.run(directory, "check")
    branch = registry.reviews.RELEASE_BRANCH + tag
    if settings["provider"] == "github":
        os.environ["GH_TOKEN"] = token
    endpoint = "git/ref/heads/" if settings["provider"] == "github" else "repository/branches/"
    if registry.provider.document(settings, endpoint + context.quote(branch)):
        raise policy.ToolError(f"Open Release Review: {branch} already exists; merge or close its review before preparing again")
    title = f"chore(release): prepare {tag}"
    policy.execute(directory, ["git", "add", "--all"])
    commit_review(directory, settings, branch, title)
    body = policy.inside(directory, "target/release-notes.md")
    body.parent.mkdir(parents=True, exist_ok=True)
    body.write_text(notes(directory, tag, tagged=False), encoding="utf-8")
    if settings["provider"] == "github":
        command = ["gh", "pr", "create", "--base", "dev", "--head", branch, "--title", title, "--body-file", str(body)]
        environment = {**os.environ, "GH_TOKEN": token}
    else:
        command = [
            "glab",
            "mr",
            "create",
            "--source-branch",
            branch,
            "--target-branch",
            "dev",
            "--title",
            title,
            "--description",
            body.read_text(encoding="utf-8"),
            "--squash-before-merge",
            "--remove-source-branch",
            "--yes",
        ]
        environment = {**os.environ, "GITLAB_TOKEN": token}
    if subprocess.run(
        command,
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
        creationflags=spawn_flags(),
        cwd=directory,
        env=environment,
        check=False,
    ).returncode:
        raise policy.ToolError(f"Open Release Review: opening the release review failed; {branch} is pushed and can be reviewed by hand")
    registry.reviews.merge(settings, branch, title)


def commit_review(directory: Path, settings: dict, branch: str, title: str) -> None:
    """Create the Preparation Commit Through the Provider API, Signed by the Provider."""
    head = context.revision(directory)
    changed = policy.execute(directory, ["git", "diff", "--cached", "--name-status", "--no-renames", "-z", "HEAD"], capture=True)
    entries = changed.rstrip("\0").split("\0") if changed else []
    additions, deletions, actions = [], [], []
    for status, name in zip(entries[::2], entries[1::2], strict=True):
        if status == "D":
            deletions.append({"path": name})
            actions.append({"action": "delete", "file_path": name})
            continue
        path = policy.inside(directory, name)
        if status not in {"A", "M"} or path.is_symlink() or not path.is_file():
            raise policy.ToolError(f"Open Release Review: unsupported release file change: {status} {name}")
        content = base64.b64encode(path.read_bytes()).decode("ascii")
        additions.append({"path": name, "contents": content})
        actions.append({"action": "create" if status == "A" else "update", "file_path": name, "content": content, "encoding": "base64"})
    if settings["provider"] == "github":
        registry.provider.document(settings, "git/refs", "POST", {"ref": "refs/heads/" + branch, "sha": head})
        payload = {
            "query": "mutation($input: CreateCommitOnBranchInput!) { createCommitOnBranch(input: $input) { commit { oid signature { isValid } } } }",
            "variables": {
                "input": {
                    "branch": {
                        "repositoryNameWithOwner": settings.get("repository", os.environ.get("GITHUB_REPOSITORY")),
                        "branchName": branch,
                    },
                    "expectedHeadOid": head,
                    "message": {"headline": title},
                    "fileChanges": {"additions": additions, "deletions": deletions},
                }
            },
        }
        response = json.loads(registry.provider.request(settings, "graphql", "POST", json.dumps(payload).encode(), graphql=True) or b"{}")
        if response.get("errors"):
            raise policy.ToolError("createCommitOnBranch failed: " + json.dumps(response["errors"]))
        commit = response.get("data", {}).get("createCommitOnBranch", {}).get("commit", {})
        verified = (commit.get("signature") or {}).get("isValid")
    else:
        # start_sha creates the new branch and its commit together; never force an existing branch.
        commit = registry.provider.document(
            settings,
            "repository/commits",
            "POST",
            {
                "branch": branch,
                "start_sha": head,
                "commit_message": title,
                "actions": actions,
                "allow_empty": True,
            },
        )
        signature = registry.provider.document(settings, "repository/commits/" + context.quote(commit["id"]) + "/signature")
        verified = signature and signature.get("verification_status") in {"verified", "verified_system"}
        if not verified:
            registry.provider.request(settings, "repository/branches/" + context.quote(branch), "DELETE")
    if not verified:
        raise policy.ToolError(
            "Open Release Review: preparation API commit is not verified; GitLab requires web-based commit signing enabled"
        )


# === Private Functions === #

# === Entry Point === #
