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
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.request
import zipfile
from collections.abc import Sequence
from pathlib import Path

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


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parse_arguments(argv)
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
        description="Prepare explicitly selected native prerequisites.",
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
    parser.add_argument("--desktop", action="store_true")
    return parser.parse_args(argv)


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
