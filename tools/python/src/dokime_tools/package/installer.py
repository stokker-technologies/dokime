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
import json
import os
import shutil
import subprocess
import sys
import tomllib
from datetime import UTC, datetime
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build.outputs import executables
from dokime_tools.test import archive
from dokime_tools.package.staging import stage_file, stage_verified
from dokime_tools.release.product import ROOT as ROOT

# Type Checking Imports


# === Public Exports === #

# === Constants === #

WEB = Path("crates/dokime-gui")
TAURI = WEB / "src-tauri"
DIST = Path("dist/tauri/windows-x86_64")
LINUX_DIST = Path("dist/tauri/linux-x86_64")
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
    if packaging["bundle"].get("externalBin") != ["binaries/dokime", "binaries/dokime-server"]:
        raise ValueError(
            "Tauri bundle.externalBin must contain the Dokime CLI and server"
        )
    if bundle.get("resources") != []:
        raise ValueError("Tauri bundle.resources must be empty; branding is compiled")
    if (
        bundle.get("windows", {}).get("nsis", {}).get("installerHooks")
        != "windows/installer-hooks.nsh"
    ):
        raise ValueError("NSIS installer hooks must install the CLI command")
    linux = json.loads(
        (directory / "tauri.linux.conf.json").read_text(encoding="utf-8")
    )
    if linux["bundle"]["targets"] != ["deb", "rpm"]:
        raise ValueError("Linux targets must be deb and rpm")
    for package in ["deb", "rpm"]:
        files = bundle.get("linux", {}).get(package, {}).get("files", {})
        if any(name.startswith("/usr/share/dokime/plugins/") for name in files):
            raise ValueError("Linux packages must not install plugin payloads")
    updater = config.get("plugins", {}).get("updater", {})
    if not updater.get("pubkey") or not updater.get("endpoints"):
        raise ValueError("updater public key and endpoints must be configured")
    print("Tauri installer configuration is valid.")


def bundle_args(platform: str, *, signed: bool) -> list[str]:
    if platform not in {"linux", "win32"}:
        raise ValueError("native installers require Windows or Linux")
    arguments = [
        "bundle",
        "--config",
        "../../branding/generated/tauri.brand.json",
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
    if profile != "release":
        raise ValueError("distribution installers are assembled only from release-profile application artifacts")
    check(root)
    signed = sys.platform == "win32" and _signing_available()
    bundles = _target_directory(root) / "release" / "bundle"
    if sys.platform == "win32":
        if (bundles / "nsis").exists():
            shutil.rmtree(bundles / "nsis")
        if (root / DIST).exists():
            shutil.rmtree(root / DIST)
    else:
        for package in ("deb", "rpm"):
            if (bundles / package).exists():
                shutil.rmtree(bundles / package)
        if (root / LINUX_DIST).exists():
            shutil.rmtree(root / LINUX_DIST)
    command = [
        "node",
        "./node_modules/@tauri-apps/cli/tauri.js",
        *bundle_args(sys.platform, signed=signed),
    ]
    subprocess.run(command, cwd=root / WEB, check=True)
    collect(root, profile=profile, signed=signed, bundles=bundles)


def stage_resources(
    root: Path = ROOT, *, profile: str = "release", target: str | None = None,
    arguments: tuple[str, ...] = (),
) -> None:
    archive.require(root, profile, archive.tree(root), arguments)
    receipt_path = archive.location(root, profile).with_name(archive.RECEIPT)
    record = json.loads(receipt_path.read_text(encoding="utf-8"))
    triple = record["target"]
    if target is not None and target != triple:
        raise ValueError("companion target must match the selected archive producer")
    for name, executable in executables(root, record, ("dokime", "dokime-server")).items():
        destination = (
            root
            / TAURI
            / "binaries"
            / (f"{name}-{triple}" + (".exe" if "windows" in triple else ""))
        )
        stage_verified(root, record, {executable: destination})
        print(f"Staged Tauri companion {name}: {destination}")


def collect(root: Path = ROOT, *, profile: str, signed: bool, bundles: Path) -> None:
    if profile != "release":
        raise ValueError("native installers are produced only by release-tag packaging")
    if sys.platform == "linux":
        version = _release_version(root)
        slug = _slug(root)
        sources = {
            f"{slug}_{version}_amd64.deb": _one(
                list((bundles / "deb").glob("*.deb")), "DEB installer"
            ),
            f"{slug}-{version}-1.x86_64.rpm": _one(
                list((bundles / "rpm").glob("*.rpm")), "RPM installer"
            ),
        }
        destination = root / LINUX_DIST
        destination.mkdir(parents=True, exist_ok=True)
        for name, source in sources.items():
            stage_file(source, destination / name)
        print(f"Built Linux packages in {bundles}")
        return
    if sys.platform != "win32":
        raise ValueError("native installers require Windows or Linux")
    nsis = bundles / "nsis"
    installer = _one(
        [p for p in nsis.rglob("*.exe") if "setup" in p.name], "NSIS installer"
    )
    version = _release_version(root)
    slug = _slug(root)
    destination = root / DIST / f"{slug}-{version}-{UPDATER_TARGET}-setup.exe"
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(installer, destination)
    if os.environ.get("DOKIME_RELEASE_CONTEXT") == "true" and not signed:
        raise ValueError(
            "final Windows releases require the configured updater signing key"
        )
    if signed:
        signatures = list(nsis.rglob(installer.name + ".sig")) or list(
            nsis.rglob("*.sig")
        )
        signature = _one(signatures, "updater signature")
        copied = destination.with_suffix(".exe.sig")
        shutil.copyfile(signature, copied)
        base = os.environ.get("TAURI_UPDATER_ARTIFACT_BASE_URL", "").strip().rstrip("/")
        if not base:
            project = PROJECT_URL
            base = f"{project}/releases/download/v{version}"
        manifest = {
            "version": version,
            "pub_date": datetime.now(UTC)
            .replace(microsecond=0)
            .isoformat()
            .replace("+00:00", "Z"),
            "platforms": {
                UPDATER_TARGET: {
                    "signature": copied.read_text(encoding="utf-8").strip(),
                    "url": f"{base}/{destination.name}",
                }
            },
        }
        (destination.parent / "latest.json").write_text(
            json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
        )
    else:
        for path in destination.parent.iterdir():
            if path.suffix == ".sig" or path.name == "latest.json":
                path.unlink()
    print(f"Built Windows installer: {destination}")


def check_package(
    root: Path = ROOT,
    *,
    dist: Path = DIST,
    expected: str | None = None,
    require_signed: bool = False,
) -> None:
    check(root)
    expected = (
        (
            expected
            or os.environ.get("EXPECTED_PACKAGE_VERSION")
            or _release_version(root)
        )
        .strip()
        .removeprefix("v")
    )
    slug = _slug(root)
    directory = root / dist
    installer = _one(
        list(directory.glob(f"{slug}-*-{UPDATER_TARGET}-setup.exe")), "Tauri installer"
    )
    if installer.name != f"{slug}-{expected}-{UPDATER_TARGET}-setup.exe":
        raise ValueError("installer version does not match the expected version")
    if (
        require_signed
        or os.environ.get("GITHUB_REF_TYPE") == "tag"
        and _signing_available()
    ):
        signature = (
            installer.with_suffix(".exe.sig").read_text(encoding="utf-8").strip()
        )
        manifest = json.loads((directory / "latest.json").read_text(encoding="utf-8"))
        platform = manifest.get("platforms", {}).get(UPDATER_TARGET, {})
        if (
            manifest.get("version") != expected
            or platform.get("signature") != signature
            or installer.name not in platform.get("url", "")
        ):
            raise ValueError(
                "updater manifest must identify the exact installer and signature"
            )
    print(f"Tauri installer artifacts are valid for {expected}.")


def acceptance(junit: Path, root: Path = ROOT, *, profile: str = "dev") -> None:
    environment = os.environ.copy()
    release = profile == "release"
    if release and sys.platform != "win32":
        raise ValueError("installed release acceptance requires Windows")
    if release:
        from dokime_tools.check.release import verify_platform

        package = root / "dist/build/windows"
        manifest = verify_platform(
            package, "windows", root, require_validation=False, check_tools=False
        )
        products = {entry["role"]: entry for entry in manifest["files"]}
        for role, label in (
            ("installer", "INSTALLER"),
            ("desktop", "SHELL"),
            ("host", "BACKEND"),
            ("client", "CLI"),
            ("plugin", "PLUGIN"),
        ):
            entry = products[role]
            environment[f"DOKIME_E2E_{label}_SHA256"] = entry["sha256"]
            if role in {"installer", "plugin"}:
                environment[f"DOKIME_E2E_{label}"] = str(
                    (package / entry["path"]).resolve()
                )
        if not environment.get("DOKIME_E2E_DRIVER"):
            driver = shutil.which("msedgedriver")
            if driver is None:
                supplied = Path("C:/SeleniumWebDrivers/EdgeDriver/msedgedriver.exe")
                if supplied.is_file():
                    driver = str(supplied)
            if driver is not None:
                environment["DOKIME_E2E_DRIVER"] = driver
    subprocess.run(
        ["node", "crates/dokime-gui/e2e/native-desktop.mjs", "--junit", str(junit), *(["--release"] if release else [])],
        cwd=root,
        env=environment,
        check=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command", choices=["check", "bundle", "resources", "verify", "acceptance"]
    )
    parser.add_argument("--junit", type=Path)
    parser.add_argument("--profile", choices=["dev", "release"], default="dev")
    parser.add_argument("--target")
    parser.add_argument("--dist-dir", type=Path, default=DIST)
    parser.add_argument("--expected-version")
    parser.add_argument("--require-signed", action="store_true")
    args = parser.parse_args()
    try:
        if args.command == "check":
            check()
        elif args.command == "bundle":
            bundle(profile=args.profile)
        elif args.command == "resources":
            stage_resources(profile=args.profile, target=args.target)
        elif args.command == "acceptance":
            if args.junit is None:
                raise ValueError(
                    "native acceptance requires the evidence runner JUnit path"
                )
            acceptance(args.junit, profile=args.profile)
        else:
            check_package(
                dist=args.dist_dir,
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
    )
    return Path(json.loads(output)["target_directory"])


def _one(paths: list[Path], label: str) -> Path:
    if len(paths) != 1:
        raise ValueError(f"expected one {label}; found {len(paths)}")
    return paths[0]


def _signing_available() -> bool:
    key = os.environ.get("TAURI_SIGNING_PRIVATE_KEY", "").strip()
    key_path = os.environ.get("TAURI_SIGNING_PRIVATE_KEY_PATH", "").strip()
    return bool(
        (key or key_path and Path(key_path).is_file())
        and os.environ.get("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "").strip()
    )


def _release_version(root: Path) -> str:
    tag = (
        os.environ.get("GITHUB_REF_NAME", "").strip().removeprefix("v")
        if os.environ.get("GITHUB_REF_TYPE") == "tag"
        else ""
    )
    return (
        tag
        or json.loads((root / TAURI / "tauri.conf.json").read_text(encoding="utf-8"))[
            "version"
        ]
    )


def _slug(root: Path) -> str:
    return tomllib.loads((root / "branding/brand.toml").read_text(encoding="utf-8"))[
        "app"
    ]["slug"]


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
