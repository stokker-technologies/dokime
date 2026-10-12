# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Preserve Installer Sequencing And Signed Artifacts"""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable

import json
from pathlib import Path


# Third Party Imports

import pytest


# Local Imports

from dokime_tools import installer


# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("missing", [None, "deb", "rpm"])
def test_linux_names(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, missing: str | None
) -> None:
    _metadata(tmp_path)
    monkeypatch.setattr(installer.sys, "platform", "linux")
    monkeypatch.delenv("GITHUB_REF_TYPE", raising=False)
    bundles = tmp_path / "target/release/bundle"
    for package, filename in (
        ("deb", "Dokime_1.2.3_amd64.deb"),
        ("rpm", "Dokime-1.2.3-1.x86_64.rpm"),
    ):
        directory = bundles / package
        directory.mkdir(parents=True)
        if package != missing:
            (directory / filename).write_bytes(package.encode())
    if missing:
        with pytest.raises(ValueError, match="expected one"):
            installer.collect(
                tmp_path, profile="release", signed=False, bundles=bundles
            )
        assert not (tmp_path / installer.LINUX_DIST).exists()
    else:
        installer.collect(tmp_path, profile="release", signed=False, bundles=bundles)
        destination = tmp_path / installer.LINUX_DIST
        assert (destination / "dokime_1.2.3_amd64.deb").read_bytes() == b"deb"
        assert (destination / "dokime-1.2.3-1.x86_64.rpm").read_bytes() == b"rpm"


def case_build_sequence(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(installer.sys, "platform", "win32")
    monkeypatch.setattr(installer, "_target_directory", lambda _: tmp_path / "target")
    for signed in [False, True]:
        for failure in [None, "check", "build"]:
            calls: list[str] = []

            def record(phase: str) -> None:
                calls.append(phase)
                if phase == failure:
                    raise ValueError("build failed")

            monkeypatch.setattr(installer, "check", lambda _: record("check"))
            monkeypatch.setattr(installer, "_signing_available", lambda: signed)

            def run(command, **options):
                assert command[:2] == [
                    "node",
                    "./node_modules/@tauri-apps/cli/tauri.js",
                ]
                assert options["cwd"] == tmp_path / installer.WEB
                assert ("--no-sign" in command) == (not signed)
                record("build")

            monkeypatch.setattr(installer.subprocess, "run", run)

            def collect(root, **options):
                assert root == tmp_path
                assert options["signed"] == signed
                assert options["profile"] == "release"
                record("collect")

            monkeypatch.setattr(installer, "collect", collect)
            if failure:
                with pytest.raises(ValueError, match="build failed"):
                    installer.build(tmp_path)
                assert calls == (
                    ["check"] if failure == "check" else ["check", "build"]
                )
            else:
                installer.build(tmp_path)
                assert calls == ["check", "build", "collect"]


def case_signed_artifacts(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    _metadata(tmp_path)
    monkeypatch.setattr(installer.sys, "platform", "win32")
    monkeypatch.delenv("GITHUB_REF_NAME", raising=False)
    monkeypatch.delenv("TAURI_UPDATER_ARTIFACT_BASE_URL", raising=False)
    monkeypatch.setenv("GITHUB_REF_TYPE", "tag")
    monkeypatch.delenv("EXPECTED_PACKAGE_VERSION", raising=False)
    nsis = tmp_path / "target/release/bundle/nsis"
    nsis.mkdir(parents=True)
    (nsis / "Dokime_setup.exe").write_bytes(b"installer")
    (nsis / "Dokime_setup.exe.sig").write_text(" signature \n")
    installer.collect(tmp_path, profile="release", signed=True, bundles=nsis.parent)
    directory = tmp_path / installer.DIST
    name = "dokime-1.2.3-windows-x86_64-setup.exe"
    assert (directory / name).read_bytes() == b"installer"
    assert (directory / (name + ".sig")).read_text().strip() == "signature"
    path = directory / "latest.json"
    original = json.loads(path.read_text())
    assert original["version"] == "1.2.3"
    assert (
        original["platforms"]["windows-x86_64"]["url"]
        == f"https://github.com/stokker-technologies/dokime/releases/download/v1.2.3/{name}"
    )
    monkeypatch.setattr(installer, "check", lambda _: None)
    installer.check_package(tmp_path, require_signed=True)
    for key, value in [
        ("signature", "wrong"),
        ("url", "https://example.test/other.exe"),
    ]:
        mutated = json.loads(json.dumps(original))
        mutated["platforms"]["windows-x86_64"][key] = value
        path.write_text(json.dumps(mutated))
        with pytest.raises(ValueError, match="exact installer"):
            installer.check_package(tmp_path, require_signed=True)
    path.write_text(json.dumps(original))
    with pytest.raises(ValueError, match="version"):
        installer.check_package(tmp_path, expected="9.9.9")
    installer.collect(tmp_path, profile="release", signed=False, bundles=nsis.parent)
    assert not path.exists()
    assert not (directory / (name + ".sig")).exists()
    assert (directory / name).read_bytes() == b"installer"
    installer.collect(tmp_path, profile="dev", signed=False, bundles=nsis.parent)
    assert (nsis / "BUILD-PROFILE").read_text() == "dev\n"
    assert (directory / name).read_bytes() == b"installer"
    monkeypatch.setattr(installer.sys, "platform", "linux")
    linux = tmp_path / "target/debug/bundle"
    installer.collect(tmp_path, profile="dev", signed=False, bundles=linux)
    assert (linux / "BUILD-PROFILE").read_text() == "dev\n"


def case_companion_staging(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    for triple, extension in [
        ("x86_64-pc-windows-msvc", ".exe"),
        ("x86_64-unknown-linux-gnu", ""),
    ]:
        executable = tmp_path / ("built" + extension)
        executable.write_bytes(b"native companion")
        calls = []

        def output(command, **options):
            calls.append(command)
            assert options["cwd"] == tmp_path
            return (
                json.dumps(
                    {
                        "reason": "compiler-artifact",
                        "target": {"name": "dokime"},
                        "executable": str(executable),
                    }
                )
                + "\n"
            )

        monkeypatch.setattr(installer.subprocess, "check_output", output)
        installer.stage_resources(tmp_path, profile="dev", target=triple)
        destination = (
            tmp_path / installer.TAURI / "binaries" / f"dokime-{triple}{extension}"
        )
        assert destination.read_bytes() == executable.read_bytes()
        assert calls[0][:4] == ["mise", "exec", "--", "cargo"]
        assert calls[0][-2:] == ["--target", triple]
        metadata = destination.stat()
        installer.stage_resources(tmp_path, profile="dev", target=triple)
        assert destination.stat().st_ino == metadata.st_ino
        assert destination.stat().st_mtime_ns == metadata.st_mtime_ns


# === Private Functions === #


def _metadata(root: Path) -> None:
    config = root / installer.TAURI / "tauri.conf.json"
    config.parent.mkdir(parents=True, exist_ok=True)
    config.write_text('{"version": "1.2.3"}')
    brand = root / "branding/brand.toml"
    brand.parent.mkdir()
    brand.write_text('[app]\nslug = "dokime"\n')


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_build_sequence, id="build_sequence"),
        pytest.param(case_signed_artifacts, id="signed_artifacts"),
        pytest.param(case_companion_staging, id="companion_staging"),
    ],
)
def test_contract(
    scenario: Callable[..., None], tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    scenario(tmp_path, monkeypatch)
