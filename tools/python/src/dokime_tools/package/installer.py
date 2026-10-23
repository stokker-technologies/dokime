# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Check And Stage Native Desktop Packages"""

# === Imports === #

# Standard Library Imports
import argparse
import base64
import binascii
import json
import os
import re
import shutil
import subprocess
import sys
import tomllib
from datetime import UTC, datetime
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools import temporary
from dokime_tools.build import paths
from dokime_tools.build.frontend import ROOT
from dokime_tools.package import linux
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports

# === Public Exports === #

# === Constants === #

WEB = Path("crates/dokime-gui")
TAURI = WEB / "src-tauri"
UPDATER_TARGET = "windows-x86_64"
PROJECT_URL = "https://github.com/stokker-technologies/dokime"

# === Types === #

# === Classes === #

# === Public Functions === #


def check(root: Path = ROOT) -> None:
    directory = root / TAURI
    for relative in [
        "tauri.conf.json",
        "tauri.bundle.conf.json",
        "icons/icon.ico",
        "windows/installer-hooks.nsh",
    ]:
        if not (directory / relative).is_file():
            raise ValueError(f"missing installer input: {relative}")
    config = json.loads((directory / "tauri.conf.json").read_text(encoding="utf-8"))
    bundle = config["bundle"]
    if "nsis" not in bundle["targets"]:
        raise ValueError("Tauri bundle targets must include nsis")
    if bundle.get("externalBin") != []:
        raise ValueError("Cargo-facing Tauri config must not require companion build outputs")
    packaging = json.loads((directory / "tauri.bundle.conf.json").read_text(encoding="utf-8"))
    icons = packaging["bundle"].get("icon", [])
    if not icons or not any(icon.endswith(".png") for icon in icons) or "icons/icon.ico" not in icons:
        raise ValueError("Tauri bundle.icon must select generated PNG and Windows icon inputs")
    for icon in icons:
        if not (directory / icon).is_file():
            raise ValueError(f"missing installer icon: {icon}")
    if packaging["bundle"].get("externalBin") != ["binaries/dokime", "binaries/dokime-server"]:
        raise ValueError("Tauri bundle.externalBin must contain the Dokime CLI and server")
    if bundle.get("resources") != []:
        raise ValueError("Tauri bundle.resources must be empty; organization is compiled")
    if bundle.get("windows", {}).get("nsis", {}).get("installerHooks") != "windows/installer-hooks.nsh":
        raise ValueError("NSIS installer hooks must install the CLI command")
    linux = json.loads((directory / "tauri.linux.conf.json").read_text(encoding="utf-8"))
    if linux["bundle"]["targets"] != ["deb", "rpm"]:
        raise ValueError("Linux targets must be deb and rpm")
    for package in ["deb", "rpm"]:
        files = bundle.get("linux", {}).get(package, {}).get("files", {})
        if any(name.startswith("/usr/share/dokime/plugins/") for name in files):
            raise ValueError("Linux packages must not install plugin payloads")
    updater = config.get("plugins", {}).get("updater", {})
    if not updater.get("pubkey"):
        raise ValueError("the updater verification key must be compiled into the application")
    if updater.get("endpoints") or config.get("plugins", {}).get("gitlab"):
        raise ValueError("update sources must come from the organization policy")
    print("Tauri installer configuration is valid.")


def bundle_args(platform: str, *, signed: bool) -> list[str]:
    if platform not in {"linux", "win32"}:
        raise ValueError("native installers require Windows or Linux")
    arguments = [
        "bundle",
        "--config",
        "../../organization/generated/tauri.organization.json",
        "--config",
        "src-tauri/tauri.bundle.conf.json",
        "--bundles",
        "deb,rpm" if platform == "linux" else "nsis",
    ]
    if platform == "linux":
        arguments += ["--config", "src-tauri/tauri.linux.conf.json"]
    if not signed:
        arguments += [
            "--config",
            json.dumps({"bundle": {"createUpdaterArtifacts": False}, "plugins": {"updater": {"pubkey": ""}}}),
            "--no-sign",
        ]
    return arguments


def bundle(root: Path = ROOT, *, profile: str = "release") -> None:
    """Bundle The Built Shell With Its Companions; Development Packages Use Tauri's Debug Mode."""
    check(root)
    release = profile == "release"
    windows_release = release and sys.platform == "win32"
    signed = windows_release and _signing_available()
    if windows_release and _release_requested() and not signed:
        raise ValueError("final Windows releases require the configured updater signing key and password")

    # Stage Matching Product Bytes
    directory = _target_directory(root) / ("release" if release else "debug")
    stage_resources(root, directory=directory, profile=profile)
    bundles = directory / "bundle"
    for package in ("nsis",) if sys.platform == "win32" else ("deb", "rpm"):
        if (bundles / package).exists():
            shutil.rmtree(bundles / package)
    command = [
        "node",
        "./node_modules/@tauri-apps/cli/tauri.js",
        *bundle_args(sys.platform, signed=signed),
        *([] if release else ["--debug"]),
    ]
    subprocess.run(command, cwd=root / WEB, check=True)
    collect(root, profile=profile, signed=signed, bundles=bundles)


def stage_resources(root: Path = ROOT, *, directory: Path, profile: str) -> None:
    """Place The Built Executables Where The Tauri Bundler Reads Them, Without Compiling."""
    built = paths.output(root, "build", profile) / "bin"
    suffix = ".exe" if sys.platform == "win32" else ""
    triple = _host_triple(root)
    for name in ("dokime", "dokime-server"):
        destination = root / TAURI / "binaries" / f"{name}-{triple}{suffix}"
        stage_file(_built(built, f"{name}{suffix}"), destination)
        print(f"Staged Tauri companion {name}: {destination}")
    shell = _built(built, f"dokime-gui{suffix}")
    destination = directory / shell.name
    if not destination.exists() or not destination.samefile(shell):
        stage_file(shell, destination)


def collect(root: Path = ROOT, *, profile: str, signed: bool, bundles: Path) -> None:
    destination = paths.output(root, "package", profile) / "rust"
    plugins = sorted((paths.output(root, "build", profile) / "plugins").glob("*.dokime"))
    if not plugins:
        raise ValueError("packages require the matching built plugin bundle; run `mise run build` first")
    for kind in ("installers", "plugins"):
        clear_output(destination / kind)
    for plugin in plugins:
        stage_file(plugin, destination / "plugins" / plugin.name)
    installers = destination / "installers"
    if sys.platform == "linux":
        version = _release_version(root)
        slug = _slug(root)
        sources = {
            f"{slug}_{version}_amd64.deb": _one(list((bundles / "deb").glob("*.deb")), "DEB installer"),
            f"{slug}-{version}-1.x86_64.rpm": _one(list((bundles / "rpm").glob("*.rpm")), "RPM installer"),
        }
        rpm = sources[f"{slug}-{version}-1.x86_64.rpm"]
        launcher = _one(list((bundles / "rpm").rglob("*.desktop")), "RPM desktop launcher")
        linux.verify(rpm, launcher)
        installers.mkdir(parents=True, exist_ok=True)
        for name, source in sources.items():
            stage_file(source, installers / name)
        print(f"Packaged the {profile} profile: Linux packages in {installers}")
        return
    if sys.platform != "win32":
        raise ValueError("native installers require Windows or Linux")
    nsis = bundles / "nsis"
    installer = _one([p for p in nsis.rglob("*.exe") if "setup" in p.name], "NSIS installer")
    version = _release_version(root)
    slug = _slug(root)
    destination = installers / f"{slug}-{version}-{UPDATER_TARGET}-setup.exe"
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(installer, destination)
    if profile == "release" and _release_requested() and not signed:
        raise ValueError("final Windows releases require the configured updater signing key")
    if signed:
        signatures = list(nsis.rglob(installer.name + ".sig")) or list(nsis.rglob("*.sig"))
        signature = _one(signatures, "updater signature")
        copied = destination.with_suffix(".exe.sig")
        shutil.copyfile(signature, copied)
        verify_signature(root, destination, copied)
        base = os.environ.get("TAURI_UPDATER_ARTIFACT_BASE_URL", "").strip().rstrip("/")
        if not base:
            project = PROJECT_URL
            base = f"{project}/releases/download/v{version}"
        manifest = {
            "version": version,
            "pub_date": datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
            "platforms": {
                UPDATER_TARGET: {
                    "signature": copied.read_text(encoding="utf-8").strip(),
                    "url": f"{base}/{destination.name}",
                }
            },
        }
        (destination.parent / "latest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    else:
        for path in destination.parent.iterdir():
            if path.suffix == ".sig" or path.name == "latest.json":
                path.unlink()
    print(f"Packaged the {profile} profile: Windows installer {destination}")


def check_package(
    root: Path = ROOT,
    *,
    dist: Path | None = None,
    profile: str = "release",
    expected: str | None = None,
    require_signed: bool = False,
) -> None:
    check(root)
    expected = (expected or os.environ.get("EXPECTED_PACKAGE_VERSION") or _release_version(root)).strip().removeprefix("v")
    slug = _slug(root)
    directory = paths.output(root, "package", profile) / "rust/installers" if dist is None else root / dist
    installer = _one(list(directory.glob(f"{slug}-*-{UPDATER_TARGET}-setup.exe")), "Tauri installer")
    if installer.name != f"{slug}-{expected}-{UPDATER_TARGET}-setup.exe":
        raise ValueError("installer version does not match the expected version")

    # Verify Published Bytes Independently Of Private Signing Credentials
    signature_file = installer.with_suffix(".exe.sig")
    final_release = profile == "release" and _release_requested()
    metadata_present = signature_file.exists() or (directory / "latest.json").exists()
    if require_signed or final_release or metadata_present:
        signature = signature_file.read_text(encoding="utf-8").strip()
        manifest = json.loads((directory / "latest.json").read_text(encoding="utf-8"))
        platform = manifest.get("platforms", {}).get(UPDATER_TARGET, {})
        identity_matches = manifest.get("version") == expected and platform.get("signature") == signature
        named_installer = platform.get("url", "").rsplit("/", 1)[-1]
        if not identity_matches or named_installer != installer.name:
            raise ValueError("updater manifest must identify the exact installer and signature")
        verify_signature(root, installer, signature_file)
    print(f"Tauri installer artifacts are valid for {expected}.")


def verify_signature(root: Path, installer: Path, signature: Path) -> None:
    """Verify Exact Installer Bytes Against The Shipped Updater Public Key."""
    config = json.loads((root / TAURI / "tauri.conf.json").read_text(encoding="utf-8"))
    encoded_key = config["plugins"]["updater"]["pubkey"].strip()
    encoded_signature = signature.read_text(encoding="utf-8").strip()
    try:
        public_key = base64.b64decode(encoded_key, validate=True)
        signed = base64.b64decode(encoded_signature, validate=True)
    except (binascii.Error, ValueError) as error:
        raise ValueError("updater public key and signature must contain valid base64") from error
    with temporary.directory() as scratch:
        key_file = scratch / "updater.pub"
        signature_file = scratch / "installer.minisig"
        key_file.write_bytes(public_key)
        signature_file.write_bytes(signed)

        # Minisign Verifies The Payload And Its Signed Comment
        payload = str(installer.resolve())
        result = subprocess.run(
            ["minisign", "-V", "-q", "-m", payload, "-p", str(key_file), "-x", str(signature_file)],
            cwd=root,
            capture_output=True,
            check=False,
        )
    if result.returncode:
        raise ValueError("installer signature does not match its bytes and the configured updater public key")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["check", "bundle", "resources", "verify"])
    parser.add_argument("--profile", choices=["dev", "release"], default="dev")
    parser.add_argument("--dist-dir", type=Path)
    parser.add_argument("--expected-version")
    parser.add_argument("--require-signed", action="store_true")
    parser.add_argument("--release-assets", action="store_true")
    args = parser.parse_args()
    try:
        if args.release_assets:
            if args.command != "verify" or args.dist_dir is not None or not os.environ.get("RELEASE_ASSET_ROOT"):
                raise ValueError("--release-assets requires verify and the native RELEASE_ASSET_ROOT")
            args.dist_dir = Path(os.environ["RELEASE_ASSET_ROOT"])
        if args.command == "check":
            check()
        elif args.command == "bundle":
            bundle(profile=args.profile)
        elif args.command == "resources":
            stage_resources(directory=_target_directory(ROOT) / ("release" if args.profile == "release" else "debug"), profile=args.profile)
        else:
            check_package(
                dist=args.dist_dir,
                profile=args.profile,
                expected=args.expected_version,
                require_signed=args.require_signed,
            )
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


# === Private Functions === #


def _target_directory(root: Path) -> Path:
    output = subprocess.check_output(
        [
            "mise",
            "exec",
            "--",
            "cargo",
            "metadata",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            str(root / TAURI / "Cargo.toml"),
        ],
        cwd=root,
        text=True,
        encoding="utf-8",
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stderr=sys.stderr,
    )
    return Path(json.loads(output)["target_directory"])


def _built(directory: Path, name: str) -> Path:
    path = directory / name
    if not path.is_file():
        raise ValueError(f"missing built executable {path}; run `mise run build` first")
    return path


def _host_triple(root: Path) -> str:
    output = subprocess.check_output(
        ["mise", "exec", "--", "rustc", "-vV"],
        cwd=root,
        text=True,
        encoding="utf-8",
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stderr=sys.stderr,
    )
    return next(line.split(": ", 1)[1] for line in output.splitlines() if line.startswith("host: "))


def _one(paths: list[Path], label: str) -> Path:
    if len(paths) != 1:
        raise ValueError(f"expected one {label}; found {len(paths)}")
    return paths[0]


def _signing_available() -> bool:
    key = os.environ.get("TAURI_SIGNING_PRIVATE_KEY", "").strip()
    key_path = os.environ.get("TAURI_SIGNING_PRIVATE_KEY_PATH", "").strip()
    return bool((key or key_path and Path(key_path).is_file()) and os.environ.get("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "").strip())


def _release_version(root: Path) -> str:
    version = json.loads((root / TAURI / "tauri.conf.json").read_text(encoding="utf-8"))["version"]
    tag = _stable_tag()
    if tag and tag.removeprefix("v") != version:
        raise ValueError("release tag version differs from the configured installer version")
    return version


def _stable_tag() -> str:
    tag = os.environ.get("CI_COMMIT_TAG", "").strip()
    if os.environ.get("GITHUB_REF", "").startswith("refs/tags/"):
        tag = os.environ["GITHUB_REF"].removeprefix("refs/tags/")
    elif os.environ.get("GITHUB_REF_TYPE") == "tag":
        tag = os.environ.get("GITHUB_REF_NAME", "").strip()
    return tag if re.fullmatch(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", tag) else ""


def _release_requested() -> bool:
    return os.environ.get("DOKIME_RELEASE_CONTEXT") == "true" or bool(_stable_tag())


def _slug(root: Path) -> str:
    return tomllib.loads((root / "organization/organization.toml").read_text(encoding="utf-8"))["app"]["slug"]


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
