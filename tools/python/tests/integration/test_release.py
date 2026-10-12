# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Validate Cross Platform Release Artifact Identity."""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable
import json
import os
import subprocess
import sys
from pathlib import Path
from zipfile import ZipFile

# Third Party Imports
import pytest

# Local Imports
from dokime_tools import _policy, evidence
from dokime_tools.check import release as checks
from dokime_tools.ci import artifacts as native
from dokime_tools.package import artifacts as stage
from dokime_tools.release import inputs as source_inputs
from dokime_tools.check.release import (
    TARGETS,
    digest,
    verify_release,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

COMMIT = "a" * 40
TAG = "v1.2.3"

# === Types === #

# === Classes === #

# === Public Functions === #


def case_release_pair(tmp_path: Path) -> None:
    project = tmp_path / "metadata"
    project.mkdir()
    (project / "Cargo.toml").write_text(
        '[package]\nname = "metadata"\nversion = "0.1.0"\n'
        'description = "Ё · Unicode Metadata"\n[lib]\npath = "lib.rs"\n',
        encoding="utf-8",
    )
    (project / "lib.rs").write_bytes(b"")
    environment = dict(os.environ)
    environment.update(
        GIT_AUTHOR_NAME="Fixture",
        GIT_AUTHOR_EMAIL="fixture@example.com",
        GIT_COMMITTER_NAME="Fixture",
        GIT_COMMITTER_EMAIL="fixture@example.com",
    )
    for command in [
        ["cargo", "generate-lockfile", "--offline"],
        ["git", "init"],
        ["git", "add", "."],
        ["git", "-c", "commit.gpgsign=false", "commit", "-m", "Fixture"],
    ]:
        subprocess.run(
            command,
            cwd=project,
            env=environment,
            check=True,
            capture_output=True,
            encoding="utf-8",
        )
    with pytest.MonkeyPatch.context() as windows:
        windows.setattr(subprocess, "_text_encoding", lambda: "cp1252")
        identity = source_inputs(project)
        assert identity["source_tree"] and len(identity["graph"]) == 1
        metadata = evidence._metadata(project)
        assert metadata["packages"][0]["description"] == "Ё · Unicode Metadata"
        probe = [
            sys.executable,
            "-c",
            "import sys; sys.stdout.buffer.write('Ё · Unicode Metadata'.encode('utf-8'))",
        ]
        assert _policy.execute(project, probe, capture=True) == "Ё · Unicode Metadata"
    root = _release(tmp_path)
    manifests = verify_release(root, COMMIT, TAG, tmp_path)
    assert {manifest["platform"] for manifest in manifests} == {"linux", "windows"}
    downloads = tmp_path / "dist/downloads"
    downloads.mkdir(parents=True)
    for platform in ("linux", "windows"):
        package = root / platform
        template = package / "reports/report.html"
        original = template.read_bytes()
        archive = downloads / f"dokime-{platform}.zip"
        with ZipFile(archive, "w") as contents:
            for path in package.rglob("*"):
                if path.is_file():
                    contents.write(path, path.relative_to(package))
        native.restore(tmp_path, platform)
        restored = tmp_path / "dist/build" / platform / "reports/report.html"
        assert restored.read_bytes() == original
        canonical = tmp_path / "crates/dokime-app/dist-report/report.html"
        assert canonical.read_bytes() == original
        manifest = next(item for item in manifests if item["platform"] == platform)
        stage._report_template(tmp_path, manifest)
        canonical.unlink()
        with pytest.raises(ValueError, match="canonical report template"):
            stage._report_template(tmp_path, manifest)
        canonical.write_bytes(b"changed canonical report")
        with pytest.raises(ValueError, match="canonical report template"):
            stage._report_template(tmp_path, manifest)
        canonical.write_bytes(original)
        with ZipFile(archive, "w") as contents:
            for path in package.rglob("*"):
                if path.is_file():
                    contents.writestr(
                        path.relative_to(package).as_posix(),
                        b"changed report" if path == template else path.read_bytes(),
                    )
        with pytest.raises(ValueError, match="checksum or size mismatch"):
            native.restore(tmp_path, platform)
        assert restored.read_bytes() == original
        assert canonical.read_bytes() == original


@pytest.mark.parametrize("platform", ["linux", "windows"])
def test_release_platform(tmp_path: Path, platform: str) -> None:
    root = _release(tmp_path)
    (root / platform / "manifest.json").unlink()
    with pytest.raises(ValueError, match="missing release artifact"):
        verify_release(root, COMMIT, TAG, tmp_path)


@pytest.mark.parametrize(
    "field,value",
    [
        ("build_revision", "invalid"),
        ("build_revision", "b" * 40),
        ("package_version", "1.2.4"),
        ("architecture", "aarch64"),
        ("profile", "dev"),
        ("abi", "cp313"),
        ("python", "cp314"),
        ("platform", "linux"),
        ("target", "x86_64-pc-windows-gnu"),
        ("inputs", {}),
    ],
)
def test_release_drift(tmp_path: Path, field: str, value: object) -> None:
    root = _release(tmp_path)
    path = root / "windows/manifest.json"
    manifest = json.loads(path.read_text())
    manifest[field] = value
    path.write_text(json.dumps(manifest))
    with pytest.raises(ValueError, match="identity|version"):
        verify_release(root, COMMIT, TAG, tmp_path)


@pytest.mark.parametrize("change", ["changed", "missing", "unlisted", "dependency"])
def test_release_refuses(tmp_path: Path, change: str) -> None:
    root = _release(tmp_path)
    wheel = next((root / "linux").glob("*.whl"))
    if change == "changed":
        wheel.write_bytes(b"different wheel")
    elif change == "missing":
        wheel.unlink()
    elif change == "unlisted":
        (root / "linux/unverified.whl").write_bytes(b"extra")
    else:
        (tmp_path / "Cargo.lock").write_text("changed dependency resolution")
    with pytest.raises(ValueError):
        verify_release(root, COMMIT, TAG, tmp_path)


@pytest.mark.parametrize("change", ["duplicate", "traversal", "absolute", "backslash"])
def test_release_inventory(tmp_path: Path, change: str) -> None:
    root = _release(tmp_path)
    path = root / "linux/manifest.json"
    manifest = json.loads(path.read_text())
    if change == "duplicate":
        manifest["files"].append(manifest["files"][0])
    else:
        manifest["files"][0]["path"] = {
            "traversal": "../outside.whl",
            "absolute": "/outside.whl",
            "backslash": "..\\outside.whl",
        }[change]
    path.write_text(json.dumps(manifest))
    with pytest.raises(ValueError):
        verify_release(root, COMMIT, TAG, tmp_path)


@pytest.mark.parametrize("result", ["failure", "error", "skipped", "empty"])
def test_release_passing(tmp_path: Path, result: str) -> None:
    root = _release(tmp_path)
    content = f"<testsuite><testcase><{result}/></testcase></testsuite>"
    if result == "empty":
        content = "<testsuite/>"
    _replace(root / "linux", "test:tests-acceptance-python.xml", content.encode())
    with pytest.raises(ValueError, match="validation"):
        verify_release(root, COMMIT, TAG, tmp_path)


def case_release_wheel(tmp_path: Path) -> None:
    root = _release(tmp_path)
    before = (root / "linux/manifest.json").read_bytes()
    with pytest.raises(ValueError, match="validated packages"):
        verify_release(root, "b" * 40, TAG, tmp_path)
    manifests = verify_release(root, COMMIT, TAG, tmp_path)
    assert manifests[0]["build_revision"] == COMMIT
    assert (root / "linux/manifest.json").read_bytes() == before


@pytest.mark.parametrize("report", ["abi3-audit", "linux-audit"])
def test_release_audit(tmp_path: Path, report: str) -> None:
    root = _release(tmp_path)
    document = (
        {"specs": {"other.whl": {"kind": "wheel"}}}
        if report == "abi3-audit"
        else {"wheel": "other.whl"}
    )
    _replace(root / "linux", report, json.dumps(document).encode())
    with pytest.raises(ValueError, match="audit"):
        verify_release(root, COMMIT, TAG, tmp_path)


def case_release_compiler(tmp_path: Path) -> None:
    root = _release(tmp_path)
    path = root / "linux/manifest.json"
    manifest = json.loads(path.read_text())
    manifest["toolchain"] = manifest["toolchain"].replace(
        TARGETS["linux"], TARGETS["windows"]
    )
    path.write_text(json.dumps(manifest))
    with pytest.raises(ValueError, match="native toolchain"):
        verify_release(root, COMMIT, TAG, tmp_path)


def case_release_distribution(tmp_path: Path) -> None:
    root = _release(tmp_path)
    path = root / "linux/manifest.json"
    manifest = json.loads(path.read_text())
    entry = next(item for item in manifest["files"] if item["role"] == "sdist")
    old = root / "linux" / entry["path"]
    entry["path"] = "other-1.2.3.tar.gz"
    old.rename(root / "linux" / entry["path"])
    path.write_text(json.dumps(manifest))
    with pytest.raises(ValueError, match="unified dokime distribution"):
        verify_release(root, COMMIT, TAG, tmp_path)


@pytest.mark.parametrize(
    "change", [None, "latest", "version", "signature", "empty", "foreign", "target"]
)
def test_signed_installer(tmp_path, monkeypatch, change):
    root = _release(tmp_path)
    project = "https://github.com/stokker-technologies/dokime"
    monkeypatch.setenv("GITHUB_SERVER_URL", project)
    installer = "dokime-1.2.3-windows-x86_64-setup.exe"
    target = {
        "signature": "signed fixture",
        "url": f"{project}/releases/download/{TAG}/{installer}",
    }
    metadata = {"version": "1.2.3", "platforms": {"windows-x86_64": target}}
    signature = "signed fixture"
    if change == "latest":
        target["url"] = target["url"].replace(TAG, "permalink/latest")
    elif change == "version":
        metadata["version"] = "1.2.4"
    elif change == "signature":
        target["signature"] = "different signature"
    elif change == "empty":
        target["signature"] = signature = ""
    elif change == "foreign":
        target["url"] = target["url"].replace(
            "stokker-technologies/dokime", "stokker-technologies/other"
        )
    elif change == "target":
        metadata["platforms"] = {"windows-aarch64": target}

    directory = root / "windows"
    path = directory / "manifest.json"
    manifest = json.loads(path.read_text())
    for role, name, content in [
        ("signature", f"{installer}.sig", signature + "\n"),
        ("updater", "latest.json", json.dumps(metadata)),
    ]:
        artifact = directory / name
        artifact.write_text(content)
        manifest["files"].append(
            {
                "role": role,
                "path": name,
                "bytes": artifact.stat().st_size,
                "sha256": digest(artifact),
            }
        )
    path.write_text(json.dumps(manifest))

    if change is None:
        verify_release(root, COMMIT, TAG, tmp_path)
    else:
        with pytest.raises(ValueError, match="exact tagged installer"):
            verify_release(root, COMMIT, TAG, tmp_path)


@pytest.fixture(autouse=True)
def resolved_inputs(monkeypatch):
    monkeypatch.setattr(checks, "source_revision", lambda root: COMMIT)
    monkeypatch.setattr(
        checks.release,
        "inputs",
        lambda root: {
            "source_tree": "c" * 40,
            "locks": digest(root / "Cargo.lock"),
        },
        raising=False,
    )


# === Private Functions === #


def _release(source: Path) -> Path:
    (source / "Cargo.lock").write_text("locked dependencies")
    (source / "mise.toml").write_text('[tools]\npython="3.13.15"\nuv="0.11.33"\n')
    sdk = source / "crates/dokime-python"
    sdk.mkdir(parents=True)
    (sdk / "pyproject.toml").write_text(
        '[build-system]\nrequires=["maturin==1.15.0"]\n'
    )
    root = source / "release"
    for platform in ("linux", "windows"):
        directory = root / platform
        directory.mkdir(parents=True)
        wheel = f"dokime-1.2.3-cp313-abi3-{'win_amd64' if platform == 'windows' else 'manylinux_2_28_x86_64'}.whl"
        files = {
            "wheel": (wheel, b"already verified native wheel fixture"),
            "abi3-audit": (
                "abi3.json",
                json.dumps(
                    {
                        "specs": {
                            wheel: {
                                "kind": "wheel",
                                "wheel": [
                                    {
                                        "name": "_native.pyd"
                                        if platform == "windows"
                                        else "_native.abi3.so",
                                        "result": {
                                            "is_abi3": True,
                                            "is_abi3_baseline_compatible": True,
                                            "baseline": "3.13",
                                            "non_abi3_symbols": [],
                                            "future_abi3_objects": {},
                                        },
                                    }
                                ],
                            }
                        }
                    }
                ).encode(),
            ),
        }
        for name in stage.SUITES:
            files[f"test:tests-{name}.xml"] = (
                f"tests-{name}.xml",
                b"<testsuite><testcase name='native'/></testsuite>",
            )
        if platform == "linux":
            files.update(
                {
                    "deb": ("dokime_1.2.3_amd64.deb", b"deb fixture"),
                    "rpm": ("dokime-1.2.3-1.x86_64.rpm", b"rpm fixture"),
                    "sdist": ("dokime-1.2.3.tar.gz", b"sdist fixture"),
                    "linux-audit": (
                        "linux.json",
                        json.dumps(
                            {
                                "version": 1,
                                "wheel": wheel,
                                "pure": False,
                                "overall_tag": "manylinux_2_28_x86_64",
                                "unsupported_isa": False,
                            }
                        ).encode(),
                    ),
                }
            )
        else:
            files["installer"] = (
                "dokime-1.2.3-windows-x86_64-setup.exe",
                b"installer fixture",
            )
        files.update(
            {
                role: (role, role.encode())
                for role in (
                    "host",
                    "tool",
                    "desktop",
                    "plugin",
                    "protocol",
                    "protocol-checksum",
                )
            }
        )
        (directory / "licenses").mkdir()
        (directory / "reports").mkdir()
        files["report-template"] = (
            "reports/report.html",
            "<html>“Original Report” · Температура</html>".encode("utf-8"),
        )
        files["license"] = ("licenses/LICENSE-APACHE", b"Apache license fixture")
        files["sdist"] = ("dokime-1.2.3.tar.gz", b"sdist fixture")
        if platform == "linux":
            from dokime_tools.package.linux import IMAGE, PACKAGES

            files["wheel-build"] = (
                "wheel-build.json",
                json.dumps(
                    {
                        "version": 1,
                        "image": IMAGE,
                        "compatibility": "manylinux_2_28",
                        "rustc": "original version",
                        "python": "Python 3.13.15",
                        "uv": "uv 0.11.33",
                        "maturin": "maturin 1.15.0",
                        "packages": list(PACKAGES),
                        "wheel_sha256": checks.hashlib.sha256(
                            files["wheel"][1]
                        ).hexdigest(),
                        "sdist_sha256": checks.hashlib.sha256(
                            files["sdist"][1]
                        ).hexdigest(),
                    }
                ).encode(),
            )
        artifacts = []
        for role, (name, content) in files.items():
            (directory / name).write_bytes(content)
            artifacts.append(
                {
                    "role": role,
                    "path": name,
                    "bytes": len(content),
                    "sha256": digest(directory / name),
                }
            )
        manifest = {
            "version": 1,
            "build_revision": COMMIT,
            "package_version": TAG[1:],
            "tools": {
                name: "original version"
                for name in ("mise", "rustc", "cargo", "python", "uv", "node", "pnpm")
            },
            "platform": platform,
            "architecture": "x86_64",
            "python": "cp313",
            "abi": "abi3",
            "profile": "release",
            "target": TARGETS[platform],
            "toolchain": f"rustc 1.99.0 (fixture)\nrelease: 1.99.0\nhost: {TARGETS[platform]}",
            "inputs": checks.inputs(source),
            "files": artifacts,
            "validation": {"suites": list(stage.SUITES)},
        }
        (directory / "manifest.json").write_text(json.dumps(manifest))
    return root


def _replace(root: Path, role: str, content: bytes) -> None:
    path = root / "manifest.json"
    manifest = json.loads(path.read_text())
    artifact = next(item for item in manifest["files"] if item["role"] == role)
    (root / artifact["path"]).write_bytes(content)
    artifact["bytes"] = len(content)
    artifact["sha256"] = digest(root / artifact["path"])
    path.write_text(json.dumps(manifest))


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_release_pair, id="release_pair"),
        pytest.param(case_release_wheel, id="release_wheel"),
        pytest.param(case_release_compiler, id="release_compiler"),
        pytest.param(case_release_distribution, id="release_distribution"),
    ],
)
def test_contract(scenario: Callable[..., None], tmp_path: Path) -> None:
    scenario(tmp_path)
