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
from dokime_tools.build import paths
from dokime_tools.build.frontend import ROOT
from dokime_tools.package.staging import stage_file

# Type Checking Imports


# === Public Exports === #

# === Constants === #

WEB = Path("crates/dokime-gui")
TAURI = WEB / "src-tauri"
DIST = Path("target/package/release/windows-x86_64/rust/installers")
LINUX_DIST = Path("target/package/release/linux-x86_64/rust/installers")
PLATFORM = "windows" if sys.platform == "win32" else "linux"
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
        raise ValueError("Tauri bundle.externalBin must contain the Dokime CLI and server")
    if bundle.get("resources") != []:
        raise ValueError("Tauri bundle.resources must be empty; branding is compiled")
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
    """Bundle The Built Shell With Its Companions; Development Packages Use Tauri's Debug Mode."""
    check(root)
    release = profile == "release"
    signed = release and sys.platform == "win32" and _signing_available()
    directory = _target_directory(root) / ("release" if release else "debug")
    stage_resources(root, directory=directory, profile=profile)
    bundles = directory / "bundle"
    for package in ("nsis",) if sys.platform == "win32" else ("deb", "rpm"):
        if (bundles / package).exists():
            shutil.rmtree(bundles / package)
    if release:
        dist = DIST if sys.platform == "win32" else LINUX_DIST
        if (root / dist).exists():
            shutil.rmtree(root / dist)
    command = [
        "node",
        "./node_modules/@tauri-apps/cli/tauri.js",
        *bundle_args(sys.platform, signed=signed),
        *([] if release else ["--debug"]),
    ]
    subprocess.run(command, cwd=root / WEB, check=True)
    if release:
        collect(root, signed=signed, bundles=bundles)
        plugins = paths.output(root, "build", profile) / "plugins"
        for plugin in plugins.glob("*.dokime"):
            stage_file(plugin, paths.output(root, "package", profile) / "rust/plugins" / plugin.name)
    else:
        collect_development(root, bundles=bundles, profile=profile)


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


def collect_development(root: Path = ROOT, *, bundles: Path, profile: str) -> None:
    """Gather The Unsigned Development Installers And The Plugin Bundle."""
    destination = paths.output(root, "package", profile) / "rust"
    for kind in ("installers", "plugins"):
        if (destination / kind).exists():
            shutil.rmtree(destination / kind)
    patterns = ["nsis/*-setup.exe"] if sys.platform == "win32" else ["deb/*.deb", "rpm/*.rpm"]
    installers = [path for pattern in patterns for path in bundles.glob(pattern)]
    if len(installers) != len(patterns):
        raise ValueError(f"expected {len(patterns)} development installers under {bundles}; found {len(installers)}")
    plugins = sorted((paths.output(root, "build", profile) / "plugins").glob("*.dokime"))
    if not plugins:
        raise ValueError("development packages require the built plugin bundle; run `mise run build` first")
    version = _release_version(root)
    slug = _slug(root)
    for source in installers:
        name = (
            f"{slug}-{version}-{UPDATER_TARGET}-setup.exe"
            if sys.platform == "win32"
            else (f"{slug}_{version}_amd64.deb" if source.suffix == ".deb" else f"{slug}-{version}-1.x86_64.rpm")
        )
        stage_file(source, destination / "installers" / name)
        print(f"Packaged {(destination / 'installers' / name).relative_to(root).as_posix()}")
    for source in plugins:
        stage_file(source, destination / "plugins" / source.name)
        print(f"Packaged {(destination / 'plugins' / source.name).relative_to(root).as_posix()}")


def package_wheel(root: Path = ROOT, *, profile: str) -> None:
    """Gather The Built Client Wheel Beside The Other Packages Of This Platform."""
    wheels = sorted((paths.output(root, "build", profile) / "python").glob("dokime-*"))
    if not wheels:
        raise ValueError("packaging requires the built client wheel; run `mise run build` first")
    destination = paths.output(root, "package", profile) / "python"
    if destination.exists():
        shutil.rmtree(destination)
    for wheel in wheels:
        stage_file(wheel, destination / wheel.name)
        print(f"Packaged {(destination / wheel.name).relative_to(root).as_posix()}")


def collect(root: Path = ROOT, *, signed: bool, bundles: Path) -> None:
    if sys.platform == "linux":
        version = _release_version(root)
        slug = _slug(root)
        sources = {
            f"{slug}_{version}_amd64.deb": _one(list((bundles / "deb").glob("*.deb")), "DEB installer"),
            f"{slug}-{version}-1.x86_64.rpm": _one(list((bundles / "rpm").glob("*.rpm")), "RPM installer"),
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
    installer = _one([p for p in nsis.rglob("*.exe") if "setup" in p.name], "NSIS installer")
    version = _release_version(root)
    slug = _slug(root)
    destination = root / DIST / f"{slug}-{version}-{UPDATER_TARGET}-setup.exe"
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(installer, destination)
    if os.environ.get("DOKIME_RELEASE_CONTEXT") == "true" and not signed:
        raise ValueError("final Windows releases require the configured updater signing key")
    if signed:
        signatures = list(nsis.rglob(installer.name + ".sig")) or list(nsis.rglob("*.sig"))
        signature = _one(signatures, "updater signature")
        copied = destination.with_suffix(".exe.sig")
        shutil.copyfile(signature, copied)
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
    print(f"Built Windows installer: {destination}")


def check_package(
    root: Path = ROOT,
    *,
    dist: Path = DIST,
    expected: str | None = None,
    require_signed: bool = False,
) -> None:
    check(root)
    expected = (expected or os.environ.get("EXPECTED_PACKAGE_VERSION") or _release_version(root)).strip().removeprefix("v")
    slug = _slug(root)
    staged = os.environ.get("RELEASE_ASSET_ROOT")
    directory = Path(staged) if staged else root / dist
    installer = _one(list(directory.glob(f"{slug}-*-{UPDATER_TARGET}-setup.exe")), "Tauri installer")
    if installer.name != f"{slug}-{expected}-{UPDATER_TARGET}-setup.exe":
        raise ValueError("installer version does not match the expected version")
    if require_signed or os.environ.get("GITHUB_REF_TYPE") == "tag" and _signing_available():
        signature = installer.with_suffix(".exe.sig").read_text(encoding="utf-8").strip()
        manifest = json.loads((directory / "latest.json").read_text(encoding="utf-8"))
        platform = manifest.get("platforms", {}).get(UPDATER_TARGET, {})
        if manifest.get("version") != expected or platform.get("signature") != signature or installer.name not in platform.get("url", ""):
            raise ValueError("updater manifest must identify the exact installer and signature")
    print(f"Tauri installer artifacts are valid for {expected}.")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["check", "bundle", "resources", "verify", "wheel"])
    parser.add_argument("--profile", choices=["dev", "release"], default="dev")
    parser.add_argument("--dist-dir", type=Path, default=DIST)
    parser.add_argument("--expected-version")
    parser.add_argument("--require-signed", action="store_true")
    args = parser.parse_args()
    try:
        if args.command == "check":
            check()
        elif args.command == "bundle":
            bundle(profile=args.profile)
        elif args.command == "wheel":
            package_wheel(profile=args.profile)
        elif args.command == "resources":
            stage_resources(directory=_target_directory(ROOT) / ("release" if args.profile == "release" else "debug"), profile=args.profile)
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


def _built(directory: Path, name: str) -> Path:
    path = directory / name
    if not path.is_file():
        raise ValueError(f"missing built executable {path}; run `mise run build` first")
    return path


def _host_triple(root: Path) -> str:
    output = subprocess.check_output(["mise", "exec", "--", "rustc", "-vV"], cwd=root, text=True, encoding="utf-8")
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
    tag = os.environ.get("GITHUB_REF_NAME", "").strip().removeprefix("v") if os.environ.get("GITHUB_REF_TYPE") == "tag" else ""
    return tag or json.loads((root / TAURI / "tauri.conf.json").read_text(encoding="utf-8"))["version"]


def _slug(root: Path) -> str:
    return tomllib.loads((root / "branding/brand.toml").read_text(encoding="utf-8"))["app"]["slug"]


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
