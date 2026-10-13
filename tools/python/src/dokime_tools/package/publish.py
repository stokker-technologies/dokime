# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Reconcile Native Python Release Uploads"""

# === Imports === #

# Standard Library Imports
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

# Third Party Imports
# Local Imports
from dokime_tools.check.release import (
    digest,
    release_tag,
    verify_release,
    verify_source_identity,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #
RESPONSE_BYTES = 4 * 1024 * 1024

# === Types === #


# === Classes === #
@dataclass(frozen=True)
class Artifact:
    path: Path
    sha256: str
    size: int


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class Registry:
    """Exact GitHub Release Inventories; Credentials Remain In Headers."""

    max_files = 3

    def __init__(self, api: str, project: str, token: str):
        if (
            api != "https://api.github.com"
            or project != "stokker-technologies/dokime"
            or not token
        ):
            raise ValueError(
                "publication requires the configured GitHub repository and token"
            )
        self.url = f"{api}/repos/{project}"
        self.project = project
        self._token = token
        self._opener = build_opener(NoRedirect())
        self.version: str | None = None

    def request(
        self,
        method: str,
        path: str,
        payload: dict | None = None,
        *,
        missing: bool = False,
    ) -> dict | None:
        request = Request(
            self.url + path,
            method=method,
            data=json.dumps(payload).encode() if payload is not None else None,
            headers={
                "Authorization": f"Bearer {self._token}",
                "Accept": "application/vnd.github+json",
                "Content-Type": "application/json",
                "X-GitHub-Api-Version": "2022-11-28",
            },
        )
        try:
            with self._opener.open(request, timeout=30) as response:
                contents = response.read(RESPONSE_BYTES + 1)
        except HTTPError as error:
            if missing and error.code == 404:
                return None
            raise
        if len(contents) > RESPONSE_BYTES:
            raise ValueError("release metadata exceeds response budget")
        try:
            value = json.loads(contents)
            if not isinstance(value, dict):
                raise TypeError("release API returned invalid metadata")
        except TypeError as error:
            raise ValueError("release API returned invalid metadata") from error
        return value

    def ensure_tag(self, tag: str, commit: str) -> None:
        verify_source_identity(commit, tag)
        path = f"/git/ref/tags/{quote(tag, safe='')}"
        if self.request("GET", path, missing=True) is None:
            try:
                self.request(
                    "POST", "/git/refs", {"ref": f"refs/tags/{tag}", "sha": commit}
                )
            except (HTTPError, URLError, TimeoutError):
                pass  # Resolve An Uncertain Creation Without Replacing The Tag.
        self.verify_tag(tag, commit)

    def download(self, asset: dict, expected: Artifact, destination: Path) -> None:
        identity = asset.get("id")
        if type(identity) is not int or identity <= 0:
            raise ValueError("release asset ID is invalid")
        url = f"{self.url}/releases/assets/{identity}"
        headers = {
            "Authorization": f"Bearer {self._token}",
            "Accept": "application/octet-stream",
            "X-GitHub-Api-Version": "2022-11-28",
        }
        for _ in range(4):
            try:
                with (
                    self._opener.open(
                        Request(url, headers=headers), timeout=60
                    ) as response,
                    destination.open("wb") as output,
                ):
                    size = 0
                    while chunk := response.read(1024 * 1024):
                        size += len(chunk)
                        if size > expected.size:
                            raise ValueError(
                                "remote release asset exceeds expected size"
                            )
                        output.write(chunk)
                if size != expected.size or digest(destination) != expected.sha256:
                    raise ValueError(
                        "remote release asset bytes differ from validated bytes"
                    )
                return
            except HTTPError as error:
                if error.code not in (301, 302, 303, 307, 308):
                    raise
                url = error.headers.get("Location", "")
                redirect = urlsplit(url)
                if (
                    redirect.scheme != "https"
                    or not redirect.netloc
                    or redirect.username
                    or redirect.password
                ):
                    raise ValueError("invalid release storage redirect") from error
                headers = {}
        raise ValueError("too many release storage redirects")

    def verify_tag(self, tag: str, commit: str) -> None:
        ref = self.request("GET", f"/git/ref/tags/{quote(tag, safe='')}")
        if ref is None or ref.get("ref") != f"refs/tags/{tag}":
            raise ValueError("remote release tag is missing")
        target = ref["object"]
        for _ in range(4):
            if target.get("type") == "commit":
                if target.get("sha") != commit:
                    raise ValueError("remote release tag identifies a different commit")
                return
            if target.get("type") != "tag" or not re.fullmatch(
                r"[a-f0-9]{40}", target.get("sha", "")
            ):
                raise ValueError("remote tag metadata is invalid")
            annotated = self.request("GET", f"/git/tags/{target['sha']}")
            if annotated is None:
                raise ValueError("remote annotated tag is missing")
            target = annotated["object"]
        raise ValueError("remote annotated tag exceeds its nesting budget")

    def release(self, version: str, *, create: bool = False) -> dict | None:
        if not re.fullmatch(r"\d+\.\d+\.\d+", version):
            raise ValueError("release assets require a stable version")
        self.version = version
        tag = "v" + version
        remote = self.request("GET", f"/releases/tags/{tag}", missing=True)
        if remote is None and create:
            self.verify_tag(tag, os.environ["GITHUB_SHA"])
            try:
                self.request(
                    "POST",
                    "/releases",
                    {
                        "tag_name": tag,
                        "name": f"Dokime {tag}",
                        "draft": True,
                        "prerelease": False,
                    },
                )
            except (HTTPError, URLError, TimeoutError):
                pass
            remote = self.request("GET", f"/releases/tags/{tag}", missing=True)
        if remote is not None and (
            remote.get("tag_name") != tag
            or type(remote.get("id")) is not int
            or remote["id"] <= 0
            or remote.get("prerelease") is not False
        ):
            raise ValueError("native release identity is invalid")
        return remote

    def owns(self, name: str) -> bool:
        return name.endswith((".whl", ".tar.gz"))

    def files(self, version: str) -> dict[str, tuple[str, int]]:
        remote = self.release(version)
        if remote is None:
            return {}
        assets = remote.get("assets")
        if not isinstance(assets, list) or len(assets) > 32:
            raise ValueError("native release inventory exceeds its bounds")
        files = {}
        for item in assets:
            name = item.get("name")
            if (
                not isinstance(name, str)
                or not re.fullmatch(r"[A-Za-z0-9_.-]{1,255}", name)
                or name in {".", ".."}
            ):
                raise ValueError("release asset name is invalid")
            if not self.owns(name):
                continue
            checksum, size = item.get("digest"), item.get("size")
            if (
                name in files
                or len(files) >= self.max_files
                or item.get("state") != "uploaded"
                or not isinstance(checksum, str)
                or not re.fullmatch(r"sha256:[a-f0-9]{64}", checksum)
                or type(size) is not int
                or size <= 0
            ):
                raise ValueError(
                    "release returned an ambiguous or invalid file inventory"
                )
            files[name] = (checksum[7:], size)
        return files

    def upload(self, name: str, artifact: Artifact) -> None:
        if (
            not self.version
            or not re.fullmatch(r"[A-Za-z0-9_.-]{1,255}", name)
            or name in {".", ".."}
        ):
            raise ValueError("release upload identity is invalid")
        remote = self.release(self.version, create=True)
        if remote is None:
            raise ValueError("release creation has no authoritative receipt")
        url = f"https://uploads.github.com/repos/{self.project}/releases/{remote['id']}/assets?name={quote(name, safe='')}"
        with artifact.path.open("rb") as content:
            request = Request(
                url,
                data=content,
                method="POST",
                headers={
                    "Authorization": f"Bearer {self._token}",
                    "Content-Type": "application/octet-stream",
                    "Content-Length": str(artifact.size),
                    "X-GitHub-Api-Version": "2022-11-28",
                },
            )
            try:
                with self._opener.open(request, timeout=60) as response:
                    response.read(RESPONSE_BYTES + 1)
            except (HTTPError, URLError, TimeoutError):
                pass  # Reconcile Unknown Outcomes; Never Overwrite An Asset.


# === Public Functions === #
def main() -> int:
    root = Path.cwd()
    receipt = root / ".stkr/state/release/python-published.json"
    receipt.unlink(missing_ok=True)
    try:
        commit, tag = os.environ["GITHUB_SHA"], release_tag(root)
        verify_source_identity(commit, tag)
        directory = root / "dist/build"
        manifests = verify_release(directory, commit, tag, root)
        expected = artifacts(directory, manifests)
        registry = Registry(
            os.environ["GITHUB_API_URL"],
            os.environ["GITHUB_REPOSITORY"],
            os.environ["GITHUB_TOKEN"],
        )
        registry.ensure_tag(tag, commit)
        reconcile(expected, registry, tag[1:])
        verify_source_identity(commit, tag)
        if verify_release(directory, commit, tag, root) != manifests:
            raise ValueError("validated release manifests changed during publication")
        receipt.parent.mkdir(parents=True, exist_ok=True)
        receipt.write_text(
            json.dumps(
                {
                    "commit": commit,
                    "tag": tag,
                    "files": {
                        name: {"sha256": item.sha256, "bytes": item.size}
                        for name, item in expected.items()
                    },
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
    except (KeyError, ValueError, OSError, subprocess.SubprocessError):
        # HTTP/subprocess errors may contain server-supplied text or credentials.
        print(
            "Python publication incomplete; verify inputs and registry state before retrying.",
            file=sys.stderr,
        )
        return 1
    return 0


def artifacts(directory: Path, manifests: list[dict]) -> dict[str, Artifact]:
    expected: dict[str, Artifact] = {}
    for manifest in manifests:
        for entry in manifest["files"]:
            if entry["role"] != "wheel" and not (
                entry["role"] == "sdist" and manifest["platform"] == "linux"
            ):
                continue
            path = directory / manifest["platform"] / entry["path"]
            if path.name in expected:
                raise ValueError("duplicate Python distribution filename")
            expected[path.name] = Artifact(path, entry["sha256"], entry["bytes"])
    if len(expected) != 3:
        raise ValueError(
            "Python publication requires two native wheels and one source distribution"
        )
    return expected


def reconcile(expected: dict[str, Artifact], registry: Registry, version: str) -> None:
    remote = registry.files(version)
    _matching(expected, remote)
    for name, artifact in expected.items():
        _unchanged(artifact)
        if name in remote:
            continue
        registry.upload(name, artifact)
        _unchanged(artifact)
        remote = registry.files(version)
        _matching(expected, remote)
        if name not in remote:
            raise ValueError("uploaded distribution has no verified registry receipt")
    remote = registry.files(version)
    verify_inventory(expected, remote)
    for artifact in expected.values():
        _unchanged(artifact)


def verify_inventory(
    expected: dict[str, Artifact], remote: dict[str, tuple[str, int]]
) -> None:
    """Require exact content and completeness before a dependent release step."""
    _matching(expected, remote)
    if set(remote) != set(expected):
        raise ValueError("published distribution inventory is incomplete")


# === Private Functions === #
def _matching(
    expected: dict[str, Artifact], remote: dict[str, tuple[str, int]]
) -> None:
    for name, actual in remote.items():
        if name not in expected or actual != (
            expected[name].sha256,
            expected[name].size,
        ):
            raise ValueError("registry content conflicts with the validated release")


def _unchanged(artifact: Artifact) -> None:
    if (
        artifact.path.is_symlink()
        or artifact.path.stat().st_size != artifact.size
        or digest(artifact.path) != artifact.sha256
    ):
        raise ValueError("validated distribution changed before publication completed")


# === Entry Point === #
if __name__ == "__main__":
    raise SystemExit(main())
