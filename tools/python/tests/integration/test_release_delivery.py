# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Final Commit Tags, Remote Bytes And Draft Signature Admission"""

# === Imports === #

# Standard Library Imports
import base64
import json
import shutil
import subprocess
import sys
from collections.abc import Callable
from http.client import HTTPMessage
from io import BytesIO
from pathlib import Path
from urllib.error import HTTPError, URLError

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.check.release import digest
from dokime_tools.ci import route
from dokime_tools.package import finalize, publish

# Type Checking Imports

# === Public Exports === #

# === Constants === #
COMMIT = "a" * 40
TAG = "v1.2.3"

# === Types === #

# === Classes === #

# === Public Functions === #


def case_tag(tmp_path, monkeypatch, scenario):
    registry = publish.Registry(
        "https://api.github.com", "stokker-technologies/dokime", "secret"
    )
    monkeypatch.setattr(publish, "verify_source_identity", lambda *args: None)
    ref = (
        None
        if scenario in {"create", "lost", "missing"}
        else {
            "ref": "refs/tags/" + TAG,
            "object": {
                "type": "commit",
                "sha": "b" * 40 if scenario == "conflict" else COMMIT,
            },
        }
    )
    posts = []

    def request(method, path, payload=None, **kwargs):
        nonlocal ref
        if method == "POST":
            assert payload is not None
            posts.append(payload)
            if scenario != "missing":
                ref = {
                    "ref": payload["ref"],
                    "object": {"type": "commit", "sha": payload["sha"]},
                }
            if scenario == "lost":
                raise URLError("uncertain response")
        return ref

    monkeypatch.setattr(registry, "request", request)
    if scenario in {"conflict", "missing"}:
        with pytest.raises(ValueError):
            registry.ensure_tag(TAG, COMMIT)
    else:
        registry.ensure_tag(TAG, COMMIT)
    assert len(posts) == int(scenario in {"create", "lost", "missing"})
    assert all(item == {"ref": "refs/tags/" + TAG, "sha": COMMIT} for item in posts)


def case_download(tmp_path, monkeypatch, scenario):
    original = tmp_path / "original"
    original.write_bytes(b"final signed bytes")
    expected = publish.Artifact(original, digest(original), original.stat().st_size)
    registry = publish.Registry(
        "https://api.github.com", "stokker-technologies/dokime", "secret"
    )
    calls = []

    def open_request(request, timeout):
        calls.append(request)
        if len(calls) == 1:
            assert request.get_header("Authorization") == "Bearer secret"
            assert request.full_url.endswith("/releases/assets/9")
            headers = HTTPMessage()
            headers["Location"] = (
                "http://storage.example/file"
                if scenario == "insecure"
                else "https://storage.example/file"
            )
            raise HTTPError(request.full_url, 302, "redirect", headers, None)
        assert not request.has_header("Authorization")
        data = original.read_bytes()
        return BytesIO(
            b"x" * len(data)
            if scenario == "hash"
            else data + b"extra"
            if scenario == "size"
            else data
        )

    monkeypatch.setattr(registry._opener, "open", open_request)
    target = tmp_path / "remote"
    if scenario == "valid":
        registry.download({"id": 9}, expected, target)
        assert target.read_bytes() == original.read_bytes()
    else:
        with pytest.raises(ValueError):
            registry.download({"id": 9}, expected, target)


def case_draft(tmp_path, monkeypatch, scenario):
    registry = finalize.ReleaseRegistry(
        "https://api.github.com", "stokker-technologies/dokime", "secret", "1.2.3"
    )
    remote = {"id": 1, "draft": True}
    posts = []
    monkeypatch.setattr(registry, "verify_tag", lambda *args: None)
    monkeypatch.setattr(registry, "release", lambda version: remote)
    monkeypatch.setattr(registry, "request", lambda *args: posts.append(args))

    def reject(*args):
        raise ValueError("remote checksum or signature failed")

    monkeypatch.setattr(finalize, "verify_downloads", reject)
    with pytest.raises(ValueError):
        registry.finalize(TAG, COMMIT, "Notes", {})
    assert not posts and remote["draft"] is True


def case_signature(tmp_path, monkeypatch, scenario):
    executable = shutil.which("minisign")
    if executable is None:
        if sys.platform == "win32":
            pytest.skip("Minisign publication verification runs on the Linux publisher")
        raise AssertionError("the locked Minisign tool must be installed")
    key, public = tmp_path / "key", tmp_path / "public"
    subprocess.run(
        [executable, "-G", "-W", "-s", str(key), "-p", str(public)],
        check=True,
        capture_output=True,
    )
    installer = tmp_path / "setup.exe"
    installer.write_bytes(b"native installer fixture")
    raw_signature = tmp_path / "signature"
    subprocess.run(
        [
            executable,
            "-S",
            "-m",
            str(installer),
            "-s",
            str(key),
            "-x",
            str(raw_signature),
        ],
        check=True,
        capture_output=True,
    )
    (tmp_path / "setup.exe.sig").write_text(
        base64.b64encode(raw_signature.read_bytes()).decode()
    )
    manifest = {
        "files": [
            {"role": role, "path": name}
            for role, name in [
                ("installer", "setup.exe"),
                ("signature", "setup.exe.sig"),
                ("updater", "latest.json"),
            ]
        ]
    }
    (tmp_path / "windows--manifest.json").write_text(json.dumps(manifest))
    config = tmp_path / "crates/dokime-gui/src-tauri/tauri.conf.json"
    config.parent.mkdir(parents=True)
    config.write_text(
        json.dumps(
            {
                "plugins": {
                    "updater": {
                        "pubkey": base64.b64encode(public.read_bytes()).decode()
                    }
                }
            }
        )
    )
    if scenario == "changed":
        installer.write_bytes(b"modified native installer")
        with pytest.raises(subprocess.CalledProcessError):
            finalize.verify_signature(tmp_path, tmp_path)
    else:
        finalize.verify_signature(tmp_path, tmp_path)


def case_hotfix(tmp_path, monkeypatch, scenario):
    (tmp_path / "Cargo.toml").write_text('[workspace.package]\nversion = "1.2.3"\n')
    (tmp_path / "stkr.toml").write_text("[release.hotfix]\n")
    event = {
        "pull_request": {
            "base": {
                "ref": "main",
                "sha": "b" * 40,
                "repo": {"full_name": route.REPOSITORY},
            },
            "head": {
                "ref": "fix/patch",
                "sha": "c" * 40,
                "repo": {"full_name": route.REPOSITORY},
            },
        }
    }
    monkeypatch.setattr(
        route.subprocess,
        "check_output",
        lambda command, **kwargs: "v1.2.2\n" if command[1] == "tag" else "b" * 40,
    )
    monkeypatch.setattr(
        route,
        "_request",
        lambda path: {
            "tag_name": "v1.2.2",
            "draft": scenario == "draft",
            "prerelease": False,
        },
    )
    if scenario == "draft":
        with pytest.raises(ValueError, match="actual published release"):
            route.verify(event, tmp_path)
    else:
        route.verify(event, tmp_path)


# === Private Functions === #


@pytest.mark.parametrize(
    "scenario,argument",
    [
        *[
            pytest.param(case_tag, value, id="tag-" + value)
            for value in ["existing", "create", "lost", "missing", "conflict"]
        ],
        *[
            pytest.param(case_download, value, id="download-" + value)
            for value in ["valid", "hash", "size", "insecure"]
        ],
        pytest.param(case_draft, "failure", id="draft-failure"),
        *[
            pytest.param(case_signature, value, id="signature-" + value)
            for value in ["valid", "changed"]
        ],
        *[
            pytest.param(case_hotfix, value, id="hotfix-" + value)
            for value in ["valid", "draft"]
        ],
    ],
)
def test_delivery_contract(
    scenario: Callable[..., None], argument: str, tmp_path: Path, monkeypatch
) -> None:
    scenario(tmp_path, monkeypatch, argument)


# === Entry Point === #
