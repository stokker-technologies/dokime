# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Focused C++ SDK Build, Proof and Source Distribution Tasks."""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path
from time import perf_counter

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.ci.native import visual_studio
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[3]

# === Types === #

# === Classes === #

# === Public Functions === #


def run(command: list[str], *, action: str, cwd: Path, env: dict[str, str], log: Path) -> None:
    print(f"{action}: {subprocess.list2cmdline(command)}", flush=True)
    if log.exists():
        attempt = len(list(log.parent.glob(f"{log.stem}-*.log"))) + 1
        shutil.copyfile(log, log.with_name(f"{log.stem}-{attempt}.log"))
    started = perf_counter()
    output = log.open("w", encoding="utf-8")
    try:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdin=sys.stdin,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
            creationflags=spawn_flags(),
        )
        try:
            if process.stdout is None:
                raise RuntimeError(f"{action}: Connect Child Output and Rerun the Task")
            for line in process.stdout:
                output.write(line)
                output.flush()
                print(line, end="", flush=True)
        finally:
            if process.stdout is not None:
                process.stdout.close()
            returncode = process.wait()
    finally:
        output.close()
    print(f"{action}: Exit {returncode} After {perf_counter() - started:.2f}s; Log: {log}", flush=True)
    if returncode:
        print(
            f"{action} Failed: Inspect {log}, Fix the Reported Error and Rerun {subprocess.list2cmdline(command)}",
            file=sys.stderr,
            flush=True,
        )
        raise subprocess.CalledProcessError(returncode, command)


def environment() -> dict[str, str]:
    environment = dict(os.environ)
    # Locate the Pinned Dependency Checkout.
    print("Prepare C++ Dependencies: Locate the Mise Cache", flush=True)
    cache = subprocess.check_output(["mise", "cache", "path"], text=True, stderr=sys.stderr, creationflags=spawn_flags()).strip()
    manifest = json.loads((ROOT / "sdk/cpp/vcpkg.json").read_text())
    revision = manifest["builtin-baseline"]
    checkout = Path(cache) / "vcpkg" / revision[:12]
    environment["VCPKG_ROOT"] = str(checkout)

    # Bootstrap the Immutable Tool Checkout Outside the Repository.
    if not checkout.is_dir():
        checkout.parent.mkdir(parents=True, exist_ok=True)
        print("Prepare C++ Dependencies: Clone vcpkg", flush=True)
        subprocess.run(
            [
                "git",
                "clone",
                "--filter=blob:none",
                "--single-branch",
                "--branch",
                "2025.06.13",
                "https://github.com/microsoft/vcpkg.git",
                str(checkout),
            ],
            check=True,
            stdout=sys.stdout,
            stderr=sys.stderr,
            creationflags=spawn_flags(),
        )
    actual = subprocess.check_output(
        ["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True, stderr=sys.stderr, creationflags=spawn_flags()
    ).strip()
    if actual != revision:
        raise RuntimeError(f"Prepare C++ Dependencies: Restore vcpkg Checkout {checkout} to {revision}; Found {actual}")
    executable = checkout / ("vcpkg.exe" if os.name == "nt" else "vcpkg")
    if not executable.is_file():
        print("Prepare C++ Dependencies: Bootstrap vcpkg", flush=True)
        bootstrap = (
            ["cmd", "/c", str(checkout / "bootstrap-vcpkg.bat"), "-disableMetrics"]
            if os.name == "nt"
            else ["sh", str(checkout / "bootstrap-vcpkg.sh"), "-disableMetrics"]
        )
        subprocess.run(bootstrap, check=True, stdout=sys.stdout, stderr=sys.stderr, creationflags=spawn_flags())

    # Ninja Uses the Native MSVC Environment on Windows.
    if os.name == "nt" and not environment.get("VCINSTALLDIR"):
        print("Prepare C++ Compiler: Load the MSVC Developer Environment", flush=True)
        tool_path = environment["PATH"]
        installer = Path(environment["PROGRAMFILES(X86)"]) / "Microsoft Visual Studio/Installer"
        developer = visual_studio() / "Common7/Tools/VsDevCmd.bat"
        output = subprocess.check_output(
            f'cmd /d /s /c ""{developer}" -no_logo -arch=x64 -host_arch=x64 >nul && set"',
            text=True,
            env=dict(environment, PATH=str(installer) + os.pathsep + environment["PATH"]),
            stderr=sys.stderr,
            creationflags=spawn_flags(),
        )
        environment.update(
            (key.upper(), value)
            for line in output.splitlines()
            if "=" in line and not line.startswith("=")
            for key, value in [line.split("=", 1)]
        )
        environment["PATH"] = tool_path + os.pathsep + environment["PATH"]
    environment["VCPKG_ROOT"] = str(checkout)
    return environment


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "build", "test", "check", "format", "package", "consumer"))
    parser.add_argument("--preset", choices=("development", "tests", "release"), default="tests")
    parser.add_argument("--target")
    parser.add_argument("--exact")
    parser.add_argument("--repeat", type=int)
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--source", type=Path)
    options = parser.parse_args()
    sdk = ROOT / "sdk/cpp"
    directory = ROOT / "target/cp" / options.preset
    directory.mkdir(parents=True, exist_ok=True)
    child = dict(os.environ) if options.action == "format" else environment()
    if options.action == "prepare":
        return
    if options.action == "format":
        formatter = subprocess.check_output(
            ["mise", "which", "clang-format"], cwd=sdk, text=True, stderr=sys.stderr, creationflags=spawn_flags()
        ).strip()
        sources = (
            [options.source]
            if options.source
            else sorted([*sdk.rglob("*.cpp"), *sdk.rglob("*.hpp"), *ROOT.glob("sdk/tests/fixtures/cpp/**/*.cpp")])
        )
        if not options.check:
            sources = [path for path in sources if not path.read_text(encoding="utf-8").startswith("// @generated")]
        if not sources:
            raise RuntimeError("Format C++ SDK: Select a Handwritten SDK Source")
        # Match the Shared Resource Renderer's Formatting Policy.
        command = [
            formatter,
            "-style={BasedOnStyle: LLVM, ColumnLimit: 140, AllowShortFunctionsOnASingleLine: Empty, AllowShortLambdasOnASingleLine: None}",
        ]
        command += ["--dry-run", "--Werror"] if options.check else ["-i"]
        command += [str(path) for path in sources]
    elif options.action in ("check", "build"):
        run(
            ["cmake", "--preset", options.preset, f"-DPython3_EXECUTABLE={sys.executable}"],
            action="Configure C++ SDK",
            cwd=sdk,
            env=child,
            log=directory / "configure.log",
        )
        if options.action != "build":
            return
        command = ["cmake", "--build", "--preset", options.preset, "--parallel", "4"]
        if options.target:
            command += ["--target", options.target]
    elif options.action == "test":
        print("Test C++ SDK: Check That the Build Is Current", flush=True)
        pending = subprocess.check_output(
            ["cmake", "--build", str(directory), "--", "-n"], cwd=sdk, env=child, text=True, stderr=sys.stderr, creationflags=spawn_flags()
        )
        if pending.strip() != "ninja: no work to do.":
            raise RuntimeError(f"Test C++ SDK: Build This Preset Before Running Its Stories\n{pending.strip()}")
        command = ["ctest", "--preset", options.preset, "--output-junit", str(directory / "junit.xml")]
        if options.exact:
            command += ["-R", "^" + re.escape(options.exact) + "$"]
        if options.verbose:
            command.append("--verbose")
        if options.repeat:
            command += ["--repeat", f"until-fail:{options.repeat}"]
    elif options.action == "consumer":
        pending = subprocess.check_output(
            ["cmake", "--build", str(directory), "--", "-n"], cwd=sdk, env=child, text=True, stderr=sys.stderr, creationflags=spawn_flags()
        )
        if pending.strip() != "ninja: no work to do.":
            raise RuntimeError(f"Build Installed Consumer: Build This Preset Before Installing Its Package\n{pending.strip()}")
        prefix = directory / "installed"
        run(
            ["cmake", "--install", str(directory), "--prefix", str(prefix)],
            action="Install C++ SDK",
            cwd=sdk,
            env=child,
            log=directory / "install.log",
        )
        consumer = ROOT / "target/cp/consumer"
        run(
            [
                "cmake",
                "--fresh",
                "--preset",
                "release",
                "-S",
                str(options.source or sdk / "examples/test"),
                "-B",
                str(consumer),
                "-G",
                "Ninja",
                f"-DCMAKE_TOOLCHAIN_FILE={child['VCPKG_ROOT']}/scripts/buildsystems/vcpkg.cmake",
                f"-DCMAKE_PREFIX_PATH={prefix}",
                f"-DVCPKG_INSTALLED_DIR={ROOT}/target/cp/v",
                "-DVCPKG_MANIFEST_INSTALL=OFF",
                "-DCMAKE_BUILD_TYPE=Debug",
            ],
            cwd=options.source or sdk / "examples/test",
            env=child,
            action="Configure Installed C++ Consumer",
            log=directory / "consumer-configure.log",
        )
        command = ["cmake", "--build", str(consumer), "--parallel", "4"]
    else:
        pending = subprocess.check_output(
            ["cmake", "--build", str(directory), "--", "-n"], cwd=sdk, env=child, text=True, stderr=sys.stderr, creationflags=spawn_flags()
        )
        if pending.strip() != "ninja: no work to do.":
            raise RuntimeError(f"Package C++ SDK: Build This Preset Before Installing Its Package\n{pending.strip()}")
        prefix = directory / "installed"
        run(
            ["cmake", "--install", str(directory), "--prefix", str(prefix)],
            action="Install C++ SDK",
            cwd=sdk,
            env=child,
            log=directory / "install.log",
        )
        archive = tarfile.TarFile.gzopen(directory / "dokime-cpp.tar.gz", "w")
        try:
            archive.add(prefix, arcname="dokime-cpp")
        finally:
            archive.close()
        return
    run(command, action=f"{options.action.title()} C++ SDK", cwd=sdk, env=child, log=directory / f"{options.action}.log")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        main()
    except (subprocess.CalledProcessError, OSError, RuntimeError) as error:
        if isinstance(error, subprocess.CalledProcessError) and error.stdout:
            print(error.stdout, end="", file=sys.stderr, flush=True)
        print(f"C++ SDK Failed: {error}; Correct the Reported Step and Rerun This Task", file=sys.stderr, flush=True)
        raise SystemExit(error.returncode if isinstance(error, subprocess.CalledProcessError) else 1) from error
