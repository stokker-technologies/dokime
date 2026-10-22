# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Keep Updater Verification Independent Of Private Signing Credentials"""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import shutil
from pathlib import Path
from types import SimpleNamespace

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.build import organization
from dokime_tools.package import installer, linux

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_organization_overlay_preserves_native_window_policy(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    assets = tmp_path / "organization"
    shutil.copytree(organization.BRAND, assets)
    config = tmp_path / "crates/dokime-gui/src-tauri/tauri.conf.json"
    config.parent.mkdir(parents=True)
    window = {"create": False, "dragDropEnabled": True, "maximized": True, "minWidth": 1080, "title": "Source Title"}
    config.write_text(json.dumps({"app": {"windows": [window]}}), encoding="utf-8")
    monkeypatch.setattr(organization, "ROOT", tmp_path)
    monkeypatch.setattr(organization, "BRAND", assets)

    organization.generate()

    overlay = json.loads((assets / "generated/tauri.organization.json").read_text())
    assert overlay["identifier"] == "com.stokker.dokime"
    assert overlay["app"]["windows"] == [{**window, "title": "Dokime"}]
    stylesheet = (assets / "generated/theme.css").read_text()
    assert "data:font/woff2;base64," in stylesheet
    assert 'url("fonts/' not in stylesheet
    assert "--brand-radius-sm: 0.0625rem" in stylesheet


@pytest.mark.parametrize("provider", ["github", "gitlab"])
def test_tag_requires_signature(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, provider: str) -> None:
    root, dist = _candidate(tmp_path, monkeypatch)
    _tag(monkeypatch, provider)

    # A Public Verifier Has No Private Key And Must Still Refuse Unsigned Bytes
    with pytest.raises(FileNotFoundError):
        installer.check_package(root, dist=dist, profile="release")


def test_tag_refuses_unsigned_bundle(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root, _dist = _candidate(tmp_path, monkeypatch)
    _tag(monkeypatch, "github")
    monkeypatch.setattr(installer.sys, "platform", "win32")

    # Missing Credentials Fail Before Reading Or Changing Build Outputs
    with pytest.raises(ValueError, match="signing key and password"):
        installer.bundle(root, profile="release")
    assert not (root / "target").exists()


def test_candidate_verifies_signature(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root, dist = _candidate(tmp_path, monkeypatch)
    payload = next(dist.glob("*.exe"))
    signature = payload.with_suffix(".exe.sig")
    signature.write_text("candidate signature", encoding="utf-8")
    metadata = {
        "version": "0.1.0",
        "platforms": {installer.UPDATER_TARGET: {"signature": "candidate signature", "url": f"https://example.com/{payload.name}"}},
    }
    (dist / "latest.json").write_text(json.dumps(metadata), encoding="utf-8")

    def refuse_signature(checked_root: Path, checked_payload: Path, checked_signature: Path) -> None:
        assert (checked_root, checked_payload, checked_signature) == (root, payload, signature)
        raise ValueError("cryptographic refusal")

    monkeypatch.setattr(installer, "verify_signature", refuse_signature)

    # Optional Candidate Signing Does Not Permit Invalid Signatures On A Branch
    with pytest.raises(ValueError, match="cryptographic refusal"):
        installer.check_package(root, dist=dist)

    # A Manifest Naming Another Download Is Rejected Before Verification
    metadata["platforms"][installer.UPDATER_TARGET]["url"] += ".other"
    (dist / "latest.json").write_text(json.dumps(metadata), encoding="utf-8")
    with pytest.raises(ValueError, match="exact installer and signature"):
        installer.check_package(root, dist=dist)


def test_development_unsigned_candidate(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root, dist = _candidate(tmp_path, monkeypatch)
    installer.check_package(root, dist=dist, profile="dev")
    assert not list(dist.glob("*.sig"))

    # Explicit Signed Verification Cannot Silently Downgrade A Candidate
    with pytest.raises(FileNotFoundError):
        installer.check_package(root, dist=dist, require_signed=True)


@pytest.mark.parametrize("profile", ["dev", "release"])
@pytest.mark.parametrize("platform", ["linux", "win32"])
def test_package_collects_and_reports_matching_profile(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str], profile: str, platform: str
) -> None:
    root, _dist = _candidate(tmp_path, monkeypatch)
    monkeypatch.setattr(installer.sys, "platform", platform)
    monkeypatch.setattr(linux, "verify", lambda *_args: None)
    bundles = tmp_path / "bundles"
    names = ["deb/candidate.deb", "rpm/candidate.rpm", "rpm/Dokime.desktop"] if platform == "linux" else ["nsis/candidate-setup.exe"]
    for name in names:
        source = bundles / name
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_bytes(profile.encode())

    for candidate in ("dev", "release"):
        plugins = installer.paths.output(root, "build", candidate) / "plugins"
        plugins.mkdir(parents=True)
        (plugins / "fixture.dokime").write_bytes(candidate.encode())

    installer.collect(root, profile=profile, signed=False, bundles=bundles)

    packaged = installer.paths.output(root, "package", profile) / "rust"
    assert (packaged / "plugins/fixture.dokime").read_bytes() == profile.encode()
    outputs = list((packaged / "installers").iterdir())
    assert len(outputs) == (2 if platform == "linux" else 1)
    assert all(path.read_bytes() == profile.encode() for path in outputs)
    other = "release" if profile == "dev" else "dev"
    assert not installer.paths.output(root, "package", other).exists()
    assert f"Packaged the {profile} profile:" in capsys.readouterr().out


@pytest.mark.parametrize("scenario", ["valid", "missing", "wrong_name", "empty_payload", "wrong_launcher", "unsupported_digest"])
def test_linux_launcher_icon(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, scenario: str) -> None:
    launcher = tmp_path / "Dokime.desktop"
    launcher.write_text("[Desktop Entry]\nName=Dokime\nExec=dokime-gui\nIcon=dokime-gui\n", encoding="utf-8")
    digest = hashlib.sha256(launcher.read_bytes()).hexdigest()
    icon = "another-app" if scenario == "wrong_name" else "dokime-gui"
    icon_digest = "" if scenario == "empty_payload" else "1" * 64
    algorithm = "1" if scenario == "unsupported_digest" else "8"
    entries = [f"/usr/share/applications/Dokime.desktop\t{digest}"]
    if scenario != "missing":
        entries.append(f"/usr/share/icons/hicolor/128x128/apps/{icon}.png\t{icon_digest}")
    output = "\n".join([algorithm, *entries]) + "\n"
    if scenario == "wrong_launcher":
        launcher.write_text("[Desktop Entry]\nIcon=another-app\n", encoding="utf-8")
    monkeypatch.setattr(linux.shutil, "which", lambda _: "/usr/bin/rpm")
    monkeypatch.setattr(linux.subprocess, "run", lambda *_args, **_kwargs: SimpleNamespace(stdout=output))

    # The Old Package Has A Launcher But No Resolvable Installed Icon
    if scenario == "valid":
        linux.verify(tmp_path / "candidate.rpm", launcher)
    else:
        with pytest.raises(ValueError, match="hicolor|packaged payload|SHA-256"):
            linux.verify(tmp_path / "candidate.rpm", launcher)


# === Private Functions === #


def _candidate(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> tuple[Path, Path]:
    for name in (
        "GITHUB_REF",
        "GITHUB_REF_TYPE",
        "GITHUB_REF_NAME",
        "CI_COMMIT_TAG",
        "DOKIME_RELEASE_CONTEXT",
        "TAURI_SIGNING_PRIVATE_KEY",
        "TAURI_SIGNING_PRIVATE_KEY_PATH",
        "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
        "EXPECTED_PACKAGE_VERSION",
    ):
        monkeypatch.delenv(name, raising=False)
    monkeypatch.setattr(installer, "check", lambda _root: None)

    root = tmp_path / "repository"
    config = root / installer.TAURI / "tauri.conf.json"
    config.parent.mkdir(parents=True)
    config.write_text(json.dumps({"version": "0.1.0"}), encoding="utf-8")
    brand = root / "organization/organization.toml"
    brand.parent.mkdir()
    brand.write_text('[app]\nslug = "dokime"\n', encoding="utf-8")

    dist = tmp_path / "installers"
    dist.mkdir()
    (dist / "dokime-0.1.0-windows-x86_64-setup.exe").write_bytes(b"candidate installer bytes")
    return root, dist


def _tag(monkeypatch: pytest.MonkeyPatch, provider: str) -> None:
    if provider == "github":
        monkeypatch.setenv("GITHUB_REF", "refs/tags/v0.1.0")
    else:
        monkeypatch.setenv("CI_COMMIT_TAG", "v0.1.0")


# === Entry Point === #
