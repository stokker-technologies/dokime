# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Mechanical Ci Gates Fixtures."""

# === Imports === #

# Standard Library Imports
import contextlib
import copy
import email.message
import hashlib
import io
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.release import (
    nightly,
    registry,
)

# Type Checking Imports

# === Public Exports === #


# === Constants === #

ROOT = Path(__file__).resolve().parents[4]


OLD = "b" * 40


NEW = "a" * 40


LATER = "c" * 40


OLD = "b" * 40


NEW = "a" * 40


LATER = "c" * 40

# === Types === #


# === Classes === #


class FakeOpener:
    """Answer Each Provider Request with the Next Scripted Status."""

    def __init__(self, codes: list[int]) -> None:
        self.codes = list(codes)
        self.methods: list[str] = []

    def open(self, request: urllib.request.Request, timeout: float) -> io.BytesIO:
        self.methods.append(request.get_method())
        code = self.codes.pop(0)
        if code != 200:
            raise urllib.error.HTTPError(request.full_url, code, "failure", email.message.Message(), io.BytesIO(b"{}"))
        return io.BytesIO(b"{}")


class FakeGitHub:
    """An In-Memory Release Provider with GitHub's Draft, Tag and Asset Semantics and Injectable Faults."""

    def __init__(self, *, previous: bool, faults: set[str] | None = None, immutable: bool = False) -> None:
        self.faults = set(faults or ())
        self.immutable = immutable
        self.tag = OLD if previous else None
        self.dev = NEW
        self.workflows = {OLD: "previous-workflows", NEW: "build-workflows"}
        self.releases: dict[int, dict] = {}
        self.assets: dict[int, list[dict]] = {}
        self.next_id = 10
        if previous:
            self.releases[1] = {
                "id": 1,
                "name": nightly.TITLE,
                "tag_name": nightly.TAG,
                "target_commitish": OLD,
                "draft": False,
                "prerelease": True,
                "body": "",
            }
            self.assets[1] = [{"id": 1, "name": "old.zip", "size": 3, "state": "uploaded"}]

    def patched(self) -> contextlib.ExitStack:
        stack = contextlib.ExitStack()
        stack.enter_context(patch.object(registry.provider, "document", side_effect=self.document))
        stack.enter_context(patch.object(registry.provider, "pages", side_effect=self.pages))
        stack.enter_context(patch.object(registry.provider, "request", side_effect=self.request))
        return stack

    def published(self) -> dict:
        (release,) = [item for item in self.releases.values() if item["tag_name"] == nightly.TAG and item["draft"] is False]
        return release

    def _fault(self, name: str, applies: bool = True, *, once: bool = True) -> None:
        if applies and name in self.faults:
            if once:
                self.faults.discard(name)
            raise policy.ToolError(f"injected {name}")

    def document(
        self, settings: dict, endpoint: str, method: str = "GET", body: dict | None = None, **kwargs: Any
    ) -> dict | list[dict] | None:
        body = copy.deepcopy(body or {})
        if endpoint == f"releases/tags/{nightly.TAG}":
            matches = []
            for matches_item in self.releases.values():
                if matches_item["tag_name"] != nightly.TAG:
                    continue
                if matches_item["draft"] is not False:
                    continue
                matches.append(matches_item)

            self._fault("lookup", any(item["id"] >= 10 for item in matches))
            return copy.deepcopy(matches[0]) if matches else None
        if endpoint == "releases" and method == "POST":
            self._fault("create")
            identity, self.next_id = self.next_id, self.next_id + 1
            self.releases[identity] = {"id": identity, **body}
            self.assets[identity] = []
            return copy.deepcopy(self.releases[identity])
        if endpoint.startswith("releases/") and method == "PATCH":
            release = self.releases[int(endpoint.split("/")[1])]
            if release.get("immutable"):
                raise policy.ToolError("immutable release")
            if body.get("draft") is True:
                self._fault("draft-previous", release["id"] < 10)
                # GitHub Lists A Release Patched Back To Draft Under An Untagged Placeholder.
                release.update(body, tag_name=f"untagged-{release['id']}")
            else:
                self._fault("publish", release["id"] >= 10)
                others = (item for item in self.releases.values() if item["id"] != release["id"])
                if any(item["tag_name"] == body.get("tag_name") and not item["draft"] for item in others):
                    raise policy.ToolError("tag_name already_exists")
                release.update(body)
                if self.immutable:
                    release["immutable"] = True
            return copy.deepcopy(release)
        if endpoint == f"git/ref/tags/{nightly.TAG}":
            return {"object": {"sha": self.tag}} if self.tag else None
        if endpoint == "git/ref/heads/dev":
            return {"object": {"sha": self.dev}}
        if endpoint.startswith("contents/.github?ref="):
            revision = endpoint.split("=", 1)[1]
            return [
                {"name": "actions", "type": "dir", "sha": "actions"},
                {"name": "workflows", "type": "dir", "sha": self.workflows[revision]},
            ]
        if endpoint == "git/refs" and method == "POST":
            self._fault("tag-create", body["sha"] == NEW)
            if self.tag is not None:
                raise policy.ToolError("Reference already exists")
            self.tag = body["sha"]
            return {}
        raise AssertionError((endpoint, method))

    def pages(self, settings: dict, endpoint: str, **kwargs: Any) -> list[dict]:
        if endpoint == "releases":
            return copy.deepcopy(list(self.releases.values()))
        assets = copy.deepcopy(self.assets[int(endpoint.split("/")[1])])
        for fault, change in (("size", {"size": 0}), ("pending", {"state": "new"}), ("digest", {"digest": "sha256:" + "0" * 64})):
            if fault in self.faults and assets:
                assets[0].update(change)
        return assets

    def request(self, settings: dict, endpoint: str, method: str = "GET", data: bytes | None = None, **kwargs: Any) -> bytes | None:
        if endpoint.startswith("releases/assets/") and method == "GET":
            identity = int(endpoint.rsplit("/", 1)[1])
            asset = next(asset for assets in self.assets.values() for asset in assets if asset["id"] == identity)
            return b"corrupt" if "download" in self.faults else asset["data"]
        if endpoint == f"git/refs/tags/{nightly.TAG}" and method == "DELETE":
            self.tag = None
            return
        if endpoint.startswith("releases/assets/") and method == "DELETE":
            identity = int(endpoint.rsplit("/", 1)[1])
            for assets in self.assets.values():
                assets[:] = [asset for asset in assets if asset["id"] != identity]
            return
        if method == "DELETE":
            identity = int(endpoint.split("/")[1])
            del self.releases[identity]
            del self.assets[identity]
            return
        release, name = int(endpoint.split("/")[1]), urllib.parse.unquote(endpoint.split("name=")[1])
        # A Persistent Upload Fault Outlasts Every Retry.
        self._fault(f"upload:{name}", once=False)
        assert isinstance(data, bytes)
        self.assets[release].append(
            {
                "id": self.next_id,
                "name": name,
                "size": len(data),
                "state": "uploaded",
                "digest": "sha256:" + hashlib.sha256(data).hexdigest(),
                "data": data,
            }
        )
        self.next_id += 1
        # The Upload Lands But The Response Is Lost, Leaving A Same-Name Asset For The Retry To Replace.
        self._fault(f"partial:{name}")
        return


# === Public Functions === #


# === Private Functions === #


def _staged_files(directory: Path) -> tuple[Path, list[dict]]:
    files = []
    for name in ("linux.zip", "windows.zip", "SHA256SUMS"):
        data = name.encode()
        (directory / name).write_bytes(data)
        files.append({"name": name, "path": name, "size": len(data), "sha256": hashlib.sha256(data).hexdigest()})
    return directory, files


# === Entry Point === #
