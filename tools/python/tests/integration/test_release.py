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
import base64
import hashlib
import io
import json
import os
import subprocess
import sys
from collections.abc import Callable
from pathlib import Path
from zipfile import ZipFile

# Third Party Imports
import pytest

# Local Imports
from dokime_tools import _policy, evidence
from dokime_tools.check import release as checks
from dokime_tools.check.release import (
    TARGETS,
    digest,
    verify_release,
)
from dokime_tools.check.wheel import verify_wheel
from dokime_tools.ci import artifacts as native
from dokime_tools.package import artifacts as stage
from dokime_tools.release import inputs as source_inputs

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
        template = package / checks.REPORT_TEMPLATE
        original = template.read_bytes()
        archive = downloads / f"dokime-{platform}.zip"
        with ZipFile(archive, "w") as contents:
            for path in package.rglob("*"):
                if path.is_file():
                    contents.write(path, path.relative_to(package))
        native.restore(tmp_path, platform)
        restored = tmp_path / "dist/build" / platform / checks.REPORT_TEMPLATE
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


def test_windows_installed_evidence(tmp_path: Path) -> None:
    root = _release(tmp_path)
    path = root / "windows/manifest.json"
    manifest = json.loads(path.read_text())
    manifest["validation"]["suites"].remove("system-javascript")
    path.write_text(json.dumps(manifest))
    with pytest.raises(ValueError, match="complete native test evidence"):
        verify_release(root, COMMIT, TAG, tmp_path)


@pytest.mark.parametrize("change", ["changed", "missing", "unlisted", "dependency"])
def test_release_refuses(tmp_path: Path, change: str) -> None:
    root = _release(tmp_path)
    wheel = next((root / "linux/python").glob("*.whl"))
    if change == "changed":
        wheel.write_bytes(b"different wheel")
    elif change == "missing":
        wheel.unlink()
    elif change == "unlisted":
        (root / "linux/python/unverified.whl").write_bytes(b"extra")
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


@pytest.mark.parametrize(
    "change",
    [
        "misplaced-binary",
        "nested-archive",
        "foreign-role",
        "proto-unlisted",
        "proto-revision",
        "proto-archive",
        "example-missing",
        "example-empty",
        "example-misplaced",
    ],
)
def test_release_layout(tmp_path: Path, change: str) -> None:
    root = _release(tmp_path)
    package = root / "linux"
    path = package / "manifest.json"
    manifest = json.loads(path.read_text())
    roles = {item["role"]: item for item in manifest["files"]}

    def move(role: str, target: str) -> None:
        entry = roles[role]
        (package / target).parent.mkdir(parents=True, exist_ok=True)
        (package / entry["path"]).rename(package / target)
        entry["path"] = target

    if change == "misplaced-binary":
        move("host", "runtime/host")
    elif change == "nested-archive":
        move("plugin", "plugins/plugin.tar")
    elif change == "foreign-role":
        foreign = package / "contracts/revision.txt"
        foreign.parent.mkdir()
        foreign.write_bytes(b"foreign")
        manifest["files"].append(
            {
                "role": "contracts",
                "path": "contracts/revision.txt",
                "bytes": foreign.stat().st_size,
                "sha256": digest(foreign),
            }
        )
    elif change == "proto-unlisted":
        extra = package / checks.PROTO / "extra.proto"
        extra.write_bytes(b"unlisted")
        manifest["files"].append(
            {
                "role": "proto:extra.proto",
                "path": f"{checks.PROTO}/extra.proto",
                "bytes": extra.stat().st_size,
                "sha256": digest(extra),
            }
        )
    elif change == "proto-archive":
        del manifest["contracts"]
    elif change == "example-missing":
        entry = roles[f"example:{checks.EXAMPLE_REPORT}"]
        (package / entry["path"]).unlink()
        manifest["files"].remove(entry)
    elif change == "example-misplaced":
        move(f"example:{checks.EXAMPLE_REPORT}", checks.EXAMPLE_REPORT)
    path.write_text(json.dumps(manifest))
    if change == "proto-revision":
        _replace(package, "proto:revision.txt", b"b" * 40 + b"\n")
    elif change == "example-empty":
        _replace(package, f"example:{checks.EXAMPLE_REPORT}", b"  \n")
    with pytest.raises(ValueError, match="layout|directory|archive|protocol|example"):
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


@pytest.mark.parametrize(
    "change",
    [
        "missing-proof",
        "wrong-inputs",
        "missing-suite",
        "ordinary-report",
        "empty-report",
    ],
)
def test_release_coverage(tmp_path: Path, change: str) -> None:
    root = _release(tmp_path)
    package = root / "linux"
    manifest_path = package / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    if change == "missing-proof":
        entry = next(
            item
            for item in manifest["files"]
            if item["role"] == "coverage:coverage.json"
        )
        (package / entry["path"]).unlink()
        manifest["files"].remove(entry)
        manifest_path.write_text(json.dumps(manifest))
    else:
        proof = json.loads(
            (package / checks.COVERAGE_EVIDENCE / "coverage.json").read_text()
        )
        if change == "wrong-inputs":
            proof["inputs"] = {}
        elif change == "missing-suite":
            proof["suites"].pop()
        else:
            name = "latest.json" if change == "ordinary-report" else "coverage-rust.xml"
            if change == "ordinary-report":
                report = json.loads(
                    (package / checks.COVERAGE_EVIDENCE / "latest.json").read_text()
                )
                report["tests"]["integration-rust"]["coverage"] = False
                content = json.dumps(report).encode()
            else:
                content = b'<coverage lines-valid="0"/>'
            _replace(package, f"coverage:{name}", content)
            entry = next(item for item in proof["files"] if item["path"] == name)
            entry.update(bytes=len(content), sha256=hashlib.sha256(content).hexdigest())
        _replace(package, "coverage:coverage.json", json.dumps(proof).encode())
    with pytest.raises(
        (ValueError, evidence.EvidenceError), match="coverage|executable"
    ):
        verify_release(root, COMMIT, TAG, tmp_path)


def case_release_wheel(tmp_path: Path) -> None:
    root = _release(tmp_path)
    before = (root / "linux/manifest.json").read_bytes()
    with pytest.raises(ValueError, match="validated packages"):
        verify_release(root, "b" * 40, TAG, tmp_path)
    manifests = verify_release(root, COMMIT, TAG, tmp_path)
    assert manifests[0]["build_revision"] == COMMIT
    assert (root / "linux/manifest.json").read_bytes() == before


@pytest.mark.parametrize("field", ["wheel", "sha256", "pure", "files"])
def test_release_audit(tmp_path: Path, field: str) -> None:
    root = _release(tmp_path)
    manifest = json.loads((root / "linux/manifest.json").read_text())
    report = next(item for item in manifest["files"] if item["role"] == "wheel-audit")
    document = json.loads((root / "linux" / report["path"]).read_text())
    document[field] = "tampered"
    _replace(root / "linux", "wheel-audit", json.dumps(document).encode())
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
    entry["path"] = "python/other-1.2.3.tar.gz"
    old.rename(root / "linux" / entry["path"])
    path.write_text(json.dumps(manifest))
    with pytest.raises(ValueError, match="dokime distribution"):
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
        ("signature", f"installers/{installer}.sig", signature + "\n"),
        ("updater", "installers/latest.json", json.dumps(metadata)),
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
    root = source / "release"
    for platform in ("linux", "windows"):
        directory = root / platform
        directory.mkdir(parents=True)
        wheel = "dokime-1.2.3-py3-none-any.whl"
        content = _wheel()
        wheel_path = directory / "python" / wheel
        wheel_path.parent.mkdir()
        wheel_path.write_bytes(content)
        audit = verify_wheel(wheel_path)["wheel-audit"]
        files = {
            "wheel": (f"python/{wheel}", content),
            "wheel-audit": (f"{checks.AUDITS}/wheel-audit.json", audit.encode()),
            "sdist": ("python/dokime-1.2.3.tar.gz", b"sdist fixture"),
        }
        for name in checks.required_suites(platform):
            files[f"test:tests-{name}.xml"] = (
                f"{checks.TEST_EVIDENCE}/tests-{name}.xml",
                b"<testsuite><testcase name='native'/></testsuite>",
            )
        if platform == "linux":
            files.update(
                {
                    "deb": ("installers/dokime_1.2.3_amd64.deb", b"deb fixture"),
                    "rpm": ("installers/dokime-1.2.3-1.x86_64.rpm", b"rpm fixture"),
                }
            )
        else:
            files["installer"] = (
                "installers/dokime-1.2.3-windows-x86_64-setup.exe",
                b"installer fixture",
            )
        files.update(
            {
                role: (f"{checks.BINARIES}/{role}", role.encode())
                for role in ("host", "client", "tool", "desktop")
            }
        )
        files["plugin"] = ("plugins/plugin.dokime", b"plugin")
        files["report-template"] = (
            checks.REPORT_TEMPLATE,
            "<html>“Original Report” · Температура</html>".encode(),
        )
        files["license"] = ("licenses/LICENSE-APACHE", b"Apache license fixture")
        files[f"example:{checks.EXAMPLE_REPORT}"] = (
            f"{checks.EXAMPLES}/{checks.EXAMPLE_REPORT}",
            "<html>Sealed charger report · Отчёт</html>".encode(),
        )
        protocol = {
            "contracts.json": b"{}",
            "descriptors.bin": b"descriptors",
            "dokime/operations.proto": b'syntax = "proto3";',
        }
        protocol["manifest.txt"] = "".join(f"{name}\n" for name in protocol).encode()
        protocol["revision.txt"] = f"{COMMIT}\n".encode("ascii")
        for name, data in protocol.items():
            files[f"proto:{name}"] = (f"{checks.PROTO}/{name}", data)
        for name, _, _ in stage.FIXTURES:
            files[f"fixture:{name}"] = (
                f"{checks.FIXTURE_BUNDLES}/{name}/{name}.dokime",
                name.encode(),
            )
        files["archive:fixture.tar.zst"] = (
            f"{checks.TEST_ARCHIVES}/fixture.tar.zst",
            b"release test executables",
        )
        files["archive:fixture.tar.json"] = (
            f"{checks.TEST_ARCHIVES}/fixture.tar.json",
            b"archive receipt",
        )
        coverage_suites = (
            "integration-rust",
            "acceptance-rust",
            "integration-python",
            "acceptance-python",
        )
        for suite in coverage_suites:
            files[f"coverage:tests-{suite}.xml"] = (
                f"{checks.COVERAGE_EVIDENCE}/tests-{suite}.xml",
                b"<testsuite><testcase name='native'/></testsuite>",
            )
        for language in ("rust", "python"):
            files[f"coverage:coverage-{language}.xml"] = (
                f"{checks.COVERAGE_EVIDENCE}/coverage-{language}.xml",
                b'<coverage lines-valid="1" lines-covered="1"><packages><package name="fixture"><classes><class name="fixture" filename="fixture.rs"><lines><line number="1" hits="1"/></lines></class></classes></package></packages></coverage>',
            )
        files["coverage:latest.json"] = (
            f"{checks.COVERAGE_EVIDENCE}/latest.json",
            json.dumps(
                {
                    "tests": {
                        suite: {
                            "outcome": "passed",
                            "coverage": True,
                            "revision": COMMIT,
                            "platform": platform,
                            "requested_configuration": None,
                        }
                        for suite in coverage_suites
                    }
                }
            ).encode(),
        )
        proof = {
            "version": 1,
            "platform": platform,
            "inputs": checks.inputs(source),
            "pipeline_id": "2",
            "run_attempt": "1",
            "coverage_job_id": "30",
            "build_manifest_sha256": "a" * 64,
            "suites": list(coverage_suites),
            "files": [
                {
                    "path": Path(name).name,
                    "bytes": len(content),
                    "sha256": hashlib.sha256(content).hexdigest(),
                }
                for role, (name, content) in files.items()
                if role.startswith("coverage:")
            ],
        }
        files["coverage:coverage.json"] = (
            f"{checks.COVERAGE_EVIDENCE}/coverage.json",
            json.dumps(proof).encode(),
        )
        artifacts = []
        for role, (name, content) in files.items():
            (directory / name).parent.mkdir(parents=True, exist_ok=True)
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
            "python": "py3",
            "abi": "none",
            "profile": "release",
            "target": TARGETS[platform],
            "toolchain": f"rustc 1.99.0 (fixture)\nrelease: 1.99.0\nhost: {TARGETS[platform]}",
            "inputs": checks.inputs(source),
            "contracts": {"archive_sha256": "e" * 64},
            "files": artifacts,
            "pipeline_id": "2",
            "run_attempt": "1",
            "validation": {
                "suites": list(checks.required_suites(platform)),
                "coverage_job_id": "30",
                "build_manifest_sha256": "a" * 64,
            },
        }
        (directory / "manifest.json").write_text(json.dumps(manifest))
    return root


def _wheel() -> bytes:
    root = "dokime-1.2.3.dist-info"
    payload = {
        "dokime/__init__.py": b"# Client",
        f"{root}/METADATA": b"Metadata-Version: 2.4\nName: dokime\nVersion: 1.2.3\n",
        f"{root}/WHEEL": b"Wheel-Version: 1.0\nRoot-Is-Purelib: true\nTag: py3-none-any\n",
    }
    rows = []
    for name, content in payload.items():
        digest = (
            base64.urlsafe_b64encode(hashlib.sha256(content).digest())
            .rstrip(b"=")
            .decode()
        )
        rows.append(f"{name},sha256={digest},{len(content)}\n")
    rows.append(f"{root}/RECORD,,\n")
    payload[f"{root}/RECORD"] = "".join(rows).encode()
    stream = io.BytesIO()
    with ZipFile(stream, "w") as archive:
        for name, content in payload.items():
            archive.writestr(name, content)
    return stream.getvalue()


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
