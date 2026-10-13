# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Validate Durable Release Asset Reconciliation"""

# === Imports === #

# Standard Library Imports
import json
from collections.abc import Callable
from io import BytesIO
from pathlib import Path
from urllib.error import URLError

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.check.release import digest
from dokime_tools.package import assets, publish

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #


# === Classes === #
class FakeAssets(assets.AssetRegistry):
    def __init__(self):
        self.remote = {}
        self.uploads = []
        self.interrupt = False

    def files(self, version):
        return dict(self.remote)

    def upload(self, name, artifact):
        self.uploads.append(name)
        self.remote[name] = (digest(artifact.path), artifact.path.stat().st_size)
        if self.interrupt:
            raise OSError("interrupted after commit")

    def asset_url(self, name):
        return f"https://github.com/stokker-technologies/dokime/releases/download/v1.2.3/{name}"


# === Public Functions === #
def case_assets_resume(tmp_path):
    directory, manifests = _release(tmp_path)
    expected = assets.artifacts(directory, manifests)
    assert {"linux--manifest.json", "windows--manifest.json"} <= expected.keys()
    assert {"dokime-linux.zip", "dokime-windows.zip"} <= expected.keys()
    registry = FakeAssets()
    registry.interrupt = True
    with pytest.raises(OSError):
        publish.reconcile(expected, registry, "1.2.3")
    registry.interrupt = False
    publish.reconcile(expected, registry, "1.2.3")
    publish.reconcile(expected, registry, "1.2.3")
    assert registry.uploads == list(expected)
    assert len(registry.remote) == 7
    manifest = json.loads(expected["release-manifest.json"].path.read_text())
    assert manifest["commit"] == "a" * 40
    sums = expected["SHA256SUMS"].path.read_text()
    assert "release-manifest.json" in sums and "SHA256SUMS" not in sums


def case_asset_flattening(tmp_path):
    directory, manifests = _release(tmp_path)
    first = manifests[1]["files"][-1]
    manifests[1]["files"].append({**first, "path": "nested/" + first["path"]})
    with pytest.raises(ValueError, match="collide"):
        assets.artifacts(directory, manifests)


@pytest.mark.parametrize("failure", [False, True])
def test_asset_url(tmp_path, monkeypatch, failure):
    directory, manifests = _release(tmp_path)
    registry = assets.AssetRegistry(
        "https://api.github.com", "stokker-technologies/dokime", "secret", "1.2.3"
    )
    name, artifact = next(iter(assets.artifacts(directory, manifests).items()))

    def open_request(request, timeout):
        assert timeout == 60
        assert request.method == "POST"
        assert request.get_header("Authorization") == "Bearer secret"
        assert request.get_header("Content-length") == str(artifact.size)
        assert request.full_url.endswith(f"/releases/1/assets?name={name}")
        assert "secret" not in request.full_url
        assert request.data.read() == artifact.path.read_bytes()
        if failure:
            raise URLError("unknown upload result")
        return BytesIO(b'{"message":"201 Created"}')

    monkeypatch.setattr(
        registry, "release", lambda *args, **kwargs: {"id": 1, "draft": True}
    )
    monkeypatch.setattr(registry._opener, "open", open_request)
    registry.upload(name, artifact)


@pytest.mark.parametrize(
    "name", ["..", "../outside", "linux/file", "file?token", "x" * 256]
)
def test_asset_names(name):
    registry = assets.AssetRegistry(
        "https://api.github.com", "stokker-technologies/dokime", "secret", "1.2.3"
    )
    with pytest.raises(ValueError):
        registry.asset_url(name)


def test_generic_identity(monkeypatch):
    registry = assets.AssetRegistry(
        "https://api.github.com", "stokker-technologies/dokime", "secret", "1.2.3"
    )

    remote = {
        "id": 1,
        "tag_name": "v1.2.3",
        "prerelease": False,
        "assets": [
            {
                "name": f"asset-{i}",
                "digest": "sha256:" + "a" * 64,
                "size": 1,
                "state": "uploaded",
            }
            for i in range(8)
        ],
    }
    monkeypatch.setattr(registry, "request", lambda *args, **kwargs: remote)
    assert len(registry.files("1.2.3")) == 8


@pytest.mark.parametrize(
    "phase", ["complete", "python_missing", "python_changed", "manifest_changed"]
)
def test_asset_manifests(tmp_path, monkeypatch, phase):
    directory, manifests = _release(tmp_path)
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(assets, "release_tag", lambda root: "v1.2.3")
    for key, value in {
        "GITHUB_SHA": "a" * 40,
        "GITHUB_REF_NAME": "v1.2.3",
        "GITHUB_API_URL": "https://api.github.com",
        "GITHUB_REPOSITORY": "stokker-technologies/dokime",
        "GITHUB_TOKEN": "secret",
    }.items():
        monkeypatch.setenv(key, value)
    validations = []

    def verify(*args):
        validations.append(args)
        if phase == "manifest_changed" and len(validations) == 2:
            raise ValueError("manifest changed")
        return manifests

    monkeypatch.setattr(assets, "verify_release", verify)
    monkeypatch.setattr(assets, "verify_source_identity", lambda *args: None)
    expected_python = publish.artifacts(directory, manifests)

    class Python:
        reads = 0

        def files(self, version):
            self.reads += 1
            if phase == "python_missing" or (
                phase == "python_changed" and self.reads == 2
            ):
                return {}
            return {
                name: (value.sha256, value.size)
                for name, value in expected_python.items()
            }

    monkeypatch.setattr(publish, "Registry", lambda *args: Python())
    registry = FakeAssets()

    # Preserve the declared inventory ceiling used by the pure inventory builder.
    class Factory:
        max_files = 32

        def __call__(self, *args):
            return registry

    monkeypatch.setattr(assets, "AssetRegistry", Factory())
    receipt = tmp_path / ".stkr/state/release/assets-published.json"
    receipt.parent.mkdir(parents=True)
    receipt.write_text("stale")
    assert assets.main() == int(phase != "complete")
    if phase == "complete":
        result = json.loads(receipt.read_text())
        assert result["commit"] == "a" * 40
        assert set(result["files"]) == set(registry.remote)
        assert all(
            item["url"].startswith(
                "https://github.com/stokker-technologies/dokime/releases/download/v1.2.3/"
            )
            for item in result["files"].values()
        )
    else:
        assert not receipt.exists()
    if phase == "python_missing":
        assert not registry.uploads


# === Private Functions === #
def _release(root: Path):
    directory = root / "dist/build"
    manifests = []
    for platform, names in [
        ("linux", ["dokime-linux.whl", "dokime.tar.gz", "audit.json"]),
        ("windows", ["dokime-windows.whl", "audit.json", "setup.exe"]),
    ]:
        path = directory / platform
        path.mkdir(parents=True)
        entries = []
        for name in names:
            file = path / name
            file.write_bytes(name.encode())
            role = (
                "wheel"
                if name.endswith("whl")
                else "sdist"
                if name.endswith("gz")
                else "installer"
                if name.endswith("exe")
                else "evidence"
            )
            entries.append(
                {
                    "path": name,
                    "role": role,
                    "sha256": digest(file),
                    "bytes": file.stat().st_size,
                }
            )
        manifest = {
            "platform": platform,
            "files": entries,
            "inputs": {"revision": "a" * 40},
            "build_revision": "a" * 40,
            "package_version": "1.2.3",
        }
        (path / "manifest.json").write_text(json.dumps(manifest))
        manifests.append(manifest)
    catalog = {"platforms": {}, "inputs": {"revision": "a" * 40}}
    downloads = root / "dist/downloads"
    downloads.mkdir(parents=True)
    for manifest in manifests:
        platform = manifest["platform"]
        archive = downloads / f"dokime-{platform}.zip"
        archive.write_bytes(f"original-{platform}-archive".encode())
        catalog["platforms"][platform] = {
            "manifest_sha256": digest(directory / platform / "manifest.json"),
            "archive": {
                "path": archive.relative_to(root).as_posix(),
                "sha256": digest(archive),
                "bytes": archive.stat().st_size,
            },
        }
    validation = root / "dist/validation"
    validation.mkdir(parents=True)
    (validation / "catalog.json").write_text(json.dumps(catalog))
    return directory, manifests


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_assets_resume, id="assets_resume"),
        pytest.param(case_asset_flattening, id="asset_flattening"),
    ],
)
def test_contract(scenario: Callable[..., None], tmp_path) -> None:
    scenario(tmp_path)
