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
import json
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.package import installer

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


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
    brand = root / "branding/brand.toml"
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
