# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Registry Provider Operations."""

# === Imports === #

# Standard Library Imports
import base64
import email.utils
import hashlib
import http.client
import json
import os
import re
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import IO, Any

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ... import policy
from .. import notices
from . import crates

# Type Checking Imports

# === Public Exports === #

# === Constants === #

UPLOAD_MAX_TIMEOUT_S = 1800

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
            raise policy.ToolError("Read Release Provider: release download redirect must use HTTPS")
        if redirected is not None and (origin.scheme, origin.netloc) != (destination.scheme, destination.netloc):
            for name in ["Authorization", "PRIVATE-TOKEN", "JOB-TOKEN"]:
                redirected.remove_header(name)
        return redirected


# === Public Functions === #


def request(
    settings: dict,
    endpoint: str,
    method: str = "GET",
    data: bytes | None = None,
    *,
    binary: bool = False,
    upload: bool = False,
    graphql: bool = False,
    channel: bool = False,
) -> bytes | None:
    provider = settings["provider"]
    if provider == "github":
        base = os.environ.get("GITHUB_API_URL", settings.get("api_url", "https://api.github.com"))
        repository = settings.get("repository", os.environ.get("GITHUB_REPOSITORY"))
        token = os.environ.get("GH_TOKEN", os.environ.get("GITHUB_TOKEN"))
        if method == "GET" and endpoint.startswith("actions/"):
            token = os.environ.get("GITHUB_TOKEN") or token
        if not repository or not token:
            raise policy.ToolError("Read Release Provider: GitHub repository and native token required")
        headers = {"Authorization": "Bearer " + token, "X-GitHub-Api-Version": "2022-11-28"}
        if upload:
            base = settings.get("upload_url", "https://uploads.github.com")
        url = (
            os.environ.get("GITHUB_GRAPHQL_URL", base.rstrip("/").removesuffix("/v3") + "/graphql")
            if graphql
            else f"{base}/repos/{repository}/{endpoint}"
        )
        # Release assets return their bytes only for octet-stream; every other REST call accepts only the REST
        # media type and answers 415 otherwise.
        headers["Accept"] = (
            "application/octet-stream" if binary and endpoint.startswith("releases/assets/") else "application/vnd.github+json"
        )
    elif provider == "gitlab":
        base = os.environ.get("CI_API_V4_URL", settings.get("api_url", "https://gitlab.com/api/v4"))
        if not isinstance(base, str) or not base:
            raise policy.ToolError("Read Release Provider: GitLab API URL required")
        project = settings.get("project_id", os.environ.get("CI_PROJECT_ID"))
        token = os.environ.get("RELEASE_TOKEN") or os.environ.get("GITLAB_TOKEN")
        job_token = os.environ.get("CI_JOB_TOKEN")
        if not project or not (token or job_token):
            raise policy.ToolError("Read Release Provider: GitLab project and protected native token required")
        if graphql:
            if not token or not base.rstrip("/").endswith("/api/v4"):
                raise policy.ToolError("Read Release Provider: GitLab policy read requires RELEASE_TOKEN and a native /api/v4 URL")
            headers = {"Authorization": "Bearer " + token}
            url = base.rstrip("/").removesuffix("/v4") + "/graphql"
        else:
            if channel:
                resource = endpoint.split("?", 1)[0]
                links = re.fullmatch(r"releases/nightly/assets/links(?:/[0-9]+)?", resource)
                allowed = (
                    resource == "releases"
                    and method == "POST"
                    or resource == "releases/nightly"
                    and method in {"GET", "PUT"}
                    or links
                    and method in {"GET", "POST", "DELETE"}
                )
                if not allowed:
                    raise policy.ToolError("Read Release Provider: channel publication requires native job-token release endpoints")
            cleanup = method == "DELETE" and endpoint.startswith("repository/branches/release%2Fv")
            merging = method == "PUT" and re.fullmatch(r"merge_requests/[0-9]+/merge", endpoint) is not None
            tagging = method == "POST" and endpoint in {"repository/tags", "repository/commits"} or cleanup or merging
            if (
                not channel
                and method != "GET"
                and not tagging
                and not ((method == "PUT" and endpoint.startswith("packages/generic/")) or (method == "POST" and endpoint == "releases"))
            ):
                raise policy.ToolError("Read Release Provider: GitLab release write endpoint is not allowed")
            if channel:
                if not job_token:
                    raise policy.ToolError("Read Release Provider: channel publication requires native CI_JOB_TOKEN")
                headers = {"JOB-TOKEN": job_token}
            elif tagging:
                if not token:
                    raise policy.ToolError(
                        "Read Release Provider: GitLab tagging and release reviews require the RELEASE_TOKEN release identity"
                    )
                headers = {"Authorization": "Bearer " + token}
            elif method == "GET":
                if not token:
                    raise policy.ToolError("Read Release Provider: GitLab reads require RELEASE_TOKEN with read_api access")
                headers = {"Authorization": "Bearer " + token}
            elif job_token:
                headers = {"JOB-TOKEN": job_token}
            elif os.environ.get("GITLAB_CI"):
                raise policy.ToolError("Read Release Provider: GitLab CI publication requires CI_JOB_TOKEN")
            elif token:
                headers = {"Authorization": "Bearer " + token}
            else:
                raise policy.ToolError("Read Release Provider: GitLab local publication requires RELEASE_TOKEN")
            url = f"{base}/projects/{urllib.parse.quote(str(project), safe='')}/{endpoint}"
        headers["Accept"] = "application/octet-stream" if binary else "application/json"
    else:
        raise policy.ToolError("Read Release Provider: release.provider must be github or gitlab")
    if data is not None:
        headers["Content-Type"] = "application/octet-stream" if binary else "application/json"
    native = urllib.request.Request(url, data=data, headers=headers, method=method)
    timeout_s = min(UPLOAD_MAX_TIMEOUT_S, 60 + len(data or b"") / 1_000_000) if binary and data is not None else 60

    def replace_partial() -> None:
        assets_endpoint = endpoint.split("?", 1)[0]
        name = urllib.parse.parse_qs(urllib.parse.urlsplit(endpoint).query)["name"][0]
        for asset in pages(settings, assets_endpoint, channel=channel):
            if asset["name"] == name:
                request(settings, f"releases/assets/{asset['id']}", "DELETE", channel=channel)

    try:
        result = notices.download(
            native,
            operation=f"release provider {method}",
            timeout_s=timeout_s,
            opener=urllib.request.build_opener(ProviderRedirect()).open,
            before_retry=replace_partial if upload else None,
        )
    except policy.ToolError as error:
        cause = error.__cause__
        if isinstance(cause, urllib.error.HTTPError) and cause.code == 404 and method in {"GET", "DELETE"}:
            return None
        raise
    if upload:
        asset = json.loads(result)
        digest = "sha256:" + hashlib.sha256(data or b"").hexdigest()
        if asset.get("size") != len(data or b"") or asset.get("digest") not in (None, digest):
            raise policy.ToolError(
                f"Read Release Provider: release provider {method} verification failed: {url}: asset size or sha256 differs"
            )
    return result


def document(settings: dict, endpoint: str, method: str = "GET", body: dict | None = None, *, channel: bool = False) -> Any:
    value = request(settings, endpoint, method, json.dumps(body).encode() if body is not None else None, channel=channel)
    return json.loads(value) if value else None


def pages(settings: dict, endpoint: str, key: str | None = None, *, channel: bool = False) -> list:
    result = []
    page = 1
    while True:
        separator = "&" if "?" in endpoint else "?"
        response = document(settings, f"{endpoint}{separator}per_page=100&page={page}", channel=channel)
        values = response.get(key, []) if key and isinstance(response, dict) else response
        if not isinstance(values, list):
            raise policy.ToolError("Read Release Provider: provider returned invalid paginated evidence")
        result.extend(values)
        if len(values) < 100:
            return result
        page += 1


def verify_duplicate_policy(settings: dict) -> None:
    """Read Back the Group's Generic Package Duplicate Protection; Publication Never Changes It."""
    project = document(settings, "")
    if not isinstance(project, dict):
        raise policy.ToolError("Read Release Provider: GitLab project identity is unavailable")
    path = project.get("path_with_namespace")
    owner = project.get("namespace")
    namespace = owner.get("full_path") if isinstance(owner, dict) else None
    if not isinstance(path, str) or not isinstance(namespace, str):
        raise policy.ToolError("Read Release Provider: GitLab group namespace identity is unavailable")
    query = "query ReleasePolicy($project: ID!) { project(fullPath: $project) { fullPath group { fullPath packageSettings { genericDuplicatesAllowed genericDuplicateExceptionRegex } } } }"
    raw = request(settings, "graphql", "POST", json.dumps({"query": query, "variables": {"project": path}}).encode(), graphql=True)
    result = json.loads(raw) if raw else {}
    if not isinstance(result, dict) or result.get("errors"):
        raise policy.ToolError("Read Release Provider: GitLab duplicate policy query failed")
    data = result.get("data")
    selected = data.get("project") if isinstance(data, dict) else None
    group = selected.get("group") if isinstance(selected, dict) else None
    package = group.get("packageSettings") if isinstance(group, dict) else None
    if (
        not isinstance(selected, dict)
        or selected.get("fullPath") != path
        or not isinstance(group, dict)
        or group.get("fullPath") != namespace
        or not isinstance(package, dict)
        or package.get("genericDuplicatesAllowed") is not False
        or package.get("genericDuplicateExceptionRegex") != ""
    ):
        raise policy.ToolError(
            "Read Release Provider: GitLab generic package duplicate protection must be verified for the project's group"
        )


def require_tag(directory: Path, tag: str) -> None:
    """The Protected Push of Exactly This Stable Tag, Checked Out at Its Commit."""
    if re.fullmatch(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", tag) is None:
        raise policy.ToolError("Read Release Provider: release requires a stable release tag")
    if os.environ.get("GITHUB_ACTIONS"):
        allowed = (
            os.environ.get("GITHUB_EVENT_NAME") == "push"
            and os.environ.get("GITHUB_REF") == "refs/tags/" + tag
            and os.environ.get("GITHUB_REF_PROTECTED") == "true"
        )
        sha = os.environ.get("GITHUB_SHA")
    else:
        allowed = (
            os.environ.get("PIPELINE_SOURCE", os.environ.get("CI_PIPELINE_SOURCE")) == "push"
            and os.environ.get("CI_COMMIT_TAG") == tag
            and os.environ.get("CI_COMMIT_REF_PROTECTED") == "true"
        )
        sha = os.environ.get("CI_COMMIT_SHA")
    if not allowed:
        raise policy.ToolError(
            f"Read Release Provider: publication requires the protected push of {tag}; an unprotected tag means the tag ruleset is missing"
        )
    actual = policy.execute(directory, ["git", "rev-parse", "HEAD"], capture=True).strip()
    tagged = policy.execute(directory, ["git", "rev-parse", "refs/tags/" + tag + "^{commit}"], capture=True).strip()
    if actual != sha or tagged != actual:
        raise policy.ToolError("Read Release Provider: release tag SHA differs from checkout")


def draft(settings: dict, tag: str, sha: str, body: str) -> dict:
    """The GitHub Draft for This Tag: Reused While Still a Draft, Never Replacing a Published Release."""
    releases = []
    for releases_item in pages(settings, "releases"):
        if releases_item["tag_name"] != tag:
            continue
        releases.append(releases_item)

    if len(releases) > 1:
        raise policy.ToolError("Read Release Provider: release tag identity ambiguous")
    if releases and not releases[0]["draft"]:
        raise policy.ToolError("Read Release Provider: published release cannot be replaced; use a new patch version")
    if releases:
        return releases[0]
    created = document(
        settings,
        "releases",
        "POST",
        {"tag_name": tag, "target_commitish": sha, "name": tag, "body": body, "draft": True, "prerelease": False},
    )
    if not isinstance(created, dict) or not created.get("draft") or created.get("tag_name") != tag:
        raise policy.ToolError("Read Release Provider: release draft was not created as a draft")
    return created


def upload(settings: dict, release: int, item: dict, data: bytes) -> str:
    """Upload One Asset to a Draft Unless Present, Then Verify Its Remote Bytes."""
    available = []
    for available_value in pages(settings, f"releases/{release}/assets"):
        if available_value["name"] != item["name"]:
            continue
        available.append(available_value)

    if len(available) > 1:
        raise policy.ToolError("Read Release Provider: duplicate draft asset")
    if available:
        endpoint = f"releases/assets/{available[0]['id']}"
    else:
        name = urllib.parse.quote(item["name"], safe="")
        uploaded = request(settings, f"releases/{release}/assets?name={name}", "POST", data, binary=True, upload=True)
        endpoint = f"releases/assets/{json.loads(uploaded or b'{}')['id']}"
    verify_download(settings, endpoint, item)
    return endpoint


def upload_generic(settings: dict, package: str, tag: str, item: dict, data: bytes) -> dict:
    """Upload One Asset to the Duplicate-Protected GitLab Generic Package Version."""
    endpoint = f"packages/generic/{urllib.parse.quote(package, safe='')}/{urllib.parse.quote(tag, safe='')}/{urllib.parse.quote(item['name'], safe='')}"
    existing = request(settings, endpoint, binary=True)
    if existing is not None and existing != data:
        raise policy.ToolError("Read Release Provider: partial upload conflict; use a new patch version")
    if existing is None:
        request(settings, endpoint, "PUT", data, binary=True)
    verify_download(settings, endpoint, item)
    base = os.environ.get("CI_API_V4_URL", settings.get("api_url", "https://gitlab.com/api/v4"))
    project = urllib.parse.quote(str(settings.get("project_id", os.environ.get("CI_PROJECT_ID"))), safe="")
    return {"endpoint": endpoint, "link": {"name": item["name"], "url": f"{base}/projects/{project}/{endpoint}"}}


def verify_download(settings: dict, endpoint: str, item: dict, *, channel: bool = False) -> bytes:
    data = request(settings, endpoint, binary=True, channel=channel)
    if data is None or len(data) != item["size"] or hashlib.sha256(data).hexdigest() != item["sha256"]:
        raise policy.ToolError(f"Read Release Provider: remote release download differs: {item['name']}")
    return data


def verify_signatures(directory: Path, settings: dict, root: Path, names: list[str]) -> None:
    """Run Product-Owned Signature Checks Over Hash-Verified Bytes with RELEASE_ASSET_ROOT Pointing at Them."""
    previous = os.environ.get("RELEASE_ASSET_ROOT")
    os.environ["RELEASE_ASSET_ROOT"] = str(root)
    try:
        for name in names:
            if not policy.inside(root, name).is_file():
                raise policy.ToolError(f"Read Release Provider: signature check input missing: {name}")
        for command in settings.get("signature_checks", []):
            policy.execute(directory, policy.command(command, "release.signature_checks"))
    finally:
        if previous is None:
            os.environ.pop("RELEASE_ASSET_ROOT", None)
        else:
            os.environ["RELEASE_ASSET_ROOT"] = previous


def oidc_claims(url: str, token: str) -> dict:
    """The Claims of a GitHub OIDC Token Requested for Diagnosis Only; the Token Itself Is Never Printed."""
    separator = "&" if "?" in url else "?"
    native = urllib.request.Request(f"{url}{separator}audience=release-identity", headers={"Authorization": "Bearer " + token})
    value = json.loads(notices.download(native, operation="release identity token fetch"))["value"]
    payload = value.split(".")[1]
    return json.loads(base64.urlsafe_b64decode(payload + "=" * (-len(payload) % 4)))


def workflow_filename(reference: str) -> str:
    """The Workflow File Name of a `owner/repo/.github/workflows/<file>@<ref>` Reference."""
    match = re.search(r"([^/]+\.ya?ml)(?=@)", reference)
    return match.group(1) if match else "(unknown)"


def delete_branch(settings: dict, branch: str) -> bool:
    """Delete a Remote Branch If It Still Exists, Since Native Source-Branch Removal Usually Got There First."""
    if settings["provider"] == "github":
        name = urllib.parse.quote(branch, safe="/")
        if document(settings, "git/ref/heads/" + name) is None:
            return False
        request(settings, "git/refs/heads/" + name, "DELETE")
    else:
        endpoint = "repository/branches/" + urllib.parse.quote(branch, safe="")
        if document(settings, endpoint) is None:
            return False
        request(settings, endpoint, "DELETE")
    return True


def create_tag(settings: dict, tag: str, sha: str) -> None:
    """Create the Lightweight Release Tag Through the Provider with the Release Token."""
    if settings["provider"] == "github":
        document(settings, "git/refs", "POST", {"ref": "refs/tags/" + tag, "sha": sha})
    else:
        document(settings, "repository/tags", "POST", {"tag_name": tag, "ref": sha})


def remote_tag(settings: dict, tag: str) -> str | None:
    """The Commit a Remote Tag Identifies, Peeling Annotated Tags."""
    name = urllib.parse.quote(tag, safe="")
    if settings["provider"] == "github":
        existing = document(settings, "git/ref/tags/" + name)
        if not existing:
            return None
        target = existing.get("object", {})
        if target.get("type") == "tag":
            target = (document(settings, "git/tags/" + target["sha"]) or {}).get("object", {})
        return target.get("sha")
    existing = document(settings, "repository/tags/" + name)
    return existing.get("commit", {}).get("id") if existing else None


def _retry_after(output: str) -> int:
    """Seconds Until the Registry-Indicated Retry Time, or the Published Interval."""
    match = re.search(r"try again after ([A-Z][a-z]{2}, \d{1,2} [A-Z][a-z]{2} \d{4} \d{2}:\d{2}:\d{2} GMT)", output)
    if match is None:
        return crates.RATE_INTERVAL_S
    retry = email.utils.parsedate_to_datetime(match.group(1)).timestamp()
    return max(1, min(crates.RATE_INTERVAL_S * 2, int(retry - time.time()) + 1))


# === Private Functions === #

# === Entry Point === #
