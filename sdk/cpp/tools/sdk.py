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

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[3]

# === Types === #

# === Classes === #

# === Public Functions === #


def environment() -> dict[str, str]:
    environment = dict(os.environ)
    cache = subprocess.check_output(
        ["mise", "cache", "path"], text=True, creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
    ).strip()
    manifest = json.loads((ROOT / "sdk/cpp/vcpkg.json").read_text())
    revision = manifest["builtin-baseline"]
    checkout = Path(cache) / "vcpkg" / revision[:12]
    environment["VCPKG_ROOT"] = str(checkout)

    # Bootstrap the Immutable Tool Checkout Outside the Repository.
    if not checkout.is_dir():
        checkout.parent.mkdir(parents=True, exist_ok=True)
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
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
    actual = subprocess.check_output(
        ["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True, creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
    ).strip()
    if actual != revision:
        raise RuntimeError(f"vcpkg Checkout: Expected {revision}, Got {actual}")
    executable = checkout / ("vcpkg.exe" if os.name == "nt" else "vcpkg")
    if not executable.is_file():
        bootstrap = (
            ["cmd", "/c", str(checkout / "bootstrap-vcpkg.bat"), "-disableMetrics"]
            if os.name == "nt"
            else ["sh", str(checkout / "bootstrap-vcpkg.sh"), "-disableMetrics"]
        )
        subprocess.run(
            bootstrap, check=True, stdout=sys.stdout, stderr=sys.stderr, creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
        )

    # Ninja Uses the Native MSVC Environment on Windows.
    if os.name == "nt" and not environment.get("VCINSTALLDIR"):
        tool_path = environment["PATH"]
        vswhere = Path(environment["PROGRAMFILES(X86)"]) / "Microsoft Visual Studio/Installer/vswhere.exe"
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
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        ).strip()
        developer = Path(installation) / "Common7/Tools/VsDevCmd.bat"
        output = subprocess.check_output(
            f'cmd /d /s /c ""{developer}" -no_logo -arch=x64 -host_arch=x64 >nul && set"',
            text=True,
            env=dict(environment, PATH=str(vswhere.parent) + os.pathsep + environment["PATH"]),
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
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
    parser.add_argument(
        "action", choices=("prepare", "configure", "build", "test", "check", "format", "package", "consumer", "conformance", "clean")
    )
    parser.add_argument("--preset", choices=("development", "tests", "release"), default="tests")
    parser.add_argument("--target")
    parser.add_argument("--exact")
    parser.add_argument("--repeat", type=int)
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--source", type=Path)
    parser.add_argument("--host", type=Path)
    options = parser.parse_args()
    sdk = ROOT / "sdk/cpp"
    directory = ROOT / "target/cp" / options.preset
    directory.mkdir(parents=True, exist_ok=True)
    child = dict(os.environ) if options.action == "format" else environment()
    if options.action == "prepare":
        return
    if options.action == "format":
        formatter = shutil.which("clang-format", path=child["PATH"])
        sources = (
            [options.source]
            if options.source
            else sorted([*sdk.rglob("*.cpp"), *sdk.rglob("*.hpp"), *ROOT.glob("sdk/tests/fixtures/cpp/**/*.cpp")])
        )
        command = [formatter or "clang-format", "-style={BasedOnStyle: LLVM, ColumnLimit: 140}"]
        command += ["--dry-run", "--Werror"] if options.check else ["-i"]
        command += [str(path) for path in sources]
    elif options.action in ("configure", "check", "build"):
        result = subprocess.run(
            ["cmake", "--preset", options.preset],
            cwd=sdk,
            env=child,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            check=False,
        )
        (directory / "configure.log").write_text(result.stdout, encoding="utf-8")
        print(result.stdout)
        result.check_returncode()
        if options.action != "build":
            return
        command = ["cmake", "--build", "--preset", options.preset, "--parallel", "4"]
        if options.target:
            command += ["--target", options.target]
    elif options.action == "clean":
        command = ["cmake", "--build", "--preset", options.preset, "--target", "clean"]
    elif options.action == "test":
        command = ["ctest", "--preset", options.preset, "--output-junit", str(directory / "junit.xml")]
        if options.exact:
            command += ["-R", "^" + re.escape(options.exact) + "$"]
        if options.verbose:
            command.append("--verbose")
        if options.repeat:
            command += ["--repeat", f"until-fail:{options.repeat}"]
    elif options.action == "consumer":
        prefix = directory / "installed"
        subprocess.run(["cmake", "--install", str(directory), "--prefix", str(prefix)], env=child, check=True)
        consumer = ROOT / "target/cp/consumer"
        subprocess.run(
            [
                "cmake",
                "-S",
                str(options.source or sdk / "starter"),
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
            env=child,
            check=True,
        )
        command = ["cmake", "--build", str(consumer), "--parallel", "4"]
    elif options.action == "conformance":
        if options.host is None:
            parser.error("--host Must Name the Built Reference Host")
        command = [
            str(options.host),
            "--plugin",
            str(directory / "tests" / ("dokime-conformance.exe" if os.name == "nt" else "dokime-conformance")),
        ]
    else:
        prefix = directory / "installed"
        subprocess.run(["cmake", "--install", str(directory), "--prefix", str(prefix)], env=child, check=True)
        with tarfile.open(directory / "dokime-cpp.tar.gz", "w:gz") as archive:
            archive.add(prefix, arcname="dokime-cpp")
        return
    log = directory / f"{options.action}.log"
    if log.exists():
        attempt = len(list(directory.glob(f"{options.action}-*.log"))) + 1
        shutil.copyfile(log, directory / f"{options.action}-{attempt}.log")
    with log.open("w", encoding="utf-8") as output:
        result = subprocess.run(command, cwd=sdk, env=child, stdout=output, stderr=subprocess.STDOUT, check=False)
    print(log.read_text(encoding="utf-8", errors="replace"))
    result.check_returncode()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
