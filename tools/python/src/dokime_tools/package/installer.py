# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Assemble Product Installers Through The Candidate Authoring Tool"""

# === Imports === #

# Standard Library Imports
import argparse
import base64
import binascii
import json
import os
import re
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools import temporary
from dokime_tools.build import paths
from dokime_tools.build.frontend import ROOT
from dokime_tools.ci import bootstrap
from dokime_tools.package import payload
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports

# === Public Exports === #

# === Constants === #

TAURI = Path("crates/dokime-gui/src-tauri")
UPDATER_TARGET = "windows-x86_64"
PROJECT_URL = "https://github.com/stokker-technologies/dokime"

# === Types === #

# === Classes === #

# === Public Functions === #


def check(root: Path = ROOT) -> None:
    config = json.loads((root / TAURI / "tauri.conf.json").read_text(encoding="utf-8"))
    updater = config.get("plugins", {}).get("updater", {})
    if not updater.get("pubkey"):
        raise ValueError("the updater verification key must be compiled into the application")
    if updater.get("endpoints") or config.get("plugins", {}).get("gitlab"):
        raise ValueError("update sources must come from the organization policy")


def bundle(root: Path = ROOT, *, profile: str = "release") -> None:
    """Package And Inspect The Matching Product Bytes Without Compiling."""
    check(root)
    signed = profile == "release" and sys.platform == "win32" and _signing_available()
    required = profile == "release" and sys.platform == "win32" and _release_requested()
    if required and not signed:
        raise ValueError("final Windows releases require the configured updater signing key and password")

    build = paths.output(root, "build", profile)
    source = build / "product/installer-payload"
    metadata = json.loads((source / "payload.json").read_text(encoding="utf-8"))
    version = _release_version(root)
    if metadata["version"] != version:
        raise ValueError("installer payload version differs from the configured release")
    tool = build / "bin" / ("cargo-dokime.exe" if sys.platform == "win32" else "cargo-dokime")
    if not tool.is_file():
        raise ValueError("packaging requires the matching built cargo-dokime executable")

    destination = paths.output(root, "package", profile) / "rust"
    installers = destination / "installers"
    host_plugin = build / "plugins/host.dokime"
    with temporary.directory() as scratch:
        manifest = scratch / "installer.toml"
        manifest.write_text(_manifest(root, metadata, signed), encoding="utf-8")
        command = [str(tool), "installer", "package", "--manifest", str(manifest), "--artifacts", str(source)]
        command += ["--plugin", str(host_plugin)]
        command += ["--output", str(scratch / "installers"), "--json", *(["--sign"] if signed else [])]
        result = scratch / "packages.json"
        _record(command, result, root)
        packages = json.loads(result.read_text(encoding="utf-8"))
        if not isinstance(packages, list) or not packages:
            raise ValueError("installer package returned an invalid output inventory")
        invalid_outputs = [path for path in packages if not isinstance(path, str) or not Path(path).is_file()]
        if invalid_outputs:
            raise ValueError("installer package returned missing output files")
        clear_output(installers)
        for artifact in (scratch / "installers").iterdir():
            stage_file(artifact, installers / artifact.name)
        _inspect(tool, installers, root)

    # Preserve The Portable Release Payload And Explicit Shipping Plugin
    payload.archive(source, destination)
    clear_output(destination / "plugins")
    stage_file(host_plugin, destination / "plugins" / host_plugin.name)


def check_package(
    root: Path = ROOT, *, dist: Path | None = None, profile: str = "release", expected: str | None = None, require_signed: bool = False
) -> None:
    """Verify Published Windows Update Bytes Independently Of Private Credentials."""
    check(root)
    expected = (expected or os.environ.get("EXPECTED_PACKAGE_VERSION") or _release_version(root)).strip().removeprefix("v")
    directory = paths.output(root, "package", profile) / "rust/installers" if dist is None else root / dist
    candidates = list(directory.glob(f"dokime-*-{UPDATER_TARGET}-setup.exe"))
    if len(candidates) != 1:
        raise ValueError("expected exactly one Windows installer")
    installer = candidates[0]
    if installer.name != f"dokime-{expected}-{UPDATER_TARGET}-setup.exe":
        raise ValueError("installer version does not match the expected version")

    signature_file = installer.with_suffix(".exe.sig")
    final_release = profile == "release" and _release_requested()
    metadata_present = signature_file.exists() or (directory / "latest.json").exists()
    if require_signed or final_release or metadata_present:
        _verify_update(root, directory, installer, signature_file, expected)


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
        command = ["minisign", "-V", "-q", "-m", str(installer.resolve()), "-p", str(key_file), "-x", str(signature_file)]
        result = subprocess.run(
            command, cwd=root, capture_output=True, check=False, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0), stdin=sys.stdin
        )
    if result.returncode:
        raise ValueError("installer signature does not match its bytes and the configured updater public key")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["check", "bundle", "verify"])
    parser.add_argument("--profile", choices=["dev", "release"], default="dev")
    parser.add_argument("--dist-dir", type=Path)
    parser.add_argument("--expected-version")
    parser.add_argument("--require-signed", action="store_true")
    parser.add_argument("--release-assets", action="store_true")
    args = parser.parse_args()

    try:
        _invoke(args)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


# === Private Functions === #


def _invoke(args: argparse.Namespace) -> None:
    release_root = os.environ.get("RELEASE_ASSET_ROOT")
    invalid_release = args.command != "verify" or args.dist_dir is not None or not release_root
    if args.release_assets and invalid_release:
        raise ValueError("--release-assets requires verify and the native RELEASE_ASSET_ROOT")
    if args.release_assets:
        args.dist_dir = Path(os.environ["RELEASE_ASSET_ROOT"])

    if args.command == "check":
        check()
    elif args.command == "bundle":
        bundle(profile=args.profile)
    else:
        check_package(dist=args.dist_dir, profile=args.profile, expected=args.expected_version, require_signed=args.require_signed)


def _manifest(root: Path, metadata: dict, signed: bool) -> str:
    formats = ["nsis"] if sys.platform == "win32" else ["deb", "rpm"]
    organization = str((root / "organization").resolve())
    lines = [
        "[installer]",
        f"version = {json.dumps(metadata['version'])}",
        f"organization = {json.dumps(organization)}",
        f"formats = {json.dumps(formats)}",
        "",
        "[dokime]",
        f"version = {json.dumps(metadata['version'])}",
        f"target = {json.dumps(metadata['target'])}",
    ]
    if not signed:
        return "\n".join(lines) + "\n"

    key = "private_key_env" if os.environ.get("TAURI_SIGNING_PRIVATE_KEY", "").strip() else "private_key_path_env"
    source = "TAURI_SIGNING_PRIVATE_KEY" if key == "private_key_env" else "TAURI_SIGNING_PRIVATE_KEY_PATH"
    base = os.environ.get("TAURI_UPDATER_ARTIFACT_BASE_URL", "").strip().rstrip("/")
    base = base or f"{PROJECT_URL}/releases/download/v{metadata['version']}"
    lines += [
        "",
        "[signing.updater]",
        f"{key} = {json.dumps(source)}",
        'password_env = "TAURI_SIGNING_PRIVATE_KEY_PASSWORD"',
        f"artifact_base_url = {json.dumps(base)}",
    ]
    return "\n".join(lines) + "\n"


def _inspect(tool: Path, directory: Path, root: Path) -> None:
    packages = [path for path in directory.iterdir() if path.suffix in {".exe", ".deb", ".rpm"}]
    if not packages:
        raise ValueError("installer package produced no native packages")
    for package in packages:
        command = [str(tool), "installer", "inspect", str(package), "--json"]
        result = package.with_suffix(package.suffix + ".inspection.json")
        _record(command, result, root)


def _record(command: list[str], path: Path, root: Path) -> None:
    with path.open("w", encoding="utf-8") as output:
        _run(command, root, stdout=output)


def _run(command: list[str], root: Path, *, stdout=None) -> None:
    environment = os.environ.copy()
    if sys.platform == "win32":
        directory = bootstrap.prepare_windows_installer(root)
        environment["PATH"] = str(directory.resolve()) + os.pathsep + environment.get("PATH", "")
    subprocess.run(
        command,
        env=environment,
        cwd=root,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout if stdout is None else stdout,
        stderr=sys.stderr,
    )


def _verify_update(root: Path, directory: Path, installer: Path, signature_file: Path, expected: str) -> None:
    signature = signature_file.read_text(encoding="utf-8").strip()
    manifest = json.loads((directory / "latest.json").read_text(encoding="utf-8"))
    platform = manifest.get("platforms", {}).get(UPDATER_TARGET, {})
    identity_matches = manifest.get("version") == expected and platform.get("signature") == signature
    named_installer = platform.get("url", "").rsplit("/", 1)[-1]
    if not identity_matches or named_installer != installer.name:
        raise ValueError("updater manifest must identify the exact installer and signature")
    verify_signature(root, installer, signature_file)


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


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
