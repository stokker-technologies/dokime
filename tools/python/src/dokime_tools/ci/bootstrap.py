# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare Native CI Platform Prerequisites"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
import urllib.parse
import urllib.request
import zipfile
from collections.abc import Sequence
from pathlib import Path
from typing import Any

# Debian Build And Installer Prerequisites
GUI_MARKER_PACKAGE = "libwebkit2gtk-4.1-dev"
GUI_PACKAGES = (
    "git",
    "xz-utils",
    "unzip",
    "build-essential",
    "pkg-config",
    "libssl-dev",
    "patchelf",
    "rpm",
    "libarchive-tools",
    "file",
    GUI_MARKER_PACKAGE,
    "libgtk-3-dev",
    "libayatana-appindicator3-1",
    "libsoup-3.0-dev",
    "libjavascriptcoregtk-4.1-dev",
    "librsvg2-dev",
    # Native Media Capture (dokime-app `media-native`): ALSA for audio input,
    # V4L2 kernel headers for cameras, and libclang for the bindings V4L2
    # generates at build time. Screenshots use the desktop portal over D-Bus.
    "libasound2-dev",
    "linux-libc-dev",
    "libclang-dev",
)
HARMOS_REPOSITORY = "https://github.com/stokker-technologies/harmos.git"
GUI_RUNTIME_PACKAGES = (
    "libwebkit2gtk-4.1-0",
    "libgtk-3-0t64",
    "libayatana-appindicator3-1",
    "libsoup-3.0-0",
    "libjavascriptcoregtk-4.1-0",
    "librsvg2-2",
    "libasound2t64",
    "libssl3t64",
)
DESKTOP_PACKAGES = (
    "webkit2gtk-driver",
    "xvfb",
    "xdotool",
    "at-spi2-core",
    "python3-gi",
    "gir1.2-atspi-2.0",
    "desktop-file-utils",
    "xdg-utils",
    "dbus",
    "gnome-keyring",
    "libnss3",
    "libnspr4",
    "libgbm1",
    "libxss1",
    "libx11-xcb1",
    "libxshmfence1",
    "fonts-liberation",
)
# Google's Stable Chrome For Testing Release, Used Only When The Hosted Browser Is Absent.
CHROME_VERSION = "156.0.8078.47"
SEVEN_ZIP_URL = "https://github.com/ip7z/7zip/releases/download/26.05/7z2605-x64.exe"
SEVEN_ZIP_SHA256 = "fd99e86f57d8f429ef5ed045bf98315af962e2309b0f1ba87a147a44c71647d0"
HARMOS_PACKAGES = ("harmos", "harmos-domain", "harmos-packaging", "harmos-sdk")
CRATES_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parse_arguments(argv)
    if arguments.harmos_key:
        print(_source_key(harmos_pin(Path.cwd())))
        return 0
    if arguments.harmos_tools:
        prepare_harmos(
            Path.cwd(),
            allow_install=os.environ.get("DOKIME_CI_ROLE") in {None, "contracts"},
            cargo_config=arguments.cargo_config,
        )
        return 0
    if arguments.gui_packages or arguments.gui_runtime:
        install_gui_packages(runtime=arguments.gui_runtime, desktop=arguments.desktop)
    if arguments.desktop:
        if sys.platform == "win32":
            prepare_windows_desktop(Path.cwd())
        elif sys.platform != "linux":
            raise ValueError("native desktop validation requires Windows or Linux")
        else:
            prepare_linux_browser(Path.cwd())
        if sys.platform == "linux" and not os.environ.get("DISPLAY"):
            github_environment = os.environ.get("GITHUB_ENV")
            if not github_environment:
                raise ValueError("the desktop job needs DISPLAY or GitHub's environment file")
            subprocess.Popen(
                ["Xvfb", ":99", "-screen", "0", "1920x1080x24"],
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )
            with Path(github_environment).open("a", encoding="utf-8") as written:
                written.write("DISPLAY=:99\n")
    _step("bootstrap complete")
    return 0


def parse_arguments(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="bootstrap.py",
        description="Prepare explicitly selected native prerequisites or the pinned Harmos tool.",
    )
    gui = parser.add_mutually_exclusive_group()
    gui.add_argument(
        "--gui-packages",
        action="store_true",
        help="install the Debian GUI development packages (Linux jobs only).",
    )
    gui.add_argument(
        "--gui-runtime",
        action="store_true",
        help="install only Ubuntu GUI runtime libraries for prebuilt artifacts.",
    )
    parser.add_argument("--harmos-key", action="store_true")
    parser.add_argument("--harmos-tools", action="store_true")
    parser.add_argument("--cargo-config", type=Path)
    parser.add_argument("--desktop", action="store_true")
    return parser.parse_args(argv)


def harmos_pin(root: Path) -> dict[str, str]:
    """Resolve Harmos To An Exact Public Git Or Locked Registry Source."""
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    dependencies = manifest["workspace"]["dependencies"]
    if any(name in dependencies for name in ("harmos-contract", "harmos-signal")):
        raise ValueError("Dokime depends on the harmos crate alone")
    dependency = dependencies.get("harmos")
    if dependency is None:
        raise ValueError("Harmos tools require a declared harmos dependency")
    if isinstance(dependency, str):
        dependency = {"version": dependency}
    if not isinstance(dependency, dict):
        raise TypeError("Harmos tools require a string or table dependency")
    url, revision, branch = (
        dependency.get("git"),
        dependency.get("rev"),
        dependency.get("branch"),
    )

    if url is None:
        return _registry_pin(root, dependencies, dependency.get("version"))
    if url != HARMOS_REPOSITORY or any(key in dependency for key in ("path", "registry", "tag")):
        raise ValueError("Harmos tools require the public Harmos Git source")
    expected = [dependency.get(key) for key in ("git", "rev", "branch")]
    for name in HARMOS_PACKAGES[1:]:
        if name not in dependencies:
            continue
        declared = dependencies[name]
        if isinstance(declared, str):
            declared = {"version": declared}
        if not isinstance(declared, dict):
            raise TypeError(f"{name} must use a dependency table")
        actual = [declared.get(key) for key in ("git", "rev", "branch")]
        unsupported = any(key in declared for key in ("path", "registry", "tag"))
        if actual != expected or unsupported:
            raise ValueError(f"{name} must use the same public Harmos Git source")

    if branch is None and isinstance(revision, str) and _is_revision(revision):
        return {"git": HARMOS_REPOSITORY, "rev": revision, "package": "cargo-harmos"}
    if revision is None and isinstance(branch, str) and branch:
        # Cargo Percent-Encodes The Branch In Its Lockfile Source
        locked = _locked_revision(root, f"git+{url}?branch={urllib.parse.quote(branch, safe='')}#")
        return {"git": url, "branch": branch, "rev": locked, "package": "cargo-harmos"}
    raise ValueError("Harmos tools require the exact declared public Git revision or a branch")


def prepare_harmos(
    root: Path,
    *,
    allow_install: bool,
    cargo_config: Path | None = None,
) -> Path:
    if cargo_config is None:
        cargo_config = _checkout_harmos_config(root)
    local_source = None

    if cargo_config is not None:
        if os.environ.get("CI") or os.environ.get("GITHUB_ACTIONS") or os.environ.get("DOKIME_CI_ROLE"):
            raise ValueError("CI refuses local Harmos tool overrides")
        config = cargo_config.resolve(strict=True)
        patches = tomllib.loads(config.read_text(encoding="utf-8"))["patch"]
        sources = [source for source in (HARMOS_REPOSITORY, "crates-io") if "harmos" in patches.get(source, {})]
        if len(sources) != 1:
            raise ValueError("local Harmos tools require one explicit public Git or crates.io patch")
        manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
        dependency = manifest["workspace"]["dependencies"].get("harmos")
        if isinstance(dependency, str):
            dependency = {"version": dependency}
        if not isinstance(dependency, dict):
            raise TypeError("local Harmos tools require a declared harmos dependency")
        namespace = HARMOS_REPOSITORY if dependency.get("git") == HARMOS_REPOSITORY else "crates-io"
        unsupported = any(key in dependency for key in ("path", "registry", "tag"))
        foreign_git = "git" in dependency and dependency["git"] != HARMOS_REPOSITORY
        if unsupported or foreign_git or sources[0] != namespace:
            raise ValueError("local Harmos patch must match the declared public dependency source")
        patch = patches[sources[0]]["harmos"]
        crate = Path(patch["path"])
        if not crate.is_absolute():
            raise ValueError("local Harmos tool configuration requires an absolute patch path")
        crate = crate.resolve(strict=True)
        package = tomllib.loads((crate / "Cargo.toml").read_text(encoding="utf-8"))["package"]
        if package["name"] != "harmos":
            raise ValueError("local Harmos patch does not identify the harmos package")
        local_source = crate.parent.parent
        tool = local_source / "crates/cargo-harmos"
        if tomllib.loads((tool / "Cargo.toml").read_text(encoding="utf-8"))["package"]["name"] != "cargo-harmos":
            raise ValueError("local Harmos checkout has no canonical tool package")
        source_identity = {
            "local_source": str(local_source),
            "source": _override_source(local_source),
            "cargo_config_sha256": hashlib.sha256(config.read_bytes()).hexdigest(),
            "package": "cargo-harmos",
        }
    else:
        source_identity = harmos_pin(root)
    identity: dict[str, Any] = {
        **source_identity,
        "platform": platform.system(),
        "architecture": platform.machine(),
    }
    if "registry" in source_identity:
        targets = {"Windows": "x86_64-pc-windows-msvc", "Linux": "x86_64-unknown-linux-gnu"}
        if identity["architecture"].lower() not in {"amd64", "x86_64"} or identity["platform"] not in targets:
            raise ValueError("published Harmos tools require native Windows or Linux x86_64")
        suffix = ".exe" if identity["platform"] == "Windows" else ""
        identity["asset"] = f"cargo-harmos-{identity['version']}-{targets[identity['platform']]}{suffix}"
        identity["release"] = f"https://github.com/stokker-technologies/harmos/releases/download/v{identity['version']}"
    else:
        identity.update(
            {
                "profile": "dev",
                "rustc": subprocess.check_output(
                    ["mise", "exec", "--", "rustc", "-vV"],
                    text=True,
                    encoding="utf-8",
                    creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                    stdin=sys.stdin,
                    stderr=sys.stderr,
                ).strip(),
                "environment": {
                    name: os.environ.get(name)
                    for name in sorted(
                        {
                            "RUSTC",
                            "RUSTC_WRAPPER",
                            "RUSTC_WORKSPACE_WRAPPER",
                            "RUSTFLAGS",
                            "CARGO_ENCODED_RUSTFLAGS",
                            "CARGO_BUILD_TARGET",
                            "CARGO_BUILD_RUSTFLAGS",
                        }
                        | {name for name in os.environ if name.startswith("CARGO_PROFILE_")}
                    )
                },
            }
        )
    destination = root / ".cache/harmos-tool"
    binary = destination / "bin" / ("cargo-harmos.exe" if os.name == "nt" else "cargo-harmos")
    receipt = destination / "receipt.json"
    available = False
    if binary.is_file() and not binary.is_symlink() and receipt.is_file() and not receipt.is_symlink():
        try:
            value = json.loads(receipt.read_text(encoding="utf-8"))
            available = value.get("identity") == identity and value.get("sha256") == hashlib.sha256(binary.read_bytes()).hexdigest()
            if "registry" in identity:
                available = available and value.get("version") == f"cargo harmos {identity['version']}"
                available = available and value.get("published_sha256") == value.get("sha256")
        except (OSError, ValueError):
            available = False
    if not available:
        if not allow_install:
            raise ValueError("exact prepared Harmos tool artifact is missing or incompatible; consumers never rebuild it")
        destination.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="harmos-tool-") as temporary:
            staged = Path(temporary)
            published_sha256 = None
            if "registry" in identity:
                published_sha256 = _install_harmos_release(staged, identity, binary.name)
            else:
                from dokime_tools.build import storage

                source_arguments = (
                    ["--path", str(local_source / "crates/cargo-harmos")]
                    if local_source is not None
                    else ["--git", identity["git"], "--rev", identity["rev"]]
                )
                storage.preflight(root, target=root / "target/tooling/harmos")
                _run(
                    [
                        "mise",
                        "exec",
                        "--",
                        "cargo",
                        "install",
                        "--debug",
                        *source_arguments,
                        "--locked",
                        "--root",
                        str(staged),
                        "--target-dir",
                        str(root / "target/tooling/harmos"),
                        identity["package"],
                    ]
                )
            if local_source is not None and _override_source(local_source) != identity["source"]:
                raise ValueError("Harmos source changed while preparing the local tool")
            executable = staged / "bin" / binary.name
            if executable.is_symlink() or not executable.is_file():
                raise ValueError("published Harmos package omitted its required CLI")
            version = subprocess.check_output(
                [str(executable), "--version"],
                text=True,
                encoding="utf-8",
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                stdin=sys.stdin,
                stderr=sys.stderr,
            ).strip()
            if "registry" in identity and version != f"cargo harmos {identity['version']}":
                raise ValueError("published Harmos executable reported an unexpected version")
            value = {
                "identity": identity,
                "sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
                "version": version,
            }
            if published_sha256 is not None:
                value["published_sha256"] = published_sha256
            (staged / "receipt.json").write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
            for path in staged.iterdir():
                if path.name not in {"bin", "receipt.json"}:
                    path.unlink()
            if destination.exists():
                shutil.rmtree(destination)
            shutil.copytree(staged, destination)
    if os.name != "nt":
        binary.chmod(0o755)
    github_path = os.environ.get("GITHUB_PATH")
    if github_path:
        with Path(github_path).open("a", encoding="utf-8") as output:
            output.write(str(binary.parent) + "\n")
    selected = _source_key(source_identity)
    _step(f"Harmos CLI uses {selected} with verified installed bytes")
    return binary


def install_gui_packages(*, runtime: bool = False, desktop: bool = False) -> None:
    """Install the declared GUI runtime or development packages when missing.

    A CI host that is missing them and cannot install them is broken, and the
    linker error it would eventually produce says nothing useful, so only a
    host with no dpkg at all -- a developer machine, never the CI image --
    gets to skip quietly.
    """
    if sys.platform != "linux" or shutil.which("dpkg") is None:
        _step("skipping GUI packages: no dpkg here, so this is not a Debian host")
        return
    selected = GUI_RUNTIME_PACKAGES if runtime else GUI_PACKAGES
    packages = (*selected, *(DESKTOP_PACKAGES if desktop else ()))
    if all(_package_installed(package) for package in packages):
        _step("skipping GUI packages: all prerequisites are installed")
        return

    prefix = []
    if os.geteuid() != 0 and shutil.which("sudo"):
        prefix = ["sudo", "-n"]
    elif os.geteuid() != 0:
        raise SystemExit("GUI prerequisites are missing and apt-get needs root; install the selected packages or run this as root.")
    _run([*prefix, "apt-get", "update"])
    _run([*prefix, "apt-get", "install", "-y", "--no-install-recommends", *packages])


def prepare_linux_browser(root: Path) -> Path:
    """Reuse An Installed Browser Or Install The Pinned Chrome For Testing Build."""
    selected = os.environ.get("DOKIME_E2E_CHROME")
    browser = shutil.which(selected or "google-chrome")
    if selected and browser is None:
        raise ValueError(f"the selected report browser is unavailable: {selected}")
    if browser is None:
        if platform.machine().lower() not in {"x86_64", "amd64"}:
            raise ValueError("the declared Linux desktop browser requires x86_64")
        destination = root / "target/tooling/desktop"
        executable = destination / "chrome-linux64/chrome"
        if not executable.is_file() or _driver_version(executable) != CHROME_VERSION:
            url = f"https://storage.googleapis.com/chrome-for-testing-public/{CHROME_VERSION}/linux64/chrome-linux64.zip"
            with tempfile.TemporaryDirectory(prefix="dokime-chrome-") as temporary:
                archive = Path(temporary) / "chrome.zip"
                urllib.request.urlretrieve(url, archive)
                destination.mkdir(parents=True, exist_ok=True)
                with zipfile.ZipFile(archive) as downloaded:
                    for member in downloaded.infolist():
                        path = (destination / member.filename).resolve()
                        if not path.is_relative_to(destination.resolve()):
                            raise ValueError("the browser archive contains an escaping path")
                        downloaded.extract(member, destination)
                        if not member.is_dir():
                            path.chmod((member.external_attr >> 16) & 0o777 or 0o644)
        browser = str(executable.resolve())
    version = _driver_version(Path(browser))
    if not version:
        raise ValueError(f"the report browser did not report its version: {browser}")
    os.environ["DOKIME_E2E_CHROME"] = browser
    if github_environment := os.environ.get("GITHUB_ENV"):
        with Path(github_environment).open("a", encoding="utf-8") as written:
            written.write(f"DOKIME_E2E_CHROME={browser}\n")
    _step(f"Report browser uses {browser} ({version})")
    return Path(browser)


def prepare_windows_installer(root: Path) -> Path:
    """Provision Full 7-Zip Without Running Its Installer."""
    if sys.platform != "win32" or platform.machine().lower() not in {"amd64", "x86_64"}:
        raise ValueError("Windows installer inspection requires native x86_64 Windows")
    destination = root / "target/tooling/installer"
    names = ("7z.exe", "7z.dll")
    receipt = destination / "7zip.json"
    available = False
    if receipt.is_file() and not receipt.is_symlink():
        try:
            cached = json.loads(receipt.read_text(encoding="utf-8"))
            files = {name: hashlib.sha256((destination / name).read_bytes()).hexdigest() for name in names}
            regular = all((destination / name).is_file() and not (destination / name).is_symlink() for name in names)
            available = regular and cached == {"archive_sha256": SEVEN_ZIP_SHA256, "files": files}
        except (OSError, ValueError):
            pass
    if available:
        return destination

    extractor = shutil.which("7za")
    if extractor is None:
        raise ValueError("Windows installer inspection bootstrap requires the mise-pinned 7za tool")
    with tempfile.TemporaryDirectory(prefix="dokime-7zip-") as temporary:
        staged = Path(temporary)
        archive = staged / "7zip.exe"
        urllib.request.urlretrieve(SEVEN_ZIP_URL, archive)
        if hashlib.sha256(archive.read_bytes()).hexdigest() != SEVEN_ZIP_SHA256:
            raise ValueError("pinned 7-Zip archive SHA-256 mismatch")
        extracted = staged / "extracted"
        _run([extractor, "x", str(archive), *names, "-spd", f"-o{extracted}", "-y"])
        if any(not (extracted / name).is_file() or (extracted / name).is_symlink() for name in names):
            raise ValueError("pinned 7-Zip archive is missing its inspection tools")
        files = {name: hashlib.sha256((extracted / name).read_bytes()).hexdigest() for name in names}
        destination.mkdir(parents=True, exist_ok=True)
        for name in names:
            shutil.copyfile(extracted / name, destination / name)
        receipt.write_text(json.dumps({"archive_sha256": SEVEN_ZIP_SHA256, "files": files}, indent=2) + "\n", encoding="utf-8")
    return destination


def prepare_windows_desktop(root: Path) -> Path:
    """Resolve WebView2 And Install Its Exact Microsoft WebDriver."""
    version = _webview_version()
    if version is None:
        with tempfile.TemporaryDirectory(prefix="dokime-webview-") as temporary:
            installer = Path(temporary) / "MicrosoftEdgeWebview2Setup.exe"
            urllib.request.urlretrieve("https://go.microsoft.com/fwlink/p/?LinkId=2124703", installer)
            _run([str(installer), "/silent", "/install"])
        version = _webview_version()
    if version is None or len(version.split(".")) != 4 or not all(part.isdigit() for part in version.split(".")):
        raise ValueError("WebView2 did not report a valid installed runtime version")
    destination = root / "target/tooling/desktop"
    driver = destination / "msedgedriver.exe"
    if not driver.is_file() or _driver_version(driver) != version:
        with tempfile.TemporaryDirectory(prefix="dokime-edgedriver-") as temporary:
            archive = Path(temporary) / "edgedriver.zip"
            urllib.request.urlretrieve(f"https://msedgedriver.microsoft.com/{version}/edgedriver_win64.zip", archive)
            with zipfile.ZipFile(archive) as downloaded:
                executable = downloaded.read("msedgedriver.exe")
            destination.mkdir(parents=True, exist_ok=True)
            staged = destination / "msedgedriver.exe.new"
            staged.write_bytes(executable)
            actual = _driver_version(staged)
            if actual != version:
                staged.unlink()
                raise ValueError(f"Microsoft WebDriver {actual!r} differs from WebView2 {version}")
            staged.replace(driver)
    _step(f"WebView2 and Microsoft WebDriver use {version}")
    github_path = os.environ.get("GITHUB_PATH")
    if github_path:
        with Path(github_path).open("a", encoding="utf-8") as written:
            written.write(str(destination) + "\n")
    return driver


def _checkout_harmos_config(root: Path) -> Path | None:
    candidate = root / ".cargo/config.toml"
    if not candidate.is_file():
        return None
    declaration = tomllib.loads(candidate.read_text(encoding="utf-8"))
    patches = declaration.get("patch", {})
    git = patches.get(HARMOS_REPOSITORY, {})
    registry = patches.get("crates-io", {})
    return candidate if "harmos" in git or "harmos" in registry else None


def _install_harmos_release(staged: Path, identity: dict[str, Any], binary: str) -> str:
    with tempfile.TemporaryDirectory(prefix="harmos-release-") as temporary:
        parent = Path(temporary)
        checksums = parent / "SHA256SUMS"
        urllib.request.urlretrieve(f"{identity['release']}/SHA256SUMS", checksums)
        matches = []
        for line in checksums.read_text(encoding="utf-8").splitlines():
            fields = line.split()
            if len(fields) == 2 and fields[1].removeprefix("*") == identity["asset"]:
                matches.append(fields[0])
        if len(matches) != 1 or re.fullmatch(r"[0-9a-f]{64}", matches[0]) is None:
            raise ValueError("published Harmos checksums omit one exact executable digest")
        expected = matches[0]
        tool = f"github:stokker-technologies/harmos[asset_pattern={identity['asset']},bin={binary}]@{identity['version']}"
        installed = parent / "installed"
        environment = os.environ.copy()
        environment.pop("MISE_ENABLE_TOOLS", None)
        subprocess.run(
            ["mise", "install-into", tool, str(installed)],
            cwd=parent,
            env=environment,
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            check=True,
        )
        executables = list(installed.rglob(binary))
        if len(executables) != 1 or executables[0].is_symlink() or not executables[0].is_file():
            raise ValueError("published Harmos package omitted its required CLI")
        executable = executables[0]
        if any(path.is_symlink() for path in executable.parents if path.is_relative_to(installed)):
            raise ValueError("published Harmos executable has a linked parent")
        if hashlib.sha256(executable.read_bytes()).hexdigest() != expected:
            raise ValueError("published Harmos executable differs from SHA256SUMS")
        destination = staged / "bin" / binary
        destination.parent.mkdir()
        shutil.copy2(executable, destination)
        return expected


def _webview_version() -> str | None:
    if sys.platform != "win32":
        return None

    import winreg

    key = r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
    for hive in (winreg.HKEY_CURRENT_USER, winreg.HKEY_LOCAL_MACHINE):
        for view in (winreg.KEY_WOW64_32KEY, winreg.KEY_WOW64_64KEY):
            try:
                with winreg.OpenKey(hive, key, 0, winreg.KEY_READ | view) as installed:
                    version = str(winreg.QueryValueEx(installed, "pv")[0])
                    if version != "0.0.0.0":
                        return version
            except FileNotFoundError:
                continue
    return None


def _driver_version(driver: Path) -> str:
    output = subprocess.check_output(
        [str(driver), "--version"],
        text=True,
        encoding="utf-8",
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stderr=sys.stderr,
    )
    version = re.search(r"\b\d+\.\d+\.\d+\.\d+\b", output)
    return version.group() if version else ""


def _registry_pin(root: Path, dependencies: dict[str, Any], requirement: object) -> dict[str, str]:
    exact = isinstance(requirement, str) and requirement.startswith("=")
    if not exact or not isinstance(requirement, str) or len(requirement) < 2:
        raise ValueError("Harmos registry tools require an exact =version declaration")
    version = requirement[1:]
    if requirement != requirement.strip():
        raise ValueError("Harmos registry tools require an exact =version declaration")
    lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))

    for name in HARMOS_PACKAGES:
        declared = dependencies.get(name)
        if isinstance(declared, str):
            declared = {"version": declared}
        if not isinstance(declared, dict):
            raise TypeError(f"{name} must declare a dependency string or table")
        unsupported = any(key in declared for key in ("git", "rev", "branch", "tag", "path", "registry", "package"))
        if declared.get("version") != requirement or unsupported:
            raise ValueError(f"{name} must declare the same exact public crates.io version")

        locked = [package for package in lock.get("package", []) if package.get("name") == name]
        if len(locked) != 1:
            raise ValueError(f"{name} has no unambiguous matching locked public crates.io version")
        if locked[0].get("source") != CRATES_SOURCE or locked[0].get("version") != version:
            raise ValueError(f"{name} has no unambiguous matching locked public crates.io version")
    return {"registry": "crates-io", "version": version, "package": "cargo-harmos"}


def _source_key(identity: dict[str, str]) -> str:
    if "rev" in identity:
        return identity["rev"]
    if "source" in identity:
        return identity["source"]
    return f"{identity['registry']}-{identity['version']}"


def _is_revision(value: object) -> bool:
    return isinstance(value, str) and len(value) == 40 and all(character in "0123456789abcdef" for character in value)


def _locked_revision(root: Path, source: str) -> str:
    # A Local Path Override Leaves No Git Source In The Lockfile
    lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
    for package in lock.get("package", []):
        locked = package.get("source", "")
        if package.get("name") == "harmos" and locked.startswith(source):
            revision = locked.removeprefix(source)
            if _is_revision(revision):
                return revision
    raise ValueError("the Harmos branch has no locked public revision; a local path override is active")


def _override_source(directory: Path) -> str:
    """Identify The Exact Local Harmos Checkout Used By An Explicit Development Override."""
    root = Path(
        subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=directory,
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
        ).stdout.strip()
    )
    files = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root,
        capture_output=True,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
    ).stdout.split(b"\0")
    digest = hashlib.sha256()
    for name in sorted(set(files) - {b""}):
        path = root / os.fsdecode(name)
        digest.update(name + b"\0")
        if path.is_symlink():
            digest.update(b"link\0" + os.fsencode(os.readlink(path)))
        elif path.is_file():
            digest.update(b"file\0" + hashlib.sha256(path.read_bytes()).digest())
        else:
            digest.update(b"missing\0")
    return digest.hexdigest()


def _package_installed(name: str) -> bool:
    return (
        subprocess.run(
            ["dpkg", "-s", name],
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=False,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
        ).returncode
        == 0
    )


def _run(command: Sequence[str]) -> None:
    _step(" ".join(command))
    subprocess.run(
        command, check=True, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0), stdin=sys.stdin, stdout=sys.stdout, stderr=sys.stderr
    )


def _step(message: str) -> None:
    print(f"[ci:bootstrap] {message}", flush=True)


if __name__ == "__main__":
    sys.exit(main())
