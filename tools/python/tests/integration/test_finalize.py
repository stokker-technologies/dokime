# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Validate Native Release Finalization"""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable
from copy import deepcopy
from http.client import HTTPMessage
from io import BytesIO
import json
from pathlib import Path
from urllib.error import HTTPError, URLError

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.package import finalize, publish

# Type Checking Imports

# === Public Exports === #

# === Constants === #
COMMIT = "a" * 40
TAG = "v1.2.3"

# === Types === #


# === Classes === #
class Registry(finalize.ReleaseRegistry):
    def __init__(self):
        super().__init__(
            "https://api.github.com", "stokker-technologies/dokime", "secret", "1.2.3"
        )
        self.remote = {
            "id": 1,
            "tag_name": TAG,
            "prerelease": False,
            "draft": True,
            "assets": [
                {
                    "name": name,
                    "digest": "sha256:" + item.sha256,
                    "size": item.size,
                    "browser_download_url": self.asset_url(name),
                }
                for name, item in _expected().items()
            ],
        }
        self.posts = 0
        self.fail_after_create = False
        self.tag_commit = COMMIT

    def request(self, method, path, payload=None, *, missing=False):
        if path.startswith("/git/ref/tags/"):
            return {
                "ref": f"refs/tags/{TAG}",
                "object": {"type": "commit", "sha": self.tag_commit},
            }
        if method == "PATCH":
            self.posts += 1
            assert payload is not None
            self.remote.update(deepcopy(payload))
            if self.fail_after_create:
                raise URLError("lost receipt")
        return self.remote


# === Public Functions === #
@pytest.mark.parametrize("lost_receipt", [False, True])
def test_finalization_retries(lost_receipt, monkeypatch):
    monkeypatch.setattr(finalize, "verify_downloads", lambda *args: None)
    registry = Registry()
    registry.fail_after_create = lost_receipt
    registry.finalize(TAG, COMMIT, "Release Notes", _expected())
    registry.finalize(TAG, COMMIT, "Release Notes", _expected())
    assert registry.posts == 1


@pytest.mark.parametrize(
    "change",
    ["commit", "notes", "url", "missing", "extra", "duplicate", "hash", "draft"],
)
def test_finalization_release(change, monkeypatch):
    monkeypatch.setattr(finalize, "verify_downloads", lambda *args: None)
    registry = Registry()
    registry.finalize(TAG, COMMIT, "Release Notes", _expected())
    entries = registry.remote["assets"]
    if change == "commit":
        registry.tag_commit = "b" * 40
    elif change == "notes":
        registry.remote["body"] = "Changed"
    elif change == "url":
        entries[0]["browser_download_url"] = "https://unrelated.example/installer"
    elif change == "missing":
        entries.pop()
    elif change == "extra":
        entries.append(
            {"name": "extra", "browser_download_url": registry.asset_url("extra")}
        )
    elif change == "duplicate":
        entries[1] = entries[0]
    elif change == "hash":
        entries[0]["digest"] = "sha256:" + "b" * 64
    else:
        registry.remote["draft"] = None
    with pytest.raises(ValueError):
        registry.finalize(TAG, COMMIT, "Release Notes", _expected())
    assert registry.posts == 1


def case_creation_fails(monkeypatch):
    monkeypatch.setattr(finalize, "verify_downloads", lambda *args: None)
    registry = Registry()
    original = registry.request

    def request(method, path, payload=None, **kwargs):
        if method == "PATCH":
            raise URLError("not committed")
        return original(method, path, payload, **kwargs)

    monkeypatch.setattr(registry, "request", request)
    with pytest.raises(ValueError):
        registry.finalize(TAG, COMMIT, "Release Notes", _expected())


def case_api_allowed(monkeypatch):
    registry = finalize.ReleaseRegistry(
        "https://api.github.com", "stokker-technologies/dokime", "secret", "1.2.3"
    )

    def open_request(request, timeout):
        assert request.get_header("Authorization") == "Bearer secret"
        assert "secret" not in request.full_url
        raise HTTPError(request.full_url, 404, "missing", HTTPMessage(), None)

    monkeypatch.setattr(registry._opener, "open", open_request)
    assert registry.request("GET", "/releases/tags/v1.2.3", missing=True) is None
    with pytest.raises(HTTPError):
        registry.verify_tag(TAG, COMMIT)
    monkeypatch.setattr(
        registry._opener, "open", lambda *args, **kwargs: BytesIO(b"[]")
    )
    with pytest.raises(ValueError, match="invalid metadata"):
        registry.request("GET", "/releases/1")


def test_partial_publication(tmp_path, monkeypatch):
    monkeypatch.setattr(finalize, "verify_downloads", lambda *args: None)
    monkeypatch.setattr(finalize.sys, "argv", ["finalize"])
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(finalize, "release_tag", lambda root: "v1.2.3")
    monkeypatch.setenv("GITHUB_SHA", COMMIT)
    monkeypatch.setenv("GITHUB_REF_NAME", TAG)
    monkeypatch.setattr(publish, "verify_source_identity", lambda *args: None)
    monkeypatch.setattr(
        publish,
        "verify_release",
        lambda *args: (_ for _ in ()).throw(
            ValueError("missing original platform manifests")
        ),
    )
    assert publish.main() == 1


@pytest.mark.parametrize(
    "phase", ["complete", "missing_python", "missing_assets", "changed_final_manifest"]
)
def test_final_release(tmp_path, monkeypatch, phase):
    monkeypatch.setattr(finalize, "verify_downloads", lambda *args: None)
    monkeypatch.setattr(finalize.sys, "argv", ["finalize"])
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(finalize, "release_tag", lambda root: "v1.2.3")
    for key, value in {
        "GITHUB_SHA": COMMIT,
        "GITHUB_REF_NAME": TAG,
        "GITHUB_API_URL": "https://api.github.com",
        "GITHUB_REPOSITORY": "stokker-technologies/dokime",
        "GITHUB_TOKEN": "secret",
    }.items():
        monkeypatch.setenv(key, value)
    receipt = tmp_path / ".stkr/state/release/release-published.json"
    receipt.parent.mkdir(parents=True)
    receipt.write_text("stale")
    monkeypatch.setattr(finalize, "verify_source_identity", lambda *args: None)
    count = 0

    def verify(*args):
        nonlocal count
        count += 1
        return (
            [{"changed": True}]
            if phase == "changed_final_manifest" and count == 3
            else []
        )

    monkeypatch.setattr(finalize, "verify_release", verify)
    expected = _expected()
    monkeypatch.setattr(publish, "artifacts", lambda *args: expected)
    monkeypatch.setattr(finalize.assets, "artifacts", lambda *args: expected)
    monkeypatch.setattr(finalize, "release_notes", lambda tag: "Release Notes")
    remote = {name: (item.sha256, item.size) for name, item in expected.items()}

    class Python:
        def files(self, version):
            return {} if phase == "missing_python" else remote

    registry = Registry()
    monkeypatch.setattr(
        registry, "files", lambda version: {} if phase == "missing_assets" else remote
    )
    monkeypatch.setattr(publish, "Registry", lambda *args: Python())
    monkeypatch.setattr(finalize, "ReleaseRegistry", lambda *args: registry)
    assert finalize.main() == int(phase != "complete")
    if phase == "complete":
        assert json.loads(receipt.read_text())["commit"] == COMMIT
        assert registry.posts == 1
    else:
        assert not receipt.exists()
    if phase.startswith("missing"):
        assert registry.posts == 0


def test_signed_contract():
    registry = Registry()
    assert (
        registry.asset_url("latest.json")
        == "https://github.com/stokker-technologies/dokime/releases/download/v1.2.3/latest.json"
    )
    assert registry.asset_url("setup.exe.sig").endswith(
        "/releases/download/v1.2.3/setup.exe.sig"
    )


# === Private Functions === #
def _expected():
    return {
        name: publish.Artifact(Path(name), "a" * 64, 1)
        for name in ("linux--installer.deb", "windows--setup.exe")
    }


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_creation_fails, id="creation_fails"),
        pytest.param(case_api_allowed, id="api_allowed"),
    ],
)
def test_contract(scenario: Callable[..., None], monkeypatch) -> None:
    scenario(monkeypatch)
