"Prepare Native Tools and Stable Toolchain Roots Before Mise."

# === Imports === #

# Standard Library Imports
import argparse
import contextlib
import ctypes
import hashlib
import http.client
import json
import os
import random
import re
import runpy
import shlex
import shutil
import ssl
import subprocess
import sys
import tempfile
import time
import tomllib
import urllib.error
import urllib.request
import zipfile
from collections.abc import Callable, Mapping
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #


ZSTD_URL = "https://github.com/facebook/zstd/releases/download/v1.5.7/zstd-v1.5.7-win64.zip"
ZSTD_SHA256 = "acb4e8111511749dc7a3ebedca9b04190e37a17afeb73f55d4425dbf0b90fad9"
MISE_URL = "https://github.com/jdx/mise/releases/download/v2026.10.17/mise-v2026.10.17-windows-x64.zip"
MISE_SHA256 = "32bcfda5f72bf7cb189e18ba93c2798bffcb4a678fd004e3127780363f2d8c8c"
DOWNLOAD_ATTEMPTS = 6
DOWNLOAD_PAUSE_SECONDS = 2
DOWNLOAD_TIMEOUT_SECONDS = 120
EXPORTED = {}
# === Types === #

# === Classes === #

# === Public Functions === #


def spawn_flags(flags: int = 0) -> int:
    """Load Shared Process Policy Before Mise Can Install the Tools Package."""
    source = Path(__file__).resolve().parents[1] / "process/__init__.py"
    module = runpy.run_path(str(source))
    operation: Callable[[int], int] = module["spawn_flags"]
    return operation(flags)


def roots(environment: Mapping[str, str], workspace: Path, home: Path) -> dict[str, str]:
    """Keep Installation Identity Independent of the Runner's Per-Job Temp Directory."""
    default_data = home / ".local/share/mise"
    if environment.get("RUNNER_OS") == "Windows" and environment.get("RUNNER_TEMP"):
        # Keep Mise On The Work Disk Beside Its Temporary Download And Extraction.
        default_data = Path(environment["RUNNER_TEMP"]) / "mise"
    data = Path(environment.get("MISE_DATA_DIR") or default_data)
    cargo = Path(environment.get("CARGO_HOME") or home / ".cargo")
    rustup = Path(environment.get("RUSTUP_HOME") or home / ".rustup")
    return {
        "MISE_DATA_DIR": str(data),
        "CARGO_HOME": str(cargo),
        "RUSTUP_HOME": str(rustup),
        "MISE_RUSTUP_HOME": str(rustup),
        "MISE_CARGO_HOME": str(cargo),
        "MISE_ROOT_ID": hashlib.sha256(str(data.resolve()).encode()).hexdigest(),
        "WORKSPACE_ROOT_ID": hashlib.sha256(str(workspace.resolve()).encode()).hexdigest(),
        "RUNNER_IMAGE_ID": environment.get("ImageOS", os.name) + "-" + environment.get("ImageVersion", "self"),
    }


def export(values: dict[str, str]) -> None:
    EXPORTED.update(values)
    os.environ.update(values)
    if destination := os.environ.get("GITHUB_ENV"):
        records = []
        for name, value in values.items():
            if "\n" in value or "\r" in value:
                raise ValueError(f"Prepare Native Prerequisites: invalid environment value: {name}")
            records.append(f"{name}={value}\n")
        path = Path(destination)
        previous = path.read_text(encoding="utf-8") if path.exists() else ""
        path.write_text(previous + "".join(records), encoding="utf-8", newline="\n")


def download(
    url: str,
    checksum: str,
    destination: Path,
    opener: Callable[..., http.client.HTTPResponse] = urllib.request.urlopen,
    sleep: Callable[[float], None] = time.sleep,
) -> bytes:
    """Fetch a Pinned Archive, Retrying Transient Release CDN Failures with Bounded Backoff."""
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.is_file() and hashlib.sha256(destination.read_bytes()).hexdigest() == checksum:
        return destination.read_bytes()
    cause = None
    for attempt in range(1, DOWNLOAD_ATTEMPTS + 1):
        try:
            response = opener(url, timeout=DOWNLOAD_TIMEOUT_SECONDS)
            try:
                content = response.read()
            finally:
                response.close()
            break
        except urllib.error.HTTPError as error:
            error.close()
            if error.code < 500 and error.code != 429:
                raise RuntimeError(
                    f"Prepare Native Prerequisites: bootstrap archive download failed: {url}: HTTP {error.code}; check the pinned URL"
                ) from error
            cause = f"HTTP {error.code}"
        except (urllib.error.URLError, TimeoutError, ssl.SSLError, ConnectionError, http.client.HTTPException) as error:
            cause = f"{type(error).__name__}: {error}"
        if attempt == DOWNLOAD_ATTEMPTS:
            raise RuntimeError(
                f"Prepare Native Prerequisites: bootstrap archive download failed after {DOWNLOAD_ATTEMPTS} attempts: {url}: {cause}; GitHub release downloads are degraded, rerun the job"
            )
        wait = DOWNLOAD_PAUSE_SECONDS * 2 ** (attempt - 1) + random.uniform(0, 1)
        print(f"bootstrap archive download attempt {attempt}/{DOWNLOAD_ATTEMPTS} failed: {cause}; retrying in {wait:.1f} s", flush=True)
        sleep(wait)
    if hashlib.sha256(content).hexdigest() != checksum:
        raise ValueError(f"Prepare Native Prerequisites: pinned bootstrap archive checksum mismatch: {url}")
    # A Partial Or Interrupted Write Never Passes As The Archive.
    descriptor, name = tempfile.mkstemp(dir=destination.parent, prefix=destination.name, suffix=".partial")
    os.close(descriptor)
    temporary = Path(name)
    try:
        temporary.write_bytes(content)
        temporary.replace(destination)
    finally:
        temporary.unlink(missing_ok=True)
    return content


def visual_studio() -> Path:
    """Select the Latest Visual Studio Installation with the Native C++ Toolchain."""
    if sys.platform != "win32":
        raise OSError("Locate Visual Studio: Run Native C++ Setup on Windows")
    vswhere = Path(os.environ["ProgramFiles(x86)"]) / "Microsoft Visual Studio/Installer/vswhere.exe"
    installation = subprocess.check_output(
        [
            str(vswhere),
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
        ],
        text=True,
        creationflags=spawn_flags(),
    ).strip()
    if not installation:
        raise RuntimeError("Locate Visual Studio: Install the Visual Studio C++ Tools and Rerun Setup")
    return Path(installation)


def windows(workspace: Path) -> None:
    if sys.platform != "win32":
        raise OSError("Prepare Native Prerequisites: Windows provisioning requires Windows")
    temporary_root(workspace)
    vcvars = visual_studio() / "VC/Auxiliary/Build/vcvars64.bat"
    # Preserve Cmd Quoting Through CreateProcess.
    output = subprocess.check_output(f'cmd.exe /d /s /c ""{vcvars}" >nul && set"', text=True, creationflags=spawn_flags())
    values = dict(line.split("=", 1) for line in output.splitlines() if "=" in line and not line.startswith("="))
    export({name.upper(): value for name, value in values.items() if name.upper() in {"PATH", "INCLUDE", "LIB", "LIBPATH"}})

    archive = workspace / ".cache/bootstrap/zstd-1.5.7/zstd.zip"
    download(ZSTD_URL, ZSTD_SHA256, archive)
    bundle = zipfile.ZipFile(archive)
    try:
        bundle.extractall(archive.parent)
    finally:
        bundle.close()
    compressor = archive.parent / "zstd-v1.5.7-win64"
    candidates = [Path(item.strip('"')) / "tar.exe" for item in os.environ["PATH"].split(os.pathsep)]
    git = shutil.which("git")
    if git:
        candidates += [Path(git).parent.parent / "usr/bin/tar.exe"]
    tar = next(
        (
            path
            for path in candidates
            if path.is_file()
            and "GNU tar"
            in subprocess.run([str(path), "--version"], capture_output=True, text=True, check=False, creationflags=spawn_flags()).stdout
        ),
        None,
    )
    if tar is None:
        raise RuntimeError("Prepare Native Prerequisites: GNU tar is required from the standard Git image")
    compiler = shutil.which("cl")
    if compiler is None:
        raise RuntimeError("Prepare Native Prerequisites: MSVC compiler is required")
    export({"PATH": os.pathsep.join([str(Path(compiler).parent), str(compressor), str(tar.parent), os.environ["PATH"]])})
    if not shutil.which("cl") or not shutil.which("link"):
        raise RuntimeError("Prepare Native Prerequisites: MSVC compiler and linker are required")

    if shutil.which("mise") is None:
        archive = workspace / ".cache/bootstrap/mise-2026.10.17/mise.zip"
        download(MISE_URL, MISE_SHA256, archive)
        bundle = zipfile.ZipFile(archive)
        try:
            bundle.extractall(archive.parent)
        finally:
            bundle.close()
        binary = archive.parent / "mise/bin"
        export({"PATH": os.pathsep.join([str(binary), os.environ["PATH"]])})
        subprocess.run(["mise", "--version"], check=True, stdout=sys.stderr, creationflags=spawn_flags())

    if os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted" and ctypes.windll.shell32.IsUserAnAdmin():
        exclusions = [
            str(workspace),
            str(Path(os.environ.get("CARGO_TARGET_DIR", str(workspace / "target"))).resolve()),
            os.environ["CARGO_HOME"],
            os.environ["MISE_DATA_DIR"],
        ]
        # The Runner-Owned Temporary Root Holds Every Lane's Scratch And Test Store; Excluding It
        # Whole Avoids Wildcard Exclusions Defender Does Not Honour On Every Volume.
        temporary_roots = native_temporary_paths()
        exclusions.extend(str(root) for root in temporary_roots)
        print("Native Scanner Exclusions: " + json.dumps(exclusions), flush=True)
        script = (
            "$paths = @("
            + ",".join("'" + path.replace("'", "''") + "'" for path in exclusions)
            + "); Add-MpPreference -ExclusionPath $paths -ErrorAction Stop; "
            "$actual = (Get-MpPreference -ErrorAction Stop).ExclusionPath; "
            "foreach ($path in $paths) { if ($actual -notcontains $path) { throw ('Native scanner exclusion was not applied: ' + $path) } }"
        )
        # Hosted Image Provisioning Is The Only Place That Changes Defender Policy.
        subprocess.run(
            ["powershell", "-NoProfile", "-NonInteractive", "-Command", script], check=True, stdout=sys.stderr, creationflags=spawn_flags()
        )
        verify_temporary_exclusions(temporary_roots)


def temporary_root(workspace: Path) -> None:
    """Keep Scratch on the Workspace Volume: Hosted Images Put TEMP on the IOPS-Capped OS Disk."""
    workspace = workspace.resolve()
    current = Path(tempfile.gettempdir()).resolve()
    if current.drive.casefold() != workspace.drive.casefold():
        runner = os.environ.get("RUNNER_TEMP")
        runner = Path(runner).resolve() if runner else None
        current = runner if runner and runner.drive.casefold() == workspace.drive.casefold() else Path(workspace.anchor) / "tmp"
        if current.is_relative_to(workspace):
            raise RuntimeError(f"Prepare Native Prerequisites: temporary directory must stay outside the checkout: {current}")
        current.mkdir(parents=True, exist_ok=True)
        export({"TEMP": str(current), "TMP": str(current)})
        tempfile.tempdir = None


def native_temporary_paths() -> list[Path]:
    if sys.platform != "win32":
        raise OSError("Prepare Native Prerequisites: native temporary directory lookup requires Windows")
    buffer = ctypes.create_unicode_buffer(32768)
    length = ctypes.windll.kernel32.GetTempPath2W(len(buffer), buffer)
    if not length or length >= len(buffer):
        raise OSError("Prepare Native Prerequisites: native temporary directory lookup failed")
    roots = (Path(buffer.value), Path(tempfile.gettempdir()))
    return list(dict.fromkeys(candidate for root in roots for candidate in (root, root.resolve())))


def scanner_status() -> dict:
    output = subprocess.check_output(
        [
            "powershell",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-MpComputerStatus | Select-Object AMRunningMode, RealTimeProtectionEnabled, OnAccessProtectionEnabled | ConvertTo-Json",
        ],
        text=True,
        creationflags=spawn_flags(),
    )
    return json.loads(output)


def verify_temporary_exclusions(roots: list[Path]) -> None:
    status = scanner_status()
    print("Native Scanner Status: " + json.dumps(status), flush=True)
    # A Scanner That Does Not Scan On Access Has Nothing To Exclude, And Its Exclusion Check Reports Nothing Excluded.
    if status.get("RealTimeProtectionEnabled") is False or status.get("OnAccessProtectionEnabled") is False:
        print("Native Scanner Check: on-access scanning is off; the exclusions take effect if it is enabled", flush=True)
        return
    scanner = Path(os.environ["ProgramFiles"]) / "Windows Defender/MpCmdRun.exe"
    for root in roots:
        for prefix in ("dokime-exclusion-", ".tmp", "project-tmp-"):
            directory_directory = tempfile.TemporaryDirectory(dir=root, prefix=prefix)
            directory = directory_directory.name
            try:
                descriptor, name = tempfile.mkstemp(dir=root, prefix=prefix)
                os.close(descriptor)
                try:
                    nested = Path(directory) / "probe.sqlite"
                    nested.write_bytes(b"")
                    for path in (Path(directory), nested, Path(name)):
                        print(f"Native Scanner Check: {path}", flush=True)
                        checked = subprocess.run(
                            [str(scanner), "-CheckExclusion", "-Path", str(path)],
                            check=False,
                            stdout=sys.stderr,
                            creationflags=spawn_flags(),
                        )
                        if checked.returncode:
                            # Show What Defender Holds Next To The Refused Probe.
                            subprocess.run(
                                ["powershell", "-NoProfile", "-NonInteractive", "-Command", "(Get-MpPreference).ExclusionPath"],
                                check=False,
                                stdout=sys.stderr,
                                creationflags=spawn_flags(),
                            )
                            raise RuntimeError(f"Prepare Native Prerequisites: native scanner exclusion does not cover {path} under {root}")
                finally:
                    Path(name).unlink(missing_ok=True)
            finally:
                directory_directory.cleanup()


def unix() -> None:
    tools = os.environ.get("MISE_ENABLE_TOOLS", "rust,node").split(",")
    compiler = "rust" in tools or any(tool.startswith("cargo:") for tool in tools)
    required = ["tar", "zstd", "unzip", "zip"] + (["cc", "c++", "make", "pkg-config"] if compiler else [])
    missing = any(shutil.which(tool) is None for tool in required)
    if compiler and shutil.which("pkg-config"):
        missing |= subprocess.run(["pkg-config", "--exists", "openssl"], check=False, creationflags=spawn_flags()).returncode != 0
    atomic = "node" in tools or any(tool.startswith("npm:") for tool in tools)
    if atomic:
        missing |= not atomic_available()
    if missing:
        elevate = ["sudo", "-n"]
        if hasattr(os, "geteuid") and os.geteuid() == 0:
            elevate = []
        if shutil.which("apt-get"):
            packages = (
                ["tar", "zstd", "unzip", "zip"]
                + (["build-essential", "pkg-config", "libssl-dev"] if compiler else [])
                + (["libatomic1"] if atomic else [])
            )
            subprocess.run([*elevate, "apt-get", "update"], check=True, creationflags=spawn_flags())
            subprocess.run(
                [*elevate, "apt-get", "install", "-y", "--no-install-recommends", *packages], check=True, creationflags=spawn_flags()
            )
        elif shutil.which("apk"):
            packages = (
                ["tar", "zstd", "unzip", "zip"]
                + (["build-base", "pkgconf", "openssl-dev"] if compiler else [])
                + (["libatomic"] if atomic else [])
            )
            subprocess.run([*elevate, "apk", "add", "--no-cache", *packages], check=True, creationflags=spawn_flags())
        else:
            raise RuntimeError("Prepare Native Prerequisites: provide native CI prerequisites through the declared image or bootstrap")
    if "GNU tar" not in subprocess.check_output(["tar", "--version"], text=True, creationflags=spawn_flags()):
        raise RuntimeError("Prepare Native Prerequisites: GNU tar is required")


def atomic_available() -> bool:
    if shutil.which("ldconfig"):
        result = subprocess.run(["ldconfig", "-p"], check=False, capture_output=True, text=True, creationflags=spawn_flags())
        if "libatomic.so.1" in result.stdout:
            return True
    directories = [Path(path) for path in ("/lib", "/lib64", "/usr/lib", "/usr/lib64")]
    return any((directory / "libatomic.so.1").is_file() or any(directory.glob("*/libatomic.so.1")) for directory in directories)


def rustup() -> None:
    # Mise Reuses The Existing Rustup Installation Instead Of Bootstrapping A Competing One.
    proxies = Path(os.environ["CARGO_HOME"]) / "bin"
    existing_rustup = shutil.which("rustup")
    if not (proxies / ("rustup.exe" if os.name == "nt" else "rustup")).is_file() and existing_rustup:
        existing = Path(existing_rustup).resolve()
        proxies.mkdir(parents=True, exist_ok=True)
        for name in ("rustup", "cargo", "rustc", "rustdoc", "rustfmt", "cargo-clippy", "clippy-driver"):
            filename = name + (".exe" if os.name == "nt" else "")
            destination = proxies / filename
            if not destination.exists():
                shutil.copy2(existing, destination)
    rustup = proxies / ("rustup.exe" if os.name == "nt" else "rustup")
    if rustup.is_file() and not (Path(os.environ["RUSTUP_HOME"]) / "settings.toml").is_file():
        subprocess.run([str(rustup), "set", "profile", "default"], check=True, stdout=sys.stderr, creationflags=spawn_flags())
    export({"PATH": os.pathsep.join([str(proxies), os.environ["PATH"]])})


def glab(version: str) -> None:
    if re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version) is None:
        raise ValueError("Prepare Native Prerequisites: GitLab CLI bootstrap requires an exact version")
    # This Explicit CI Tool Is Outside The Project Lock; Project Installs Remain Locked.
    subprocess.run(
        ["mise", "install", "--yes", f"glab@{version}"],
        env=os.environ | {"MISE_LOCKED": "0", "MISE_ENABLE_TOOLS": "glab"},
        stdout=sys.stderr,
        creationflags=spawn_flags(),
        check=True,
    )
    export({"MISE_GLAB_VERSION": version})


def default_caches(workspace: Path, environment: Mapping[str, str]) -> dict[str, dict[str, str]]:
    """Derive Compatible Provider Cache Keys From Their Actual Inputs."""

    def digest(*names: str) -> str:
        value = hashlib.sha256()
        for name in names:
            value.update((workspace / name).read_bytes())
        return value.hexdigest()

    platform = environment["RUNNER_OS"] + "-" + environment["RUNNER_ARCH"]
    tools = digest("mise.lock")
    caches = {
        "cargo_downloads": {"path": ".cache/cargo/registry/\n.cache/cargo/git/", "key": f"ci-v6-cargo-{platform}-" + digest("Cargo.lock")},
        "mise": {"path": "<MISE_DATA_DIR>", "key": f"ci-v6-mise-{platform}-{tools}-<RUNNER_IMAGE_ID>-<MISE_ROOT_ID>-<WORKSPACE_ROOT_ID>"},
        "uv": {"path": ".cache/uv/", "key": f"ci-v6-uv-{platform}-{tools}-" + digest("uv.lock")},
        "pnpm": {
            "path": ".cache/pnpm/",
            "key": f"ci-v6-pnpm-{platform}-{tools}-" + digest("crates/dokime-gui/pnpm-lock.yaml", "sites/docs/pnpm-lock.yaml"),
        },
    }
    if environment.get("COMPILER_CACHE_BACKEND") == "disk":
        profile = environment.get("CI_CACHE_PROFILE") or environment.get("PROFILE", "dev")
        prefix = f"ci-v6-sccache-{platform}-{profile}-" + environment["RUST_TOOLCHAIN"] + "-"
        caches["sccache"] = {"path": ".cache/sccache/", "key": prefix + digest("Cargo.lock"), "restore-keys": prefix}
    if environment.get("CI_BENCH") == "true":
        layer = environment.get("CI_BENCH_LAYER", "build")
        prefix = f"ci-v6-baselines-{platform}-{layer}-"
        run = environment["GITHUB_RUN_ID"] + "-" + environment["GITHUB_RUN_ATTEMPT"]
        caches["bench_baselines"] = {"path": "target/benchmarks/", "key": prefix + run, "restore-keys": prefix}
    return caches


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--platform", choices=("unix", "windows"), default=os.environ.get("SETUP_PLATFORM", "windows" if os.name == "nt" else "unix")
    )
    parser.add_argument("--roots-only", action="store_true")
    parser.add_argument("--emit-shell", action="store_true")
    parser.add_argument("--glab-version")
    options = parser.parse_args()
    if options.glab_version:
        glab(options.glab_version)
        return
    workspace = Path.cwd()
    rust = tomllib.loads((workspace / "mise.toml").read_text(encoding="utf-8"))["tools"]["rust"]
    export({"RUST_TOOLCHAIN": rust["version"] if isinstance(rust, dict) else rust})
    print("::group::" + "Toolchain Roots", flush=True)
    try:
        export(roots(os.environ, workspace, Path.home()))
        export({name: os.environ[name] for name in ("COMPILER_CACHE_BACKEND", "COMPILER_CACHE_MODE") if os.environ.get(name)})
        rustup()
        if destination := os.environ.get("GITHUB_OUTPUT"):
            records = []
            declared = json.loads(os.environ.get("CACHE_PLAN", "{}"))
            caches = declared or default_caches(workspace, os.environ)
            for name, cache in caches.items():
                for field in ("path", "key", "restore-keys"):
                    value = cache.get(field, "")
                    for variable in ("MISE_DATA_DIR", "MISE_ROOT_ID", "WORKSPACE_ROOT_ID", "RUNNER_IMAGE_ID", "RUST_TOOLCHAIN"):
                        value = value.replace("<" + variable + ">", os.environ[variable])
                    if field == "path" and not value.strip():
                        raise ValueError(f"Prepare Native Prerequisites: cache path is empty: {name}")
                    records.append(f"{name}-{field}<<CACHE_VALUE\n{value}\nCACHE_VALUE\n")
            path = Path(destination)
            previous = path.read_text(encoding="utf-8") if path.exists() else ""
            path.write_text(previous + "".join(records), encoding="utf-8", newline="\n")
    finally:
        print("::endgroup::", flush=True)
    if options.roots_only:
        return
    print("::group::" + "Native Prerequisites", flush=True)
    try:
        windows(workspace) if options.platform == "windows" else unix()
    finally:
        print("::endgroup::", flush=True)
    subprocess.run(["zstd", "--version"], check=True, stdout=sys.stderr, creationflags=spawn_flags())


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    if "--emit-shell" in sys.argv:
        with contextlib.redirect_stdout(sys.stderr):
            main()
        for name, value in EXPORTED.items():
            if os.name == "nt":
                escaped = value.replace("'", "''")
                print(f"$env:{name}='{escaped}'")
            else:
                print(f"export {name}={shlex.quote(value)}")
    else:
        main()
