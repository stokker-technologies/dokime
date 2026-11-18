"Find a Green Review of This Exact Tree and Restore Its Package Artifacts."

# === Imports === #

# Standard Library Imports
import http.client
import io
import json
import operator
import os
import re
import subprocess
import tomllib
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from collections.abc import Callable
from datetime import UTC, datetime
from pathlib import Path
from typing import IO, Any

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #


OID = re.compile(r"[0-9a-f]{40}(?:[0-9a-f]{24})?")

# === Types === #

# === Classes === #


class ProviderRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(
        self, req: urllib.request.Request, fp: IO[bytes], code: int, msg: str, headers: http.client.HTTPMessage, newurl: str
    ) -> urllib.request.Request | None:
        redirected = super().redirect_request(req, fp, code, msg, headers, newurl)
        origin = urllib.parse.urlsplit(req.full_url)
        destination = urllib.parse.urlsplit(newurl)
        if destination.scheme != "https":
            raise ValueError("Verify Release Evidence: insecure artifact redirect")
        if redirected is not None and (origin.scheme, origin.netloc) != (destination.scheme, destination.netloc):
            redirected.remove_header("PRIVATE-TOKEN")
        return redirected


# === Public Functions === #


def command(*args: str) -> str:
    return subprocess.run(args, check=True, capture_output=True, text=True, timeout=120, creationflags=spawn_flags()).stdout.strip()


def request(endpoint: str, *, binary: bool = False) -> Any:
    if os.environ.get("GITHUB_ACTIONS"):
        return json.loads(command("gh", "api", f"repos/{os.environ['GITHUB_REPOSITORY']}/{endpoint}"))
    token = os.environ["RELEASE_TOKEN"]
    if not token:
        raise ValueError("Verify Release Evidence: provider token unavailable")
    base = os.environ["CI_API_V4_URL"]
    project = urllib.parse.quote(os.environ["CI_PROJECT_ID"], safe="")
    native = urllib.request.Request(f"{base}/projects/{project}/{endpoint}", headers={"PRIVATE-TOKEN": token})
    response = urllib.request.build_opener(ProviderRedirect()).open(native, timeout=60)
    try:
        data = response.read()
    finally:
        response.close()
    return data if binary else json.loads(data)


def pages(endpoint: str, key: str | None = None) -> list[dict]:
    values = []
    for page in range(1, 101):
        separator = "&" if "?" in endpoint else "?"
        response = request(f"{endpoint}{separator}per_page=100&page={page}")
        batch = response[key] if key else response
        if not isinstance(batch, list):
            raise TypeError("Verify Release Evidence: invalid provider page")
        values.extend(batch)
        if len(batch) < 100:
            return values
    raise ValueError("Verify Release Evidence: provider pagination limit")


def same_tree(head: str, tree: str) -> bool:
    if not isinstance(head, str) or OID.fullmatch(head) is None:
        return False
    command("git", "fetch", "--quiet", "--no-tags", "--no-write-fetch-head", "origin", head)
    return command("git", "rev-parse", "--verify", head + "^{tree}") == tree


def github_review_run(run: dict, review: dict) -> bool:
    if run.get("head_sha") != review["head"]["sha"]:
        return False
    associations = run.get("pull_requests")
    if associations:
        return any(pr.get("number") == review["number"] for pr in associations)
    # GitHub Can Drop The Association After The Merged Source Branch Is Deleted.
    repository = review["head"].get("repo", {}).get("id")
    return (
        associations == []
        and isinstance(repository, int)
        and repository > 0
        and run.get("event") == "pull_request"
        and run.get("head_branch") == review["head"]["ref"]
        and run.get("head_repository", {}).get("id") == repository
        and run.get("repository", {}).get("id") == review.get("base", {}).get("repo", {}).get("id")
        and isinstance(run.get("repository", {}).get("id"), int)
    )


def github_validation_run(runs: list[dict], jobs_for: Callable[[int], list[dict]]) -> tuple[dict, list[dict]]:
    for run in sorted(runs, key=operator.itemgetter("id"), reverse=True):
        if run.get("display_title") in ("CI / Closed Review", "CI / Draft Review"):
            continue
        if run.get("display_title") != "CI / Validation" or run.get("status") != "completed" or run.get("conclusion") != "success":
            raise ValueError("Verify Release Evidence: review run is not green validation")
        jobs = jobs_for(run["id"])
        gates = []
        for gates_job in jobs:
            if gates_job.get("name") != "Check / CI / Required":
                continue
            gates.append(gates_job)

        if len(gates) != 1 or gates[0].get("conclusion") != "success":
            raise ValueError("Verify Release Evidence: review required gate is not green")
        if not jobs or any(job.get("conclusion") not in ("success", "skipped") for job in jobs):
            raise ValueError("Verify Release Evidence: review jobs are not green")
        return run, jobs
    raise ValueError("Verify Release Evidence: review has no validation run")


def github_merged_reviews(sha: str) -> list[dict]:
    selected_items = []
    for selected_review in pages(f"commits/{sha}/pulls"):
        if not selected_review.get("merged_at"):
            continue
        if selected_review.get("merge_commit_sha") != sha:
            continue
        if selected_review.get("base", {}).get("ref") != "dev":
            continue
        selected_items.append(selected_review)
    return selected_items


def github_release_mode() -> dict[str, str]:
    sha = command("git", "rev-parse", "HEAD")
    if sha != os.environ["GITHUB_SHA"]:
        raise ValueError("Verify Release Evidence: release decision checkout differs from event revision")
    fallback = {"mode": "prepare" if os.environ["GITHUB_EVENT_NAME"] == "workflow_dispatch" else "skip", "sha": ""}
    with open("tools/project.toml", "rb") as stream:
        version = tomllib.load(stream)["version"]["value"]
    tag = f"v{version}"
    if command("git", "ls-remote", "--tags", "origin", f"refs/tags/{tag}"):
        return fallback
    repository = os.environ["GITHUB_REPOSITORY"]
    head = urllib.parse.quote(f"{repository.split('/')[0]}:release/{tag}", safe="")
    reviews = []
    for reviews_review in pages(f"pulls?state=closed&base=dev&head={head}"):
        if not reviews_review.get("merged_at"):
            continue
        if reviews_review.get("base", {}).get("ref") != "dev":
            continue
        if reviews_review.get("head", {}).get("ref") != f"release/{tag}":
            continue
        if not all(reviews_review.get(side, {}).get("repo", {}).get("full_name") == repository for side in ("base", "head")):
            continue
        reviews.append(reviews_review)

    if not reviews:
        return fallback
    review = max(reviews, key=operator.itemgetter("merged_at"))
    target = review.get("merge_commit_sha", "")
    if OID.fullmatch(target) is None:
        return fallback
    try:
        command("git", "merge-base", "--is-ancestor", target, sha)
    except subprocess.CalledProcessError:
        return fallback
    target_version = tomllib.loads(command("git", "show", f"{target}:tools/project.toml"))["version"]["value"]
    if target_version != version:
        return fallback
    if os.environ["GITHUB_EVENT_NAME"] == "workflow_run":
        with open(os.environ["GITHUB_EVENT_PATH"], encoding="utf-8") as stream:
            completed = json.load(stream)["workflow_run"]
        if (
            completed.get("head_sha") != review["head"]["sha"]
            or completed.get("event") != "pull_request"
            or completed.get("conclusion") != "success"
        ):
            return fallback
    return {"mode": "tag", "sha": target}


def lookup() -> dict[str, str]:
    sha = command("git", "rev-parse", "HEAD")
    tree = command("git", "rev-parse", "HEAD^{tree}")
    if OID.fullmatch(tree) is None:
        raise ValueError("Verify Release Evidence: invalid checkout tree")
    github = bool(os.environ.get("GITHUB_ACTIONS"))
    if github:
        reviews = github_merged_reviews(sha)
    else:
        reviews = []
        for reviews_review in pages(f"repository/commits/{sha}/merge_requests"):
            if reviews_review.get("state") != "merged":
                continue
            if sha not in (reviews_review.get("merge_commit_sha"), reviews_review.get("squash_commit_sha")):
                continue
            if reviews_review.get("target_branch") != "dev":
                continue
            reviews.append(reviews_review)

    if len(reviews) != 1:
        raise ValueError("Verify Release Evidence: no unambiguous merged review")
    review = reviews[0]
    head = review["head"]["sha"] if github else review["sha"]
    branch = review["head"]["ref"] if github else review["source_branch"]
    if not same_tree(head, tree):
        raise ValueError("Verify Release Evidence: review tree differs")
    profile = "release" if branch.startswith("release/v") and os.environ.get("REVIEW_RELEASE") == "true" else "dev"
    tag = os.environ.get("GITHUB_REF", "").startswith("refs/tags/") or bool(os.environ.get("CI_COMMIT_TAG"))
    if tag and profile != "release":
        raise ValueError("Verify Release Evidence: tag requires release-profile evidence")
    required = json.loads(os.environ.get("REVIEW_PACKAGES", "[]"))
    preparation = os.environ.get("GITHUB_EVENT_NAME") == "workflow_dispatch" if github else os.environ.get("CI_PIPELINE_SOURCE") == "web"
    if tag or preparation:
        required += json.loads(os.environ.get("REVIEW_BUILDS", "[]"))
    if github:
        runs = pages(f"actions/workflows/ci.yml/runs?event=pull_request&head_sha={head}", "workflow_runs")
        runs_items = []
        for runs_run in runs:
            if not (github_review_run(runs_run, review)):
                continue
            runs_items.append(runs_run)
        runs = runs_items

        def jobs_for(identifier: int) -> list[dict]:
            return pages(f"actions/runs/{identifier}/jobs?filter=latest", "jobs")

        run, jobs = github_validation_run(runs, jobs_for)
        artifacts = pages(f"actions/runs/{run['id']}/artifacts", "artifacts")
        for name in required:
            matches = []
            for matches_artifact in artifacts:
                if matches_artifact.get("name") != name.replace("PROFILE", profile):
                    continue
                matches.append(matches_artifact)

            if len(matches) != 1 or matches[0].get("expired") is not False:
                raise ValueError("Verify Release Evidence: review package unavailable")
    else:
        review = request(f"merge_requests/{review['iid']}")
        run = review["head_pipeline"]
        if run.get("sha") != head or run.get("status") != "success" or run.get("source") != "merge_request_event":
            raise ValueError("Verify Release Evidence: review pipeline is not green")
        jobs = pages(f"pipelines/{run['id']}/jobs?include_retried=false")
        if not jobs or any(job.get("status") not in ("success", "skipped", "manual") for job in jobs):
            raise ValueError("Verify Release Evidence: review jobs are not green")
        for name in required:
            matches = []
            for matches_job in jobs:
                if matches_job.get("name") != name:
                    continue
                matches.append(matches_job)

            if len(matches) != 1 or matches[0].get("status") != "success" or not matches[0].get("artifacts_file", {}).get("filename"):
                raise ValueError("Verify Release Evidence: review package unavailable")
            expiry = matches[0].get("artifacts_expire_at")
            if expiry and datetime.fromisoformat(expiry) <= datetime.now(UTC):
                raise ValueError("Verify Release Evidence: review package expired")
    if not isinstance(run["id"], int) or run["id"] <= 0:
        raise ValueError("Verify Release Evidence: invalid review run ID")
    if tag and not any(
        job.get("name", "").endswith("Release / Rehearsal") and job.get("conclusion" if github else "status") == "success" for job in jobs
    ):
        raise ValueError("Verify Release Evidence: release review has no successful rehearsal")
    protected = os.environ.get("GITHUB_REF_PROTECTED") == "true" or os.environ.get("CI_COMMIT_REF_PROTECTED") == "true"
    if tag and not protected:
        raise ValueError("Verify Release Evidence: release tag must be protected")
    version = branch.removeprefix("release/") if profile == "release" else f"dev.{run['id']}+{head[:8]}"
    if re.fullmatch(r"(?:v\d+\.\d+\.\d+|dev\.\d+\+[0-9a-f]{8})", version) is None:
        raise ValueError("Verify Release Evidence: invalid review version")
    return {
        "validated": "true",
        "review_run": str(run["id"]),
        "review_profile": profile,
        "review_sha": head,
        "review_version": version,
        "stable_release": str(bool(tag and protected)).lower(),
    }


def restore() -> None:
    if os.environ.get("validated") != "true":
        return
    run = os.environ["review_run"]
    if not run.isdecimal():
        raise ValueError("Verify Release Evidence: invalid review pipeline")
    jobs = pages(f"pipelines/{run}/jobs?include_retried=false")
    for name in json.loads(os.environ["REVIEW_PACKAGES"]) + json.loads(os.environ.get("REVIEW_BUILDS", "[]")):
        matches = []
        for matches_job in jobs:
            if matches_job.get("name") != name:
                continue
            if matches_job.get("status") != "success":
                continue
            matches.append(matches_job)

        if len(matches) != 1:
            raise ValueError("Verify Release Evidence: review package producer unavailable")
        data = request(f"jobs/{matches[0]['id']}/artifacts", binary=True)
        if not isinstance(data, bytes):
            raise TypeError("Download Review Artifacts: The Provider Did Not Return Archive Bytes")
        archive = zipfile.ZipFile(io.BytesIO(data))
        try:
            for member in archive.infolist():
                path = Path(member.filename)
                parent = member.is_dir() and member.filename in ("target/", "target/package/")
                if (
                    path.is_absolute()
                    or ".." in path.parts
                    or "\\" in member.orig_filename
                    or (member.external_attr >> 16) & 0o170000 == 0o120000
                ):
                    raise ValueError("Verify Release Evidence: unsafe package archive path")
                if not parent and not member.filename.startswith(("target/package/", "target/ci/")):
                    raise ValueError("Verify Release Evidence: unexpected package archive path")
            archive.extractall()
        finally:
            archive.close()
    print(f"Reusing packages from review pipeline {run}")


# === Private Functions === #

# === Entry Point === #


def main() -> None:
    print("::group::Review / Release", flush=True)
    try:
        if os.environ.get("REVIEW_RESTORE") == "true":
            restore()
        else:
            result = {
                "validated": "false",
                "review_run": "",
                "review_profile": "",
                "review_sha": "",
                "review_version": "",
                "stable_release": "false",
            }
            try:
                result = lookup()
            except Exception as error:
                if os.environ.get("REVIEW_REQUIRED") == "true":
                    raise ValueError("Verify Release Evidence: required release review evidence unavailable") from error
                # Never print provider responses, commands or credentials on lookup failure.
                print(f"Review evidence unavailable ({type(error).__name__}); running the full pipeline")
            if not os.environ.get("GITHUB_ACTIONS"):
                version = (
                    os.environ.get("CI_COMMIT_TAG")
                    or f"dev.{os.environ.get('CI_PIPELINE_ID', '')}+{os.environ.get('CI_COMMIT_SHORT_SHA', '')}"
                )
                result["DEPLOY_EXPECTED_VERSION"] = result.get("review_version") or version
            output = os.environ.get("GITHUB_OUTPUT", "target/ci/validated.env")
            Path(output).parent.mkdir(parents=True, exist_ok=True)
            with open(output, "a", encoding="utf-8", newline="\n") as stream:
                stream.write("".join(f"{key}={value}\n" for key, value in result.items()))
            print(f"validated={result['validated']} review_run={result['review_run']}")
    finally:
        print("::endgroup::", flush=True)


if __name__ == "__main__":
    main()
