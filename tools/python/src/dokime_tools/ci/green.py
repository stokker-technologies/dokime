"Require Green CI for the Exact Release Commit Before Any Publication."

# === Imports === #

# Standard Library Imports
import operator
import os
import re
import time
from collections.abc import Callable

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.ci import review

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def green(sha: str, pages: Callable[..., list[dict]]) -> bool:
    """Select the Latest Dev Push and Its Required Gate, Never a Prior Run or PR."""
    if re.fullmatch(r"[0-9a-f]{40}(?:[0-9a-f]{24})?", sha) is None:
        raise ValueError("Verify Release Evidence: invalid target revision")
    runs = pages(f"actions/workflows/ci.yml/runs?branch=dev&event=push&head_sha={sha}", "workflow_runs")
    runs_items = []
    for runs_run in runs:
        if runs_run.get("head_sha") != sha:
            continue
        if runs_run.get("event") != "push":
            continue
        if runs_run.get("head_branch") != "dev":
            continue
        runs_items.append(runs_run)
    runs = runs_items
    if not runs:
        return False
    run = max(runs, key=operator.itemgetter("id"))
    if run.get("status") != "completed":
        return False
    if run.get("conclusion") != "success":
        raise ValueError("Verify Release Evidence: target commit CI failed")

    checks = pages(f"commits/{sha}/check-runs?filter=latest", "check_runs")
    gates = []
    for gates_check in checks:
        if gates_check.get("name") != "Check / CI / Required":
            continue
        if gates_check.get("app", {}).get("slug") != "github-actions":
            continue
        if gates_check.get("check_suite", {}).get("id") != run["check_suite_id"]:
            continue
        gates.append(gates_check)

    if len(gates) != 1 or gates[0].get("status") != "completed":
        return False
    if gates[0].get("conclusion") != "success":
        raise ValueError("Verify Release Evidence: target commit required gate failed")

    # Honor Additional Required Contexts In The Owner's Main Ruleset.
    rules = pages("rules/branches/main")
    required = []
    for required_rule in rules:
        if required_rule.get("type") != "required_status_checks":
            continue
        required.extend(required_rule["parameters"]["required_status_checks"])

    statuses = pages(f"commits/{sha}/statuses")
    for context in required:
        matches = []
        for matches_check in checks:
            if matches_check.get("name") != context["context"]:
                continue
            if not (not context.get("integration_id") or matches_check.get("app", {}).get("id") == context["integration_id"]):
                continue
            matches.append(matches_check)

        if matches:
            check = max(matches, key=operator.itemgetter("id"))
            if check.get("status") != "completed":
                return False
            if check.get("conclusion") != "success":
                raise ValueError("target commit required check failed: " + context["context"])
            continue
        legacy = []
        for legacy_status in statuses:
            if legacy_status.get("context") != context["context"]:
                continue
            if context.get("integration_id"):
                continue
            legacy.append(legacy_status)

        if not legacy:
            return False
        status = max(legacy, key=operator.itemgetter("id"))
        if status.get("state") == "pending":
            return False
        if status.get("state") != "success":
            raise ValueError("target commit required status failed: " + context["context"])
    return True


def wait(
    sha: str, pages: Callable[..., list[dict]], *, sleep: Callable[[float], None] = time.sleep, clock: Callable[[], float] = time.monotonic
) -> None:
    deadline = clock() + 1800
    while not green(sha, pages):
        if clock() >= deadline:
            raise ValueError("Verify Release Evidence: target commit required CI is missing or pending; rerun Release after CI succeeds")
        print("Waiting For Target Commit CI", flush=True)
        sleep(10)
    print(f"Required CI Is Green For {sha}", flush=True)


# === Private Functions === #

# === Entry Point === #


def main() -> None:
    sha = review.command("git", "rev-parse", "HEAD")
    expected = os.environ["RELEASE_TARGET_SHA"]
    if sha != expected:
        raise ValueError("Select Release Checkout: Check Out The Target Revision")
    wait(sha, review.pages)


if __name__ == "__main__":
    main()
