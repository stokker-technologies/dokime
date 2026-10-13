# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Validate Immutable Native Artifact Retrieval"""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import stat
from collections.abc import Callable
from http.client import HTTPMessage
from io import BytesIO
from pathlib import Path
from urllib.error import HTTPError
from zipfile import ZipFile, ZipInfo

# Third Party Imports
import pytest

from dokime_tools.check.release import digest

# Local Imports
from dokime_tools.ci import artifacts

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def case_preserves_executable(tmp_path: Path, monkeypatch) -> None:
    archive = tmp_path / "native.zip"
    executable = ZipInfo("bin/dokime")
    executable.create_system = 3
    executable.external_attr = (stat.S_IFREG | 0o755) << 16
    with ZipFile(archive, "w") as contents:
        contents.writestr(executable, b"original executable")
        contents.writestr("manifest.json", "{}")
    staging = tmp_path / "staging"
    artifacts._extract(archive, staging, "linux")
    target = staging / "dist/build/linux" / executable.filename
    assert target.read_bytes() == b"original executable"
    assert target.stat().st_mode & 0o777 == 0o755
    assert not (staging / "dist/build/windows").exists()


def case_refuses_unsafe_entries(tmp_path: Path, monkeypatch, name: str) -> None:
    archive = tmp_path / "native.zip"
    with ZipFile(archive, "w") as contents:
        contents.writestr(name, b"invalid")
    with pytest.raises(ValueError, match="unsafe"):
        artifacts._extract(archive, tmp_path / "staging", "linux")


def case_refuses_ambiguous_package(tmp_path: Path, monkeypatch, kind: str) -> None:
    archive = tmp_path / "native.zip"
    with ZipFile(archive, "w") as contents:
        if kind == "symlink":
            link = ZipInfo("bin/dokime")
            link.external_attr = (stat.S_IFLNK | 0o777) << 16
            contents.writestr(link, "outside")
        elif kind == "duplicate":
            contents.writestr("manifest.json", "{}")
            with pytest.warns(UserWarning, match="Duplicate"):
                contents.writestr("manifest.json", "{}")
    with pytest.raises(ValueError):
        artifacts._extract(archive, tmp_path / "staging", "linux")


def case_requires_job_identity(tmp_path: Path, monkeypatch, value: object) -> None:
    with pytest.raises(ValueError, match="reference"):
        artifacts._job(value)


def case_refuses_invalid_promotion(
    tmp_path: Path, monkeypatch, source: str, target: str
) -> None:
    event = tmp_path / "event.json"
    event.write_text(
        json.dumps({"pull_request": {"head": {"ref": source}, "base": {"ref": target}}})
    )
    monkeypatch.setenv("GITHUB_EVENT_NAME", "pull_request")
    monkeypatch.setenv("GITHUB_EVENT_PATH", str(event))
    with pytest.raises(ValueError, match="fresh native build"):
        artifacts.reuse(tmp_path)


def case_reuses_original_bytes(tmp_path: Path, monkeypatch, change: str | None) -> None:
    fixtures = tmp_path / "fixtures"
    fixtures.mkdir()
    identity = {"revision": "a" * 40, "source_tree": "a" * 40, "graph": [], "locks": {}}
    catalog = {
        "version": 1,
        "catalog_job_id": "1",
        "pipeline_id": "2",
        "run_attempt": "1",
        "head_revision": "c" * 40,
        "inputs": identity,
        "platforms": {},
    }
    records = {}
    downloads = {}
    for index, platform in enumerate(artifacts.PLATFORMS):
        build, test = str(10 + index), str(20 + index)
        record = {
            "build_job_id": build,
            "pipeline_id": "2",
            "validation": {"test_job_id": test, "pipeline_id": "2"},
        }
        records[platform] = record
        manifest = fixtures / f"{platform}.json"
        manifest.write_text(json.dumps(record))
        download = fixtures / f"dokime-{platform}.zip"
        with ZipFile(download, "w") as contents:
            contents.write(manifest, "manifest.json")
        downloads[platform] = download.read_bytes()
        catalog["platforms"][platform] = {
            "artifact_id": test,
            "artifact_sha256": "d" * 64,
            "build_job_id": build,
            "test_job_id": test,
            "manifest_sha256": digest(manifest),
            "archive": {
                "path": f"dist/downloads/{download.name}",
                "sha256": digest(download),
                "bytes": download.stat().st_size,
            },
        }
        with ZipFile(fixtures / f"{test}.zip", "w") as archive:
            archive.write(download, download.name)
    if change == "tree":
        catalog["inputs"] = {"source_tree": "b" * 40}
    elif change == "revision":
        catalog["inputs"] = identity | {"revision": "b" * 40}
    elif change == "platform":
        del catalog["platforms"]["windows"]
    elif change == "manifest":
        catalog["platforms"]["linux"]["manifest_sha256"] = "0" * 64
    elif change == "archive":
        catalog["platforms"]["linux"]["archive"]["sha256"] = "0" * 64
    original = json.dumps(catalog).encode()
    with ZipFile(fixtures / "1.zip", "w") as archive:
        archive.writestr("catalog.json", original)
    monkeypatch.setenv("VALIDATED_RUN_ARTIFACT_ID", "1")
    monkeypatch.setenv("GITHUB_EVENT_NAME", "workflow_dispatch")

    def request(endpoint):
        if endpoint.startswith("actions/runs/"):
            return {
                "status": "completed",
                "conclusion": "success",
                "head_sha": "c" * 40,
            }
        identity = endpoint.rsplit("/", 1)[1]
        name = (
            "validation-catalog-1"
            if identity == "1"
            else f"validation-{artifacts.PLATFORMS[int(identity) - 20]}-{identity}"
        )
        return {
            "id": int(identity),
            "name": name,
            "workflow_run": {"id": 2},
            "expired": False,
            "digest": "sha256:" + "d" * 64,
        }

    monkeypatch.setattr(artifacts, "_request", request)
    monkeypatch.setattr(artifacts, "inputs", lambda root: identity)
    monkeypatch.setattr(
        artifacts,
        "verify_platform",
        lambda package, platform, root, **kwargs: records[platform],
    )
    monkeypatch.setattr(
        artifacts,
        "_download",
        lambda artifact, destination: destination.write_bytes(
            (fixtures / f"{artifact['id']}.zip").read_bytes()
        ),
    )
    if change:
        with pytest.raises(ValueError, match="mismatch"):
            artifacts.reuse(tmp_path)
        assert not (tmp_path / "dist/build").exists()
        return
    artifacts.reuse(tmp_path)
    assert (tmp_path / artifacts.CATALOG).read_bytes() == original
    for platform in artifacts.PLATFORMS:
        assert (
            tmp_path / "dist/downloads" / f"dokime-{platform}.zip"
        ).read_bytes() == downloads[platform]


def case_native_job_completion(tmp_path: Path, monkeypatch, conclusion) -> None:
    monkeypatch.setenv("GITHUB_ACTIONS", "true")
    monkeypatch.setenv("GITHUB_RUN_ID", "2")
    job = {
        "id": 3,
        "name": "native (linux) / Test (linux)",
        "status": "completed",
        "conclusion": conclusion,
    }
    monkeypatch.setattr(artifacts, "_request", lambda endpoint: {"jobs": [job]})
    if conclusion == "success":
        assert artifacts.job_id("test", "linux", completed=True) == "3"
    else:
        with pytest.raises(ValueError, match="did not pass"):
            artifacts.job_id("test", "linux", completed=True)


def case_storage_integrity(tmp_path: Path, monkeypatch, mutated) -> None:
    monkeypatch.setenv("GITHUB_REPOSITORY", artifacts.REPOSITORY)
    monkeypatch.setenv("GITHUB_TOKEN", "secret")
    contents = b"original native archive"

    class Opener:
        def open(self, request, **kwargs):
            if request.full_url.startswith("https://api.github.com/"):
                assert request.get_header("Authorization") == "Bearer secret"
                headers = HTTPMessage()
                headers["Location"] = "https://storage.example/signed-archive"
                raise HTTPError(request.full_url, 302, "redirect", headers, None)
            assert request.full_url == "https://storage.example/signed-archive"
            assert not request.has_header("Authorization")
            return BytesIO(contents if not mutated else b"changed")

    monkeypatch.setattr(artifacts, "build_opener", lambda *args: Opener())
    metadata = {"id": 1, "digest": "sha256:" + hashlib.sha256(contents).hexdigest()}
    target = tmp_path / "download.zip"
    if mutated:
        with pytest.raises(ValueError, match="checksum mismatch"):
            artifacts._download(metadata, target)
    else:
        artifacts._download(metadata, target)
        assert target.read_bytes() == contents


def case_expired_artifact(tmp_path: Path, monkeypatch) -> None:
    with pytest.raises(ValueError, match="expired"):
        artifacts._valid_artifact(
            {"id": 1, "expired": True, "digest": "sha256:" + "a" * 64}
        )


def case_selected_catalog(
    tmp_path: Path, monkeypatch, selected: tuple[str, ...]
) -> None:
    identity = {"revision": "a" * 40, "source_tree": "b" * 40}
    event = {
        "ref": "refs/heads/dev" if selected == ("linux",) else "refs/heads/main",
        "repository": {"full_name": artifacts.REPOSITORY},
    }
    monkeypatch.setattr(artifacts, "inputs", lambda root: identity)
    monkeypatch.setattr(artifacts, "current_event", lambda: event)
    monkeypatch.setenv("GITHUB_RUN_ID", "17")
    monkeypatch.setenv("GITHUB_RUN_ATTEMPT", "1")
    monkeypatch.setattr(artifacts, "job_id", lambda *args, **kwargs: "90")
    monkeypatch.setattr(artifacts, "_request", lambda *args: {"head_sha": "a" * 40})
    monkeypatch.setattr(artifacts, "_valid_artifact", lambda *args: None)
    monkeypatch.setattr(artifacts, "_checksum", lambda item: "c" * 64)
    available = {}
    for index, platform in enumerate(selected):
        test = str(20 + index)
        available[f"validation-{platform}-{test}"] = {"id": test}
        available[f"coverage-{platform}-90"] = {"id": "90"}
        package = tmp_path / "dist/build" / platform
        package.mkdir(parents=True)
        (package / "manifest.json").write_text("{}")
        archive = tmp_path / "dist/downloads" / f"dokime-{platform}.zip"
        archive.parent.mkdir(exist_ok=True)
        archive.write_bytes(platform.encode())
    monkeypatch.setattr(artifacts, "_inventory", lambda run: available)
    restored = []
    monkeypatch.setattr(
        artifacts, "restore", lambda root, platform: restored.append(platform)
    )

    def verify(package, platform, root, **kwargs):
        index = selected.index(platform)
        assert kwargs["require_validation"] is True
        return {
            "pipeline_id": "17",
            "build_job_id": str(10 + index),
            "validation": {
                "pipeline_id": "17",
                "test_job_id": str(20 + index),
                "contract_job_id": "90",
                "coverage_job_id": "90",
                "lane_job_ids": {
                    suite: "90" for suite in artifacts.required_suites(platform)
                },
            },
        }

    monkeypatch.setattr(artifacts, "verify_platform", verify)
    artifacts.catalog(tmp_path)
    catalog = json.loads((tmp_path / artifacts.CATALOG).read_text())
    assert restored == list(selected)
    assert set(catalog["platforms"]) == set(selected)
    assert catalog["inputs"] == identity
    assert catalog["validation_policy"] == {
        "required_platforms": list(selected),
    }


# === Private Functions === #


@pytest.mark.parametrize(
    "scenario,arguments",
    [
        pytest.param(case_preserves_executable, (), id="preserves_executable-0"),
        pytest.param(
            case_refuses_unsafe_entries, ("../outside",), id="refuses_unsafe_entries-1"
        ),
        pytest.param(
            case_refuses_unsafe_entries, ("/absolute",), id="refuses_unsafe_entries-2"
        ),
        pytest.param(
            case_refuses_unsafe_entries,
            ("bin/../outside",),
            id="refuses_unsafe_entries-3",
        ),
        pytest.param(
            case_refuses_unsafe_entries, ("C:/outside",), id="refuses_unsafe_entries-4"
        ),
        pytest.param(
            case_refuses_unsafe_entries,
            ("dist\\build\\linux\\escape",),
            id="refuses_unsafe_entries-5",
        ),
        pytest.param(
            case_refuses_ambiguous_package,
            ("symlink",),
            id="refuses_ambiguous_package-6",
        ),
        pytest.param(
            case_refuses_ambiguous_package,
            ("duplicate",),
            id="refuses_ambiguous_package-7",
        ),
        pytest.param(
            case_refuses_ambiguous_package,
            ("missing",),
            id="refuses_ambiguous_package-8",
        ),
        pytest.param(case_requires_job_identity, (None,), id="requires_job_identity-9"),
        pytest.param(case_requires_job_identity, (42,), id="requires_job_identity-10"),
        pytest.param(case_requires_job_identity, ("0",), id="requires_job_identity-11"),
        pytest.param(
            case_requires_job_identity, ("-1",), id="requires_job_identity-12"
        ),
        pytest.param(
            case_requires_job_identity, ("42/../other",), id="requires_job_identity-13"
        ),
        pytest.param(
            case_requires_job_identity,
            ("42?token=secret",),
            id="requires_job_identity-14",
        ),
        pytest.param(
            case_refuses_invalid_promotion,
            ("feature/example", "main"),
            id="refuses_invalid_promotion-15",
        ),
        pytest.param(
            case_refuses_invalid_promotion,
            ("dev", "dev"),
            id="refuses_invalid_promotion-16",
        ),
        pytest.param(
            case_reuses_original_bytes, (None,), id="reuses_original_bytes-17"
        ),
        pytest.param(
            case_reuses_original_bytes,
            ("revision",),
            id="reuses_original_bytes-revision",
        ),
        pytest.param(
            case_reuses_original_bytes, ("tree",), id="reuses_original_bytes-18"
        ),
        pytest.param(
            case_reuses_original_bytes, ("platform",), id="reuses_original_bytes-19"
        ),
        pytest.param(
            case_reuses_original_bytes, ("manifest",), id="reuses_original_bytes-20"
        ),
        pytest.param(
            case_reuses_original_bytes, ("archive",), id="reuses_original_bytes-21"
        ),
        pytest.param(
            case_native_job_completion, ("success",), id="native_job_completion-22"
        ),
        pytest.param(
            case_native_job_completion, ("failure",), id="native_job_completion-23"
        ),
        pytest.param(
            case_native_job_completion, ("skipped",), id="native_job_completion-24"
        ),
        pytest.param(
            case_native_job_completion, ("cancelled",), id="native_job_completion-25"
        ),
        pytest.param(
            case_native_job_completion, (None,), id="native_job_completion-26"
        ),
        pytest.param(case_storage_integrity, (False,), id="storage_integrity-27"),
        pytest.param(case_storage_integrity, (True,), id="storage_integrity-28"),
        pytest.param(case_expired_artifact, (), id="expired_artifact-29"),
        pytest.param(case_selected_catalog, (("linux",),), id="catalog-linux"),
        pytest.param(case_selected_catalog, (("linux", "windows"),), id="catalog-full"),
    ],
)
def test_artifact_contract(
    scenario: Callable[..., None], arguments: tuple, tmp_path: Path, monkeypatch
) -> None:
    scenario(tmp_path, monkeypatch, *arguments)


# === Entry Point === #
