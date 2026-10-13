# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Require Complete Native Validation For The Proposed Integration Tree"""

# === Imports === #

# Standard Library Imports
import json
import os
import subprocess
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.ci.artifacts import _request, job_id
from dokime_tools.ci.policy import current_event, required_platforms
from dokime_tools.ci.route import verify as verify_route

# Type Checking Imports

# === Public Exports === #

# === Constants === #
REPOSITORY = "stokker-technologies/dokime"
ROLES = (
    "check:fast",
    "contracts:prepare:release",
    "build:release",
    "test:integration:rust",
    "test:integration:python",
    "test:integration:javascript",
    "test:acceptance:rust",
    "test:acceptance:python",
    "ci:coverage",
    "test",
)

# === Types === #

# === Classes === #

# === Public Functions === #


def verify(event: dict, needs: dict) -> None:
    if "pull_request" in event:
        verify_route(event, Path.cwd())
    required = ("native", "ci_artifacts_catalog")
    for job in required:
        if needs.get(job, {}).get("result") != "success":
            raise ValueError(f"required native job did not pass: {job}")


def verify_tree(event: dict) -> None:
    roles = ROLES
    for platform in required_platforms(event):
        if platform == "windows":
            job_id("test:system:javascript", platform, completed=True)
        for role in roles:
            job_id(role, platform, completed=True)
    if "pull_request" not in event:
        tested = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True, encoding="utf-8"
        ).strip()
        if tested != os.environ["GITHUB_SHA"]:
            raise ValueError("native gate requires the exact dispatched commit")
        return
    request = event["pull_request"]
    target = request["base"]["ref"]
    if request["base"]["repo"]["full_name"] != REPOSITORY:
        raise ValueError("native gate requires the configured repository")
    current = _request(f"branches/{target}")["commit"]["sha"]
    subprocess.run(["git", "fetch", "--no-tags", "origin", current], check=True)
    proposed = subprocess.check_output(
        ["git", "merge-tree", "--write-tree", current, request["head"]["sha"]],
        text=True,
        encoding="utf-8",
    ).splitlines()[0]
    tested = subprocess.check_output(
        ["git", "rev-parse", "HEAD^{tree}"], text=True, encoding="utf-8"
    ).strip()
    if proposed != tested:
        raise ValueError("target advancement changed the validated integration tree")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    event = current_event()
    verify(event, json.loads(os.environ["NEEDS_RESULTS"]))
    verify_tree(event)
