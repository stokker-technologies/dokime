# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"""Publish Existing Dev Packages As Immutable Builds And A Rolling Channel."""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import os
import re
import sys
import tomllib
import urllib.parse
from datetime import UTC, datetime
from pathlib import Path

# Third Party Imports
# Local Imports
from . import layout, policy, registry, run

# Type Checking Imports


# === Public Exports === #

# === Constants === #

TAG = "dev-channel"
DEV_TAG = re.compile(rf"v{run.STABLE}-dev\.([0-9]{{14}})\.g[0-9a-f]{{8}}")
WORKFLOW_MARKER = "<!-- dokime-dev-release: .github/workflows/dev-channel.yml -->"

# === Types === #

# === Classes === #

# === Public Functions === #


def publish(directory: Path) -> None:
    settings = policy.policy(directory, "release")
    if settings.get("dev_channel") is not True:
        raise policy.ToolError("release.dev_channel must opt in to channel publication")
    github = settings["provider"] == "github"
    if github:
        _retention(settings)
        if os.environ.get("GITHUB_EVENT_NAME") != "workflow_run":
            raise policy.ToolError("dev channel requires a successful dev CI completion")
        event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8"))["workflow_run"]
        eligible = event["conclusion"] == "success" and event["event"] == "push" and event["head_branch"] == "dev"
        if not eligible or event["head_repository"]["full_name"] != os.environ["GITHUB_REPOSITORY"]:
            raise policy.ToolError("ineligible dev channel CI run")
        sha, identifier, url = event["head_sha"], str(event["id"]), event["html_url"]
    else:
        source = os.environ.get("PIPELINE_SOURCE") or os.environ.get("CI_PIPELINE_SOURCE")
        eligible = source == "push" and os.environ.get("CI_COMMIT_BRANCH") == "dev"
        if not eligible or os.environ.get("CI_DEFAULT_BRANCH") != "dev":
            raise policy.ToolError("GitLab dev channel requires a push to default dev")
        sha, identifier, url = (
            os.environ["CI_COMMIT_SHA"],
            os.environ["CI_PIPELINE_ID"],
            os.environ["CI_PIPELINE_URL"],
        )
    if not re.fullmatch(r"[0-9a-f]{40,64}", sha) or not identifier.isdecimal() or run._revision(directory) != sha:
        raise policy.ToolError("dev channel checkout must match the source CI commit")
    source_run = f"[{identifier}]({url})"
    endpoint = f"releases/tags/{TAG}" if github else f"releases/{TAG}"
    previous = _github_release(settings) if github else registry.document(settings, endpoint, channel=True)
    previous_notes = (previous or {}).get("body" if github else "description", "")
    tag = _dev_tag(policy.policy(directory, "version")["value"], sha, datetime.now(UTC)) if github else TAG
    content = _notes(directory, settings, sha, source_run, previous_notes, tag=tag)
    with policy.credential_free(settings, profile="dev"):
        run._refuse_overrides(directory)
        run.dry(directory, settings, profile="dev", require_all=True, release_notes=content)
    files = layout.uploads(directory, staged=True)
    notes = (directory / run.ASSETS / run.SHARED / "CHANGELOG.md").read_text(encoding="utf-8")
    if github:
        _github(directory, settings, files, sha, identifier, notes, tag=tag)
    else:
        _gitlab(settings, files, sha, identifier, notes)


# === Private Functions === #


def _dev_tag(version: str, sha: str, timestamp: datetime) -> str:
    if re.fullmatch(run.STABLE, version) is None or re.fullmatch(r"[0-9a-f]{40,64}", sha) is None:
        raise policy.ToolError("dev build requires a workspace version and full commit SHA")
    if timestamp.tzinfo is None:
        raise policy.ToolError("dev build timestamp must include a timezone")
    return f"v{version}-dev.{timestamp.astimezone(UTC):%Y%m%d%H%M%S}.g{sha[:8]}"


def _retention(settings: dict) -> int:
    keep = settings.get("dev_release_retention", 20)
    if type(keep) is not int or keep < 1:
        raise policy.ToolError("release.dev_release_retention must be a positive integer")
    return keep


def _retired(releases: list[dict], keep: int) -> list[dict]:
    eligible = []
    for release in releases:
        match = DEV_TAG.fullmatch(release.get("tag_name", ""))
        if (
            match is None
            or release.get("prerelease") is not True
            or release.get("draft") is not False
            or WORKFLOW_MARKER not in (release.get("body") or "").splitlines()
            or release.get("author", {}).get("login") != "github-actions[bot]"
            or release.get("author", {}).get("type") != "Bot"
        ):
            continue
        try:
            datetime.strptime(match[1], "%Y%m%d%H%M%S")
        except ValueError:
            continue
        eligible.append((match[1], release["tag_name"], release))
    eligible.sort(key=lambda item: item[:2], reverse=True)
    return [release for _, _, release in eligible[keep:]]


def _github_release(settings: dict) -> dict | None:
    release = registry.document(settings, f"releases/tags/{TAG}", channel=True)
    if release is not None:
        return release
    # The Tag Lookup Can Omit Drafts; Recover Interrupted Publication By Listing.
    drafts = [item for item in registry.pages(settings, "releases", channel=True) if item.get("tag_name") == TAG]
    if len(drafts) > 1:
        raise policy.ToolError("multiple dev channel releases require owner reconciliation")
    return drafts[0] if drafts else None


def _superseded(release: dict | None, identifier: str, field: str) -> bool:
    previous = re.search(r"(?m)^\| CI Run \| \[([0-9]+)\]", (release or {}).get(field, ""))
    return previous is not None and int(previous[1]) > int(identifier)


def _notes(directory: Path, settings: dict, sha: str, source_run: str, previous: str, *, tag: str = TAG) -> str:
    github = settings["provider"] == "github"
    repository = (settings.get("repository") or os.environ.get("GITHUB_REPOSITORY")) if github else settings["project_id"]
    base = "https://github.com" if github else settings.get("api_url", "https://gitlab.com/api/v4").removesuffix("/api/v4")
    project = f"{base}/{repository}"
    commit = f"{project}/commit/{sha}" if github else f"{project}/-/commit/{sha}"
    subject = policy.execute(directory, ["git", "log", "-1", "--format=%s", sha], capture=True).strip().replace("|", "\\|")
    version = policy.policy(directory, "version")["value"]
    lines = [
        "Unsigned development build. APIs may change or break at any time.",
        "",
        "| | |",
        "|---|---|",
        f"| Commit | [`{sha[:8]}`]({commit}) {subject} |",
        f"| Version | {version} (dev) |",
    ]
    if github:
        reviews = registry.pages(settings, f"commits/{sha}/pulls", channel=True)
        reviews = [
            review
            for review in reviews
            if review.get("merge_commit_sha") == sha and review.get("merged_at") and review.get("base", {}).get("ref") == "dev"
        ]
        title = reviews[0]["title"].replace("|", "\\|") if len(reviews) == 1 else subject
        lines.append(f"| PR | {title} |")
    manifest = directory / "Cargo.toml"
    if manifest.is_file():
        cargo = tomllib.loads(manifest.read_text(encoding="utf-8"))
        dependencies = {
            **cargo.get("workspace", {}).get("dependencies", {}),
            **cargo.get("dependencies", {}),
        }
        for name, dependency in sorted(dependencies.items()):
            if isinstance(dependency, dict) and "git" in dependency:
                url = dependency["git"].removesuffix(".git")
                revision = dependency.get("rev")
                pin = (
                    f"[`{revision[:8]}`]({url}/commit/{revision})" if revision else dependency.get("tag", dependency.get("branch", "HEAD"))
                )
                lines.append(f"| {name.replace('-', ' ').title()} | {pin} (git) |")
    lines.extend(
        [
            f"| Built | {datetime.now(UTC):%Y-%m-%d %H:%M UTC} |",
            f"| CI Run | {source_run} |",
            "",
            "### Use It",
            "",
        ]
    )
    if github:
        lines.extend(
            [
                "Tools through mise:",
                "",
                "```toml",
                f'[tools."github:{repository}"]',
                f'version = "{tag}"',
                "```",
                "",
            ]
        )
    else:
        lines.extend(
            [
                f"Tools are available from the [channel downloads]({project}/-/releases/{TAG}).",
                "",
            ]
        )
    dependency = settings["product"] + ' = { git = "' + project + '", rev = "' + sha + '" }'
    lines.extend(["Libraries through Cargo:", "", "```toml", dependency, "```", ""])
    previous_commit = re.search(r"/commit/([0-9a-f]{40,64})\)", previous)
    if previous_commit and previous_commit[1] != sha:
        revision = previous_commit[1]
        try:
            present = policy.execute(
                directory,
                ["git", "rev-parse", "--verify", "--quiet", revision + "^{commit}"],
                capture=True,
            )
        except policy.ToolError:
            present = ""
        if present.strip() == revision:
            changes = policy.execute(
                directory,
                ["git", "log", "--format=- %s", "-20", f"{revision}..{sha}"],
                capture=True,
            )
            lines.extend(["### Changes Since The Previous Build", "", changes.strip(), ""])
    lines.append("The tag and source archives identify this build; use the commit link for its permanent revision.")
    return "\n".join(lines) + "\n"


def _github(
    directory: Path,
    settings: dict,
    files: list[dict],
    sha: str,
    identifier: str,
    notes: str,
    *,
    tag: str,
) -> None:
    latest = registry.document(
        settings,
        "actions/workflows/ci.yml/runs?branch=dev&event=push&status=success&per_page=1",
        channel=True,
    )
    if latest and any(int(item["id"]) > int(identifier) for item in latest["workflow_runs"]):
        print("dev channel superseded by a newer successful CI run")
        return
    release = _github_release(settings)
    if _superseded(release, identifier, "body"):
        return
    if release and release.get("immutable"):
        raise policy.ToolError("dev-channel assets are immutable; a rolling channel requires an existing mutable release")
    _github_immutable(directory, settings, files, sha, tag, notes)
    if release is None:
        release = registry.document(
            settings,
            "releases",
            "POST",
            {
                "tag_name": TAG,
                "target_commitish": sha,
                "name": "Dev Channel",
                "prerelease": True,
                "draft": True,
                "make_latest": "false",
            },
            channel=True,
        )
    endpoint = f"releases/{release['id']}"
    # Hide The Channel Before Replacing Any Download; Failed Uploads Leave A Draft.
    registry.document(settings, endpoint, "PATCH", {"draft": True}, channel=True)
    # Remove Every Old Download, Including Assets No Longer Declared.
    assets = registry.pages(settings, f"{endpoint}/assets", channel=True)
    for asset in assets:
        registry.request(settings, f"releases/assets/{asset['id']}", "DELETE", channel=True)
    _github_upload(directory, settings, files, endpoint)

    # Move The Channel Once, Only After Every Platform And Checksum Is Uploaded.
    reference = registry.document(settings, f"git/ref/tags/{TAG}", channel=True)
    if reference is None:
        registry.document(settings, "git/refs", "POST", {"ref": f"refs/tags/{TAG}", "sha": sha}, channel=True)
    else:
        registry.document(settings, f"git/refs/tags/{TAG}", "PATCH", {"sha": sha, "force": True}, channel=True)
    registry.document(
        settings,
        endpoint,
        "PATCH",
        {
            "body": notes,
            "name": "Dev Channel",
            "prerelease": True,
            "draft": False,
            "make_latest": "false",
        },
        channel=True,
    )
    for retired in _retired(registry.pages(settings, "releases", channel=True), _retention(settings)):
        registry.request(settings, f"releases/{retired['id']}", "DELETE", channel=True)
        registry.request(settings, f"git/refs/tags/{retired['tag_name']}", "DELETE", channel=True)


def _github_immutable(directory: Path, settings: dict, files: list[dict], sha: str, tag: str, notes: str) -> None:
    if DEV_TAG.fullmatch(tag) is None:
        raise policy.ToolError("invalid immutable dev tag")
    existing = registry.document(settings, f"git/ref/tags/{tag}", channel=True)
    releases = registry.pages(settings, "releases", channel=True)
    if existing is not None or any(item.get("tag_name") == tag for item in releases):
        raise policy.ToolError(f"immutable dev tag already exists: {tag}")
    # Reserve The Tag Without Force Before Creating The Draft; Races Fail Closed.
    registry.document(settings, "git/refs", "POST", {"ref": f"refs/tags/{tag}", "sha": sha}, channel=True)
    release = registry.document(
        settings,
        "releases",
        "POST",
        {
            "tag_name": tag,
            "target_commitish": sha,
            "name": tag,
            "body": notes + "\n" + WORKFLOW_MARKER + "\n",
            "prerelease": True,
            "draft": True,
            "make_latest": "false",
        },
        channel=True,
    )
    if not isinstance(release, dict) or release.get("draft") is not True or release.get("tag_name") != tag:
        raise policy.ToolError("provider did not confirm the immutable dev draft")
    endpoint = f"releases/{release['id']}"
    _github_upload(directory, settings, files, endpoint)
    registry.document(settings, endpoint, "PATCH", {"draft": False, "prerelease": True, "make_latest": "false"}, channel=True)


def _github_upload(directory: Path, settings: dict, files: list[dict], endpoint: str) -> None:
    for item in files:
        path = policy.inside(directory, item["path"])
        data = path.read_bytes()
        if len(data) != item["size"] or hashlib.sha256(data).hexdigest() != item["sha256"]:
            raise policy.ToolError("staged dev asset differs from its manifest")
        name = urllib.parse.quote(item["name"], safe="")
        registry.request(
            settings,
            f"{endpoint}/assets?name={name}",
            "POST",
            data,
            binary=True,
            upload=True,
            channel=True,
        )
    uploaded = registry.pages(settings, f"{endpoint}/assets", channel=True)
    actual = {asset["name"]: asset["size"] for asset in uploaded}
    expected = {item["name"]: item["size"] for item in files}
    if (
        len(uploaded) != len(files)
        or actual != expected
        or "SHA256SUMS" not in actual
        or any(asset.get("state") != "uploaded" for asset in uploaded)
    ):
        raise policy.ToolError("dev upload inventory is incomplete; leaving it drafted")
    for asset in uploaded:
        expected_digest = next(item["sha256"] for item in files if item["name"] == asset["name"])
        if asset.get("digest") != "sha256:" + expected_digest:
            raise policy.ToolError("dev upload digest differs; leaving it drafted")


def _gitlab(settings: dict, files: list[dict], sha: str, identifier: str, notes: str) -> None:
    endpoint = f"releases/{TAG}"
    release = registry.document(settings, endpoint, channel=True)
    if _superseded(release, identifier, "description"):
        return
    if release is None:
        registry.document(
            settings,
            "releases",
            "POST",
            {"tag_name": TAG, "ref": sha, "name": "Dev Channel", "description": notes},
            channel=True,
        )
    # Links Address This Job's Unexpiring Staged Artifacts, Never A Moving Branch.
    links = registry.pages(settings, f"{endpoint}/assets/links", channel=True)
    for link in links:
        registry.document(settings, f"{endpoint}/assets/links/{link['id']}", "DELETE", channel=True)
    for item in files:
        path = urllib.parse.quote(item["path"], safe="/")
        url = os.environ["CI_JOB_URL"] + "/artifacts/raw/" + path
        registry.document(
            settings,
            f"{endpoint}/assets/links",
            "POST",
            {
                "name": item["name"],
                "url": url,
                "direct_asset_path": "/" + item["name"],
                "link_type": "package",
            },
            channel=True,
        )
    registry.document(
        settings,
        endpoint,
        "PUT",
        {"name": "Dev Channel", "description": notes},
        channel=True,
    )


# === Entry Point === #

if __name__ == "__main__":
    try:
        publish(policy.root())
    except (policy.ToolError, OSError, ValueError, KeyError, TypeError) as error:
        print(f"dev-channel: {error}", file=sys.stderr)
        raise SystemExit(1) from error
