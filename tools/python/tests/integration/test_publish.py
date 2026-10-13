# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Validate Resumable Native Python Release Uploads"""

# === Imports === #

# Standard Library Imports
import json
from collections.abc import Callable
from http.client import HTTPMessage
from io import BytesIO
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import Request

# Third Party Imports
import pytest

from dokime_tools.check.release import digest

# Local Imports
from dokime_tools.package import publish

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #


# === Classes === #
class FakeRegistry(publish.Registry):
    def ensure_tag(self, tag, commit):
        assert tag == "v1.2.3" and commit == "a" * 40

    def __init__(self):
        self.remote: dict[str, tuple[str, int]] = {}
        self.uploaded: list[str] = []
        self.interrupt = False

    def files(self, version):
        return dict(self.remote)

    def upload(self, name, artifact):
        self.uploaded.append(artifact.path.name)
        self.remote[artifact.path.name] = (
            digest(artifact.path),
            artifact.path.stat().st_size,
        )
        if self.interrupt:
            raise OSError("connection lost after server accepted upload")


# === Public Functions === #
def test_publication_reuploading(tmp_path: Path):
    expected = _artifacts(tmp_path)
    registry = FakeRegistry()
    registry.interrupt = True
    with pytest.raises(OSError):
        publish.reconcile(expected, registry, "1.2.3")
    assert registry.uploaded == ["linux.whl"]
    registry.interrupt = False
    publish.reconcile(expected, registry, "1.2.3")
    assert registry.uploaded == list(expected)
    publish.reconcile(expected, registry, "1.2.3")
    assert registry.uploaded == list(expected)


@pytest.mark.parametrize("conflict", ["checksum", "size", "extra"])
def test_registry_upload(tmp_path: Path, conflict: str):
    expected = _artifacts(tmp_path)
    registry = FakeRegistry()
    artifact = expected["linux.whl"]
    registry.remote["unexpected.whl" if conflict == "extra" else "linux.whl"] = (
        "0" * 64 if conflict == "checksum" else artifact.sha256,
        artifact.size + 1 if conflict == "size" else artifact.size,
    )
    with pytest.raises(ValueError, match="conflicts"):
        publish.reconcile(expected, registry, "1.2.3")
    assert not registry.uploaded


def case_missing_success(tmp_path: Path, monkeypatch):
    registry = FakeRegistry()
    monkeypatch.setattr(registry, "upload", lambda name, artifact: None)
    with pytest.raises(ValueError, match="no verified registry receipt"):
        publish.reconcile(_artifacts(tmp_path), registry, "1.2.3")


@pytest.mark.parametrize("when", ["before", "during", "final"])
def test_artifact_refused(tmp_path, monkeypatch, when):
    expected = _artifacts(tmp_path)
    registry = FakeRegistry()
    if when == "before":
        expected["linux.whl"].path.write_bytes(b"tampered")
    elif when == "during":
        original = registry.upload

        def upload(name, artifact):
            original(name, artifact)
            artifact.path.write_bytes(b"tampered")

        monkeypatch.setattr(registry, "upload", upload)
    else:
        count = 0

        def files(version):
            nonlocal count
            count += 1
            return {} if count == 5 else dict(registry.remote)

        monkeypatch.setattr(registry, "files", files)
    with pytest.raises(ValueError, match="changed|incomplete"):
        publish.reconcile(expected, registry, "1.2.3")


def case_registry_bounds(monkeypatch):
    registry = publish.Registry(
        "https://api.github.com", "stokker-technologies/dokime", "secret"
    )

    def open_request(request, timeout):
        assert request.get_header("Authorization") == "Bearer secret"
        assert "secret" not in request.full_url
        return BytesIO(b'{"id":1}')

    monkeypatch.setattr(registry._opener, "open", open_request)
    assert registry.request("GET", "/releases/1") == {"id": 1}
    monkeypatch.setattr(
        registry._opener,
        "open",
        lambda *args, **kwargs: BytesIO(b"x" * (publish.RESPONSE_BYTES + 1)),
    )
    with pytest.raises(ValueError, match="response budget"):
        registry.request("GET", "/releases/1")


@pytest.mark.parametrize("case", ["state", "duplicate", "hash", "size", "identity"])
def test_registry_metadata(monkeypatch, case):
    registry = publish.Registry(
        "https://api.github.com", "stokker-technologies/dokime", "secret"
    )
    asset = {
        "name": "linux.whl",
        "digest": "sha256:" + "a" * 64,
        "size": 3,
        "state": "uploaded",
    }
    remote = {"id": 1, "tag_name": "v1.2.3", "prerelease": False, "assets": [asset]}
    if case == "state":
        asset["state"] = "starter"
    elif case == "hash":
        asset["digest"] = None
    elif case == "size":
        asset["size"] = True
    elif case == "identity":
        remote["tag_name"] = "v9.9.9"
    else:
        remote["assets"].append(asset)
    monkeypatch.setattr(registry, "request", lambda *args, **kwargs: remote)
    with pytest.raises(ValueError):
        registry.files("1.2.3")


def case_registry_states(monkeypatch):
    registry = publish.Registry(
        "https://api.github.com", "stokker-technologies/dokime", "secret"
    )
    remote = {
        "id": 1,
        "tag_name": "v1.2.3",
        "prerelease": False,
        "assets": [{"name": "installer.exe"}],
    }
    monkeypatch.setattr(registry, "request", lambda *args, **kwargs: remote)
    assert registry.files("1.2.3") == {}


@pytest.mark.parametrize(
    "api",
    [
        "http://gitlab.example/api/v4",
        "https://secret@gitlab.example/api/v4",
        "https://api.github.com?token=secret",
    ],
)
def test_registry_endpoints(api):
    with pytest.raises(ValueError):
        publish.Registry(api, "stokker-technologies/dokime", "secret")
    assert (
        publish.NoRedirect().redirect_request(
            Request("https://gitlab.example"),
            BytesIO(),
            302,
            "",
            HTTPMessage(),
            "https://other",
        )
        is None
    )


@pytest.mark.parametrize("timeout", [False, True])
def test_upload_credentials(tmp_path, monkeypatch, timeout):
    registry = publish.Registry(
        "https://api.github.com", "stokker-technologies/dokime", "secret"
    )
    registry.version = "1.2.3"
    monkeypatch.setattr(
        registry, "release", lambda *args, **kwargs: {"id": 1, "draft": True}
    )

    def open_request(request, **kwargs):
        assert request.method == "POST"
        assert (
            request.full_url
            == "https://uploads.github.com/repos/stokker-technologies/dokime/releases/1/assets?name=linux.whl"
        )
        assert request.get_header("Authorization") == "Bearer secret"
        assert "secret" not in request.full_url
        assert request.data.read() == (tmp_path / "linux.whl").read_bytes()
        if timeout:
            raise TimeoutError("unknown outcome")
        return BytesIO(b"{}")

    monkeypatch.setattr(registry._opener, "open", open_request)
    registry.upload("linux.whl", _artifacts(tmp_path)["linux.whl"])


def case_main_network(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(publish, "release_tag", lambda root: "v1.2.3")
    monkeypatch.setenv("GITHUB_SHA", "a" * 40)
    monkeypatch.setenv("GITHUB_REF_NAME", "v1.2.3")
    monkeypatch.setattr(publish, "verify_source_identity", lambda *args: None)
    receipt = tmp_path / ".stkr/state/release/python-published.json"
    receipt.parent.mkdir(parents=True)
    receipt.write_text("stale")
    (receipt.parent / "verified.json").write_text(
        json.dumps({"commit": "a" * 40, "verified_platforms": ["linux", "windows"]})
    )
    assert publish.main() == 1
    assert not receipt.exists()


def test_main_errors(tmp_path, monkeypatch, capsys):
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(publish, "release_tag", lambda root: "v1.2.3")
    monkeypatch.setenv("GITHUB_SHA", "a" * 40)
    monkeypatch.setenv("GITHUB_REF_NAME", "v1.2.3")

    def failed(*args):
        raise HTTPError("https://secret@example", 403, "secret", HTTPMessage(), None)

    monkeypatch.setattr(publish, "verify_source_identity", failed)
    assert publish.main() == 1
    assert "secret" not in capsys.readouterr().err


@pytest.mark.parametrize("changed", [False, True])
def test_main_validation(tmp_path, monkeypatch, changed):
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(publish, "release_tag", lambda root: "v1.2.3")
    for key, value in {
        "GITHUB_SHA": "a" * 40,
        "GITHUB_REF_NAME": "v1.2.3",
        "GITHUB_API_URL": "https://api.github.com",
        "GITHUB_REPOSITORY": "stokker-technologies/dokime",
        "GITHUB_TOKEN": "secret",
    }.items():
        monkeypatch.setenv(key, value)
    directory = tmp_path / "dist/build"
    manifests = []
    for platform, names in [
        ("linux", ["linux.whl", "dokime.tar.gz"]),
        ("windows", ["windows.whl"]),
    ]:
        (directory / platform).mkdir(parents=True)
        entries = []
        for name in names:
            path = directory / platform / name
            path.write_bytes(name.encode())
            entries.append(
                {
                    "role": "wheel" if name.endswith("whl") else "sdist",
                    "path": name,
                    "sha256": digest(path),
                    "bytes": path.stat().st_size,
                }
            )
        manifests.append({"platform": platform, "files": entries})
    validations = []

    def verify(*args):
        validations.append(args)
        if changed and len(validations) == 2:
            raise ValueError("changed final manifest")
        return manifests

    identities = []
    monkeypatch.setattr(publish, "verify_release", verify)
    monkeypatch.setattr(
        publish, "verify_source_identity", lambda *args: identities.append(args)
    )
    registry = FakeRegistry()
    monkeypatch.setattr(publish, "Registry", lambda *args: registry)
    assert publish.main() == int(changed)
    assert len(validations) == len(identities) == 2
    receipt = tmp_path / ".stkr/state/release/python-published.json"
    if changed:
        assert not receipt.exists()
    else:
        result = json.loads(receipt.read_text())
        assert result["commit"] == "a" * 40
        assert set(result["files"]) == set(registry.uploaded)
        assert len(result["files"]) == 3


@pytest.mark.parametrize(
    "case",
    [
        "valid",
        "unprotected",
        "wrong_head",
        "wrong_tag",
        "dirty",
        "unmerged",
        "foreign",
        "feature",
    ],
)
def test_source_checkout(tmp_path, monkeypatch, case):
    from dokime_tools.check import release
    from dokime_tools.ci import route

    event = {
        "action": "closed",
        "pull_request": {
            "merged": case != "unmerged",
            "merge_commit_sha": "a" * 40,
            "base": {
                "ref": "main",
                "repo": {"full_name": "stokker-technologies/dokime"},
            },
            "head": {
                "sha": "a" * 40,
                "ref": "feature" if case == "feature" else "release/1.2.3",
                "repo": {
                    "full_name": "other/dokime"
                    if case == "foreign"
                    else "stokker-technologies/dokime"
                },
            },
        },
    }
    path = tmp_path / "event.json"
    path.write_text(json.dumps(event))
    for key, value in {
        "GITHUB_REF_PROTECTED": "false" if case == "unprotected" else "true",
        "GITHUB_REF": "refs/heads/main",
        "GITHUB_REPOSITORY": "stokker-technologies/dokime",
        "GITHUB_EVENT_NAME": "pull_request",
        "GITHUB_EVENT_PATH": str(path),
    }.items():
        monkeypatch.setenv(key, value)
    monkeypatch.setattr(route, "version", lambda root: "1.2.3")
    if case == "feature":
        (tmp_path / "stkr.toml").write_text("")
    monkeypatch.chdir(tmp_path)

    def output(command):
        if command[1] == "status":
            return " M Cargo.toml" if case == "dirty" else ""
        return "b" * 40 if case == "wrong_head" else "a" * 40

    monkeypatch.setattr(release, "_git_output", output)
    tag = "v1.2.4" if case == "wrong_tag" else "v1.2.3"
    if case == "valid":
        release.verify_source_identity("a" * 40, tag)
    else:
        with pytest.raises(ValueError):
            release.verify_source_identity("a" * 40, tag)


# === Private Functions === #
def _artifacts(root):
    result = {}
    for name in ("linux.whl", "windows.whl", "dokime.tar.gz"):
        path = root / name
        path.write_bytes(name.encode())
        result[name] = publish.Artifact(path, digest(path), path.stat().st_size)
    return result


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_missing_success, id="missing_success"),
        pytest.param(case_main_network, id="main_network"),
    ],
)
def test_contract(scenario: Callable[..., None], tmp_path: Path, monkeypatch) -> None:
    scenario(tmp_path, monkeypatch)


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_registry_bounds, id="registry_bounds"),
        pytest.param(case_registry_states, id="registry_states"),
    ],
)
def test_contract_2(scenario: Callable[..., None], monkeypatch) -> None:
    scenario(monkeypatch)
