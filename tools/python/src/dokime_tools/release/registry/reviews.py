# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Registry Reviews Operations."""

# === Imports === #

# Standard Library Imports
import os
import re
import subprocess
import urllib.parse
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.ci.review

from ... import policy
from ...process import spawn_flags
from . import provider

# Type Checking Imports

# === Public Exports === #

# === Constants === #

RELEASE_BRANCH = "release/"

# === Types === #

# === Classes === #

# === Public Functions === #


def merge(settings: dict, branch: str, title: str) -> None:
    """Squash the Release Review Into Dev Immediately and Verify Its Commit."""
    # Find the Release Review
    if settings["provider"] == "github":
        repository = settings.get("repository", os.environ.get("GITHUB_REPOSITORY", ""))
        head = urllib.parse.quote(repository.split("/")[0] + ":" + branch, safe="")
        reviews = provider.pages(settings, f"pulls?state=open&base=dev&head={head}")
    else:
        source = urllib.parse.quote(branch, safe="")
        reviews = provider.pages(settings, f"merge_requests?state=opened&target_branch=dev&source_branch={source}")
    if len(reviews) != 1:
        raise policy.ToolError(f"Merge Release Review: find exactly one open release review for {branch} and merge it again")
    review = reviews[0]

    # Squash the Release Review
    if settings["provider"] == "github":
        result = (
            provider.document(
                settings,
                f"pulls/{review['number']}/merge",
                "PUT",
                {"merge_method": "squash", "commit_title": title, "sha": review["head"]["sha"]},
            )
            or {}
        )
        merged = result.get("merged") is True
        sha = result.get("sha")
        endpoint = f"commits/{sha}"
    else:
        result = (
            provider.document(
                settings,
                f"merge_requests/{review['iid']}/merge",
                "PUT",
                {"sha": review["sha"], "squash": True, "squash_commit_message": title, "should_remove_source_branch": True},
            )
            or {}
        )
        merged = result.get("state") == "merged"
        sha = result.get("squash_commit_sha")
        endpoint = f"repository/commits/{sha}"
    if not merged or not isinstance(sha, str) or not re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", sha):
        raise policy.ToolError(f"Verify Release Review: inspect {branch} and complete its squash merge into dev")

    # Verify the Merge Commit
    commit = provider.document(settings, endpoint) or {}
    revision = commit.get("sha") if settings["provider"] == "github" else commit.get("id")
    message = commit.get("commit", {}).get("message", "") if settings["provider"] == "github" else commit.get("message", "")
    if revision != sha or message.split("\n", 1)[0] != title:
        raise policy.ToolError(f"Verify Release Review: inspect merge commit {sha}; its revision or release headline differs")


def find(settings: dict, sha: str, branch: str) -> dict | None:
    """The Merged Release Review Whose Merge Commit Is This Revision, If Any."""
    if settings["provider"] == "github":
        matches = []
        for matches_item in provider.pages(settings, f"commits/{sha}/pulls"):
            if not matches_item.get("merged_at"):
                continue
            if matches_item.get("merge_commit_sha") != sha:
                continue
            if matches_item.get("base", {}).get("ref") != "dev":
                continue
            if matches_item.get("head", {}).get("ref") != branch:
                continue
            matches.append(matches_item)

        if len(matches) > 1:
            raise policy.ToolError("Verify Release Review: the revision identifies several merged release pull requests")
        return {"head": matches[0]["head"]["sha"], "number": matches[0]["number"]} if matches else None
    matches = []
    for matches_item in provider.pages(settings, f"repository/commits/{sha}/merge_requests"):
        if matches_item.get("state") != "merged":
            continue
        if sha not in {matches_item.get("merge_commit_sha"), matches_item.get("squash_commit_sha"), matches_item.get("sha")}:
            continue
        if matches_item.get("target_branch") != "dev":
            continue
        if matches_item.get("source_branch") != branch:
            continue
        matches.append(matches_item)

    if len(matches) > 1:
        raise policy.ToolError("Verify Release Review: the revision identifies several merged release merge requests")
    return {"head": matches[0]["sha"], "number": matches[0]["iid"]} if matches else None


def rehearsed(directory: Path, settings: dict, review: dict, sha: str) -> None:
    """The Release Review Head Passed Its Rehearsal and Its Tree Is Exactly the Merged Tree."""
    head = review["head"]
    if settings["provider"] == "github":
        pull = provider.document(settings, f"pulls/{review['number']}")
        runs = provider.pages(settings, f"actions/workflows/ci.yml/runs?event=pull_request&head_sha={head}", "workflow_runs")
        try:
            if not isinstance(pull, dict) or pull.get("number") != review["number"] or pull.get("head", {}).get("sha") != head:
                raise ValueError("Verify Release Review: review identity or head differs")
            candidates = []
            for candidates_run in runs:
                if not dokime_tools.ci.review.github_review_run(candidates_run, pull):
                    continue
                candidates.append(candidates_run)

            def jobs_for(identifier: int) -> list[dict]:
                return provider.pages(settings, f"actions/runs/{identifier}/jobs?filter=latest", "jobs")

            def run_for(identifier: int) -> dict:
                return provider.document(settings, f"actions/runs/{identifier}")

            _, jobs = dokime_tools.ci.review.github_validation_run(candidates, jobs_for, run_for)
        except (KeyError, TypeError, ValueError) as error:
            raise policy.ToolError(
                f"Verify Release Review: release PR #{review['number']} head {head} has no green validation: {error}"
            ) from error
        if not any(
            dokime_tools.ci.review.job_named(job, dokime_tools.ci.review.REHEARSAL) and job.get("conclusion") == "success" for job in jobs
        ):
            raise policy.ToolError(
                f"Verify Release Review: release PR #{review['number']} head {head} has no successful {dokime_tools.ci.review.REHEARSAL} check"
            )
        _same_tree(directory, head, sha)
        return
    # The merge request's latest pipeline ran on its head; its rehearsal job must have succeeded there.
    merged_review = provider.document(settings, f"merge_requests/{review['number']}")
    pipeline = merged_review.get("head_pipeline") if isinstance(merged_review, dict) else None
    if not isinstance(pipeline, dict) or pipeline.get("sha") != head or not isinstance(pipeline.get("id"), int):
        raise policy.ToolError(f"Verify Release Review: release MR !{review['number']} has no pipeline for its head {head}")
    pipeline_jobs = provider.pages(settings, f"pipelines/{pipeline['id']}/jobs?include_retried=false")
    jobs = []
    for jobs_job in pipeline_jobs:
        if not dokime_tools.ci.review.job_named(jobs_job, dokime_tools.ci.review.REHEARSAL):
            continue
        jobs.append(jobs_job)

    if not jobs or any(job.get("status") != "success" for job in jobs):
        raise policy.ToolError(
            f"Verify Release Review: release MR !{review['number']} pipeline {pipeline['id']} has no successful {dokime_tools.ci.review.REHEARSAL} job"
        )
    _same_tree(directory, head, sha)


# === Private Functions === #


def _same_tree(directory: Path, head: str, merged: str) -> None:
    revisions = (head, merged)
    if any(not isinstance(revision, str) or not re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", revision) for revision in revisions):
        raise policy.ToolError("Verify Release Review: rehearsal tree verification requires full commit object IDs")
    commands = [["git", "fetch", "--quiet", "--no-tags", "--no-write-fetch-head", "origin", *revisions]]
    commands.extend(["git", "rev-parse", "--verify", revision + "^{tree}"] for revision in revisions)
    outputs = []
    for command in commands:
        try:
            output = subprocess.run(
                command,
                creationflags=spawn_flags(),
                cwd=directory,
                env={**os.environ, "GIT_TERMINAL_PROMPT": "0"},
                capture_output=True,
                text=True,
                timeout=120,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise policy.ToolError("Verify Release Review: could not fetch and resolve the rehearsed and merged Git trees") from error
        if output.returncode:
            raise policy.ToolError("Verify Release Review: could not fetch and resolve the rehearsed and merged Git trees")
        outputs.append(output.stdout.strip())
    trees = outputs[1:]
    if any(not re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", tree) for tree in trees):
        raise policy.ToolError("Verify Release Review: rehearsal tree verification returned no valid tree object IDs")
    if trees[0] != trees[1]:
        raise policy.ToolError(
            "Verify Release Review: the merged tree differs from the rehearsed release review head; update the review and merge again"
        )


# === Entry Point === #
