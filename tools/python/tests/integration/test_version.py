# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Preserve Version Metadata Consistency"""

# === Imports === #

# Standard Library Imports
import os
import tomllib
from collections.abc import Callable
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools import version

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def case_metadata_sync(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    _metadata(tmp_path)
    version.check(tmp_path)
    monkeypatch.setenv("DOKIME_VERSION_MODE", "release")
    monkeypatch.setenv("DOKIME_VERSION_RELEASE", "0.4.9")
    calls = []

    def native_lock(command, **kwargs):
        calls.append(command)
        current = tomllib.loads((tmp_path / "Cargo.toml").read_text())["workspace"][
            "package"
        ]["version"]
        if command[0] == "cargo":
            text = (
                f'[[package]]\nname = "dokime"\nversion = "{current}"\n'
                f'[[package]]\nname = "dokime-macros"\nversion = "{current}"\n'
                '[[package]]\nname = "external"\nversion = "9.9.9"\nsource = "registry+https://example.test"\n'
            )
            (tmp_path / "Cargo.lock").write_text(text)
        else:
            (tmp_path / "uv.lock").write_text(
                f'[[package]]\nname = "dokime-rust-workspace"\nversion = "{version.python_version(current)}"\n'
                f'[[package]]\nname = "dokime"\nversion = "{version.python_version(current)}"\n[package.source]\neditable = "packages/dokime"\n'
            )
        assert kwargs == {"cwd": tmp_path, "check": True}

    monkeypatch.setattr(version.subprocess, "run", native_lock)
    version.sync(tmp_path)
    version.check(tmp_path)
    assert calls == [["cargo", "update", "--workspace"], ["uv", "lock"]]
    lock = tomllib.loads((tmp_path / "Cargo.lock").read_text())["package"]
    assert [package["version"] for package in lock] == ["0.4.9", "0.4.9", "9.9.9"]
    before = {p: p.read_bytes() for p in tmp_path.rglob("*") if p.is_file()}
    version.sync(tmp_path)
    assert before == {p: p.read_bytes() for p in tmp_path.rglob("*") if p.is_file()}
    monkeypatch.setenv("DOKIME_VERSION_MODE", "dev")
    with pytest.raises(ValueError, match="DOKIME_VERSION_MODE=release"):
        version.sync(tmp_path)
    version.run(tmp_path, "sync", value="0.5.0")
    version.run(tmp_path, "check")
    assert (
        tomllib.loads((tmp_path / "Cargo.toml").read_text())["workspace"]["package"][
            "version"
        ]
        == "0.5.0"
    )
    assert os.environ["DOKIME_VERSION_MODE"] == "dev"
    assert os.environ["DOKIME_VERSION_RELEASE"] == "0.4.9"
    before = {p: p.read_bytes() for p in tmp_path.rglob("*") if p.is_file()}
    for verb, value in [("sync", "0.5"), ("check", "0.5.0"), ("unknown", None)]:
        with pytest.raises(ValueError):
            version.run(tmp_path, verb, value=value)
        assert before == {p: p.read_bytes() for p in tmp_path.rglob("*") if p.is_file()}


def test_client_metadata(tmp_path: Path) -> None:
    _metadata(tmp_path)
    path = tmp_path / "uv.lock"
    original = path.read_text()
    for text, message in [
        (
            original.replace(
                'editable = "packages/dokime"',
                'editable = "packages/incorrect"',
            ),
            "dokime source",
        ),
        (
            original.replace(
                'name = "dokime"\nversion = "0.1.0"',
                'name = "dokime"\nversion = "0.0.0"',
            ),
            "dokime version",
        ),
    ]:
        path.write_text(text)
        with pytest.raises(ValueError, match=message):
            version.check(tmp_path)
        assert path.read_text() == text
    path.write_text(original)
    version.check(tmp_path)


def test_version_rewrites() -> None:
    source = '[workspace.package]\nname = "x"\nversion = "0.1.0"\n[other]\nversion = "9.0.0"\n'
    assert version.replace_section(
        source, "workspace.package", "1.0.0"
    ) == source.replace('version = "0.1.0"', 'version = "1.0.0"')
    with pytest.raises(ValueError):
        version.replace_section(source, "absent", "1.0.0")
    with pytest.raises(ValueError, match="stable"):
        version.python_version("1.2.3-dev.0")
    assert version.python_version("1.2.3") == "1.2.3"


def case_forced_version(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    _metadata(tmp_path)
    for mode, release in [
        ("invalid", "1.2.3"),
        ("dev", ""),
        ("", "1.2.3"),
        ("release", "1.2"),
    ]:
        monkeypatch.setenv("DOKIME_VERSION_MODE", mode)
        monkeypatch.setenv("DOKIME_VERSION_RELEASE", release)
        before = (tmp_path / "Cargo.toml").read_bytes()
        with pytest.raises(ValueError):
            version.sync(tmp_path)
        assert (tmp_path / "Cargo.toml").read_bytes() == before


def case_tag_resolution(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    _metadata(tmp_path)
    (tmp_path / ".jj").mkdir()
    for key in ("DOKIME_VERSION_MODE", "DOKIME_VERSION_RELEASE", "GITHUB_REF_NAME"):
        monkeypatch.delenv(key, raising=False)
    calls: list[list[str]] = []
    answers = iter(["v1.2.3\n"])

    def output(command: list[str], **_kwargs: object) -> str:
        calls.append(command)
        return next(answers)

    monkeypatch.setattr(version.subprocess, "check_output", output)
    assert version._version_target(tmp_path) == ("1.2.3", "release")
    assert calls[0][:2] == ["jj", "log"]
    assert len(calls) == 1
    path = tmp_path / "Cargo.toml"
    path.write_text(path.read_text().replace('version = "0.1.0"', 'version = "9.0.0"'))
    for tags, expected in [("v0.1.0\n", "0.2.0"), ("", "9.0.0")]:
        calls.clear()
        answers = iter(["", tags, "/repo/.jj/repo/store/git\n", "v0.2.0\n"])
        assert version._version_target(tmp_path) == (expected, "release")
        assert calls[-1] == [
            "git-cliff",
            "--repository",
            "/repo/.jj/repo/store/git",
            "--bumped-version",
            "--bump",
            "patch",
        ]
    monkeypatch.setenv("GITHUB_REF_TYPE", "tag")
    monkeypatch.setenv("GITHUB_REF_NAME", "v2.3.4")
    assert version._version_target(tmp_path) == ("2.3.4", "release")
    monkeypatch.setenv("GITHUB_REF_TYPE", "tag")
    monkeypatch.setenv("GITHUB_REF_NAME", "invalid")
    with pytest.raises(ValueError, match="GITHUB_REF_NAME"):
        version._version_target(tmp_path)


def case_prepare(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    (tmp_path / "stkr.toml").write_text("")
    branch = "release/1.2.3"
    calls = []

    def output(command, **kwargs):
        calls.append(command)
        if command[0] == "git-cliff":
            assert command[-2:] == ["--bump", "patch"]
            return "v1.2.3"
        return branch if command[1] == "branch" else ""

    monkeypatch.setattr(version.subprocess, "check_output", output)
    stamped = []
    monkeypatch.setattr(
        version,
        "sync",
        lambda root, *, value: stamped.append((root, value)),
    )
    monkeypatch.setattr(version, "check", lambda root: None)
    monkeypatch.setattr(
        version.subprocess, "run", lambda command, **kwargs: calls.append(command)
    )
    monkeypatch.delenv("DOKIME_VERSION_RELEASE", raising=False)
    version.prepare(tmp_path)
    assert stamped == [(tmp_path, "1.2.3")]
    assert "DOKIME_VERSION_RELEASE" not in os.environ
    assert calls[-1] == [
        "git-cliff",
        "--config",
        "cliff.toml",
        "--bump",
        "patch",
        "--tag",
        "v1.2.3",
        "--output",
        "CHANGELOG.md",
    ]
    branch = "feat/feature"
    with pytest.raises(ValueError, match="prepare requires"):
        version.prepare(tmp_path)
    assert len(stamped) == 1


# === Private Functions === #


def _metadata(root: Path) -> None:
    files = {
        "Cargo.toml": '[workspace]\nmembers = ["crates/dokime", "crates/dokime-macros"]\n[workspace.package]\nversion = "0.1.0"\n',
        "crates/dokime/Cargo.toml": '[package]\nname = "dokime"\n',
        "crates/dokime-macros/Cargo.toml": '[package]\nname = "dokime-macros"\n',
        "Cargo.lock": '[[package]]\nname = "dokime"\nversion = "0.1.0"\n[[package]]\nname = "dokime-macros"\nversion = "0.1.0"\n[[package]]\nname = "external"\nversion = "9.9.9"\nsource = "registry+https://example.test"\n',
        "pyproject.toml": '[project]\nname = "dokime-rust-workspace"\nversion = "0.1.0"\n',
        "packages/dokime/pyproject.toml": '[project]\nname = "dokime"\nversion = "0.1.0"\n',
        "uv.lock": '[[package]]\nname = "dokime-rust-workspace"\nversion = "0.1.0"\n[[package]]\nname = "dokime"\nversion = "0.1.0"\n[package.source]\neditable = "packages/dokime"\n',
        "crates/dokime-gui/package.json": '{\n  "version": "0.1.0"\n}\n',
        "crates/dokime-gui/src-tauri/tauri.conf.json": '{\n  "version": "0.1.0"\n}\n',
        "crates/dokime-gui/src-tauri/Cargo.toml": '[package]\nname = "dokime-gui"\nversion.workspace = true\n',
        "pnpm-lock.yaml": "importers:\n  crates/dokime-gui:\n",
        "CHANGELOG.md": "# Changelog\n\n## [Unreleased]\n",
    }
    for relative, text in files.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_prepare, id="prepare"),
        pytest.param(case_metadata_sync, id="metadata_sync"),
        pytest.param(case_forced_version, id="forced_version"),
        pytest.param(case_tag_resolution, id="tag_resolution"),
    ],
)
def test_contract(
    scenario: Callable[..., None], tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    scenario(tmp_path, monkeypatch)
