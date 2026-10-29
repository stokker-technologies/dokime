# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Keep Updater Verification Independent Of Private Signing Secrets"""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import shutil
import stat
import zipfile
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.build import organization
from dokime_tools.package import installer, payload
from dokime_tools.prepare import resources

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_installer_keeps_compiled_trust_and_organization_owned_sources(tmp_path: Path) -> None:
    source = installer.ROOT / installer.TAURI
    destination = tmp_path / installer.TAURI
    destination.mkdir(parents=True)
    shutil.copyfile(source / "tauri.conf.json", destination / "tauri.conf.json")
    installer.check(tmp_path)

    path = destination / "tauri.conf.json"
    config = json.loads(path.read_text())
    config["plugins"]["updater"]["endpoints"] = ["https://example.test/obsolete.json"]
    path.write_text(json.dumps(config), encoding="utf-8")
    with pytest.raises(ValueError, match="organization policy"):
        installer.check(tmp_path)
    del config["plugins"]["updater"]["endpoints"]
    del config["plugins"]["updater"]["pubkey"]
    path.write_text(json.dumps(config), encoding="utf-8")
    with pytest.raises(ValueError, match="verification key"):
        installer.check(tmp_path)


def test_organization_generation(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    assets = tmp_path / "organization"
    shutil.copytree(organization.BRAND, assets)
    config = tmp_path / "crates/dokime-gui/src-tauri/tauri.conf.json"
    config.parent.mkdir(parents=True)
    compiled = {"identifier": "com.example.fixed", "app": {"windows": [{"create": False, "title": "Compiled Title"}]}}
    config.write_text(json.dumps(compiled), encoding="utf-8")
    generated = tmp_path / "crates/dokime-gui/src/generated/organization"
    monkeypatch.setattr(organization, "ROOT", tmp_path)
    monkeypatch.setattr(organization, "BRAND", assets)
    monkeypatch.setattr(organization, "GENERATED", generated)

    organization.generate()

    resolved = json.loads((generated / "default-organization.json").read_text())
    assert resolved["label"] == "Dokime"
    assert resolved["title"] == "Dokime"
    assert resolved["logos"]
    assert all(logo["src"].startswith("data:image/svg+xml;base64,") for logo in resolved["logos"].values())
    assert json.loads(config.read_text()) == compiled
    assert not (assets / "generated").exists()
    stylesheet = (generated / "theme.css").read_text()
    assert "data:font/woff2;base64," in stylesheet
    assert 'url("fonts/' not in stylesheet
    assert "--brand-radius-sm: 0.0625rem" in stylesheet


def test_organization_resources_include_sidebar_image(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    assets = tmp_path / "organization"
    shutil.copytree(organization.BRAND, assets)
    sidebar = assets / "installer/sidebar.png"
    sidebar.parent.mkdir()
    sidebar.write_bytes((assets / "icons/icon.png").read_bytes())
    distribution = tmp_path / "dist-brand"
    monkeypatch.setattr(resources, "BRAND", assets)
    monkeypatch.setattr(resources, "DISTRIBUTION", distribution)
    monkeypatch.setattr(resources, "ensure_report", lambda: None)

    resources.refresh()

    assert (distribution / "installer/sidebar.png").read_bytes() == sidebar.read_bytes()
    assert (distribution / "organization.toml").read_bytes() == (assets / "organization.toml").read_bytes()


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

    # Missing Secrets Fail Before Reading Or Changing Build Outputs
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

    def refuse_signature(checked_root: Path, checked_payload: Path, checked_signature: str) -> None:
        assert (checked_root, checked_payload, checked_signature) == (root, payload, "candidate signature")
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


def test_published_signature_from_manifest(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root, dist = _candidate(tmp_path, monkeypatch)
    executable = next(dist.glob("*.exe"))
    metadata = {
        "version": "0.1.0",
        "platforms": {installer.UPDATER_TARGET: {"signature": "embedded signature", "url": f"https://example.com/{executable.name}"}},
    }
    manifest = dist / "latest.json"
    manifest.write_text(json.dumps(metadata), encoding="utf-8")
    verified = []
    monkeypatch.setattr(installer, "verify_signature", lambda *args: verified.append(args))

    installer.check_package(root, dist=dist, require_signed=True)

    assert verified == [(root, executable, "embedded signature")]
    assert not executable.with_suffix(".exe.sig").exists()

    # A Sidecar Cannot Override The Manifest Signature
    executable.with_suffix(".exe.sig").write_text("different signature", encoding="utf-8")
    with pytest.raises(ValueError):
        installer.check_package(root, dist=dist, require_signed=True)
    assert len(verified) == 1


@pytest.mark.parametrize("signature", [None, "", " ", 1])
def test_published_signature_required(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, signature: object) -> None:
    root, dist = _candidate(tmp_path, monkeypatch)
    executable = next(dist.glob("*.exe"))
    metadata = {
        "version": "0.1.0",
        "platforms": {installer.UPDATER_TARGET: {"signature": signature, "url": f"https://example.com/{executable.name}"}},
    }
    (dist / "latest.json").write_text(json.dumps(metadata), encoding="utf-8")
    verified = []
    monkeypatch.setattr(installer, "verify_signature", lambda *args: verified.append(args))

    with pytest.raises(ValueError):
        installer.check_package(root, dist=dist, require_signed=True)
    assert verified == []


@pytest.mark.parametrize("native", ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"])
def test_payload_profiles(tmp_path: Path, native: str) -> None:
    expected_identity = {
        "identifier": "com.example.compiled",
        "slug": "dokime",
        "updater_public_key": "compiled-trust",
    }
    sources = {}
    for profile in ("dev", "release"):
        expected = _native_build(tmp_path, profile, native)
        source = payload.export(tmp_path, profile)
        sources[profile] = source
        metadata = json.loads((source / "payload.json").read_text())
        assert source == installer.paths.output(tmp_path, "build", profile) / "product/installer-payload"
        assert metadata == {
            "schema": 1,
            "version": "1.2.3",
            "target": native,
            "identity": expected_identity,
            "files": {name: hashlib.sha256(data).hexdigest() for name, data in expected.items()},
        }
        archive = payload.archive(source, installer.paths.output(tmp_path, "package", profile) / "rust")
        assert archive.name == f"dokime-payload-1.2.3-{native}.zip"
        with zipfile.ZipFile(archive) as bundle:
            assert set(bundle.namelist()) == {"payload.json", *expected}
            assert json.loads(bundle.read("payload.json")) == metadata
            for name, data in expected.items():
                assert bundle.read(name) == data
                mode = bundle.getinfo(name).external_attr >> 16
                assert stat.S_ISREG(mode)
                assert stat.S_IMODE(mode) == (0o755 if name.startswith("bin/") else 0o644)
    assert sources["dev"] != sources["release"]
    suffix = ".exe" if "windows" in native else ""
    for profile, source in sources.items():
        assert (source / f"bin/dokime{suffix}").read_bytes() == f"{profile} cli".encode()
        assert (source / f"bin/dokime-gui{suffix}").read_bytes() == f"{profile} shell".encode()
        assert (
            source / "organization/organization.toml"
        ).read_bytes() == f'[app]\nlabel = "Stokker Technologies {profile.title()}"\n'.encode()
        assert (source / "organization/theme/light.css").read_bytes() == f"/* {profile} default */".encode()


@pytest.mark.parametrize("change", ["modified", "escape"])
def test_payload_mutation_refused(tmp_path: Path, change: str) -> None:
    _native_build(tmp_path, "dev", "x86_64-unknown-linux-gnu")
    source = payload.export(tmp_path, "dev")
    output = tmp_path / "distribution"
    archive = payload.archive(source, output)
    original = archive.read_bytes()
    if change == "modified":
        (source / "bin/dokime").write_bytes(b"changed after compilation")
    else:
        manifest = json.loads((source / "payload.json").read_text())
        manifest["files"]["../outside"] = "0" * 64
        (source / "payload.json").write_text(json.dumps(manifest), encoding="utf-8")

    with pytest.raises(ValueError, match="changed after Build|unsafe installer payload path"):
        payload.archive(source, output)
    assert archive.read_bytes() == original


def test_payload_input_retained(tmp_path: Path) -> None:
    _native_build(tmp_path, "dev", "x86_64-unknown-linux-gnu")
    source = payload.export(tmp_path, "dev")
    original = {path.relative_to(source): path.read_bytes() for path in source.rglob("*") if path.is_file()}
    binary = installer.paths.output(tmp_path, "build", "dev") / "bin/dokime"
    binary.unlink()

    with pytest.raises(ValueError, match="regular file"):
        payload.export(tmp_path, "dev")
    assert {path.relative_to(source): path.read_bytes() for path in source.rglob("*") if path.is_file()} == original


# === Private Functions === #


def test_installer_explicitly_selects_shipping_host(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    _native_build(tmp_path, "dev", "x86_64-pc-windows-msvc")
    monkeypatch.setattr(installer.sys, "platform", "win32")
    monkeypatch.setattr(installer, "_stable_tag", lambda: "")
    build = installer.paths.output(tmp_path, "build", "dev")
    (build / "bin/cargo-dokime.exe").write_bytes(b"tool")
    host = build / "plugins/host.dokime"
    host.parent.mkdir()
    host.write_bytes(b"host plugin")
    payload.export(tmp_path, "dev")
    calls = []

    def record(command: list[str], output: Path, _root: Path) -> None:
        calls.append(command)
        if "package" in command:
            directory = Path(command[command.index("--output") + 1])
            directory.mkdir()
            package = directory / "dokime-setup.exe"
            package.write_bytes(b"installer")
            paths = [str(package)]
            output.write_text(json.dumps(paths), encoding="utf-8")
        else:
            output.write_text("{}", encoding="utf-8")

    monkeypatch.setattr(installer, "_record", record)
    installer.bundle(tmp_path, profile="dev")
    package = calls[0]
    assert package[package.index("--plugin") + 1] == str(host)
    assert package.count("--plugin") == 1
    destination = installer.paths.output(tmp_path, "package", "dev") / "rust"
    assert (destination / "plugins/host.dokime").read_bytes() == host.read_bytes()
    exported = destination / "plugins"
    assert list(exported.iterdir()) == [exported / "host.dokime"]


def _native_build(root: Path, profile: str, native: str) -> dict[str, bytes]:
    build = installer.paths.output(root, "build", profile)
    (build / "product").mkdir(parents=True, exist_ok=True)
    (build / "product/target.txt").write_text(native, encoding="utf-8")
    config = root / installer.TAURI / "tauri.conf.json"
    config.parent.mkdir(parents=True, exist_ok=True)
    config.write_text(
        json.dumps(
            {
                "version": "1.2.3",
                "identifier": "com.example.compiled",
                "plugins": {"updater": {"pubkey": "compiled-trust"}},
            }
        ),
        encoding="utf-8",
    )
    suffix = ".exe" if "windows" in native else ""
    expected = {
        "LICENSE": b"fixture license",
        "organization/organization.toml": f'[app]\nlabel = "Stokker Technologies {profile.title()}"\n'.encode(),
        "organization/theme/light.css": f"/* {profile} default */".encode(),
        f"bin/dokime{suffix}": f"{profile} cli".encode(),
        f"bin/dokime-gui{suffix}": f"{profile} shell".encode(),
    }
    for name, data in expected.items():
        if name == "LICENSE" or name.startswith("organization/"):
            source = root / name
        else:
            source = build / name
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_bytes(data)
    return expected


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
    brand.write_text('[app]\nlabel = "Dokime"\n', encoding="utf-8")

    dist = tmp_path / "installers"
    dist.mkdir()
    (dist / "dokime-0.1.0-x86_64-pc-windows-msvc-setup.exe").write_bytes(b"candidate installer bytes")
    return root, dist


def _tag(monkeypatch: pytest.MonkeyPatch, provider: str) -> None:
    if provider == "github":
        monkeypatch.setenv("GITHUB_REF", "refs/tags/v0.1.0")
    else:
        monkeypatch.setenv("CI_COMMIT_TAG", "v0.1.0")


# === Entry Point === #
