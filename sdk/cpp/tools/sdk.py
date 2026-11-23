# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Focused C++ SDK Build, Proof and Native Package Tasks."""

# === Imports === #

# Standard Library Imports
import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tomllib
from pathlib import Path
from time import perf_counter

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.ci.native import visual_studio
from dokime_tools.process import spawn_flags
from dokime_tools.release.notices import native_notices

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


def environment(*, dependencies: bool = True) -> dict[str, str]:
    environment = dict(os.environ)
    # Resolve Native Build Tools Before vcpkg Enters Its Dependency Workspace.
    started = perf_counter()
    print("Prepare C++ Tools: Resolve CMake and Ninja", flush=True)
    tool_directories = []
    for tool in ("cmake", "ninja"):
        executable = subprocess.check_output(
            ["mise", "which", tool],
            cwd=ROOT / "sdk/cpp",
            text=True,
            stderr=sys.stderr,
            creationflags=spawn_flags(),
        ).strip()
        tool_directories.append(str(Path(executable).parent))
    print(f"Prepare C++ Tools: Completed After {perf_counter() - started:.2f}s", flush=True)

    # Ninja Uses the Native MSVC Environment on Windows.
    tool_path = environment["PATH"]
    if os.name == "nt" and not environment.get("VCINSTALLDIR"):
        started = perf_counter()
        print("Prepare C++ Compiler: Load the MSVC Developer Environment", flush=True)
        installer = Path(environment["PROGRAMFILES(X86)"]) / "Microsoft Visual Studio/Installer"
        developer = visual_studio() / "Common7/Tools/VsDevCmd.bat"
        # cmd.exe Cannot Inherit a PATH Longer Than Its Command-Line Limit.
        system = Path(environment["SYSTEMROOT"])
        developer_path = os.pathsep.join((str(installer), str(system / "System32"), str(system)))
        output = subprocess.check_output(
            f'cmd /d /s /c ""{developer}" -no_logo -arch=x64 -host_arch=x64 >nul && set"',
            text=True,
            env=dict(environment, PATH=developer_path),
            stderr=sys.stderr,
            creationflags=spawn_flags(),
        )
        environment.update(
            (key.upper(), value)
            for line in output.splitlines()
            if "=" in line and not line.startswith("=")
            for key, value in [line.split("=", 1)]
        )
        print(f"Prepare C++ Compiler: Completed After {perf_counter() - started:.2f}s", flush=True)

    if dependencies:
        # Locate the Pinned Dependency Checkout.
        started = perf_counter()
        print("Prepare C++ Dependencies: Locate the Mise Cache", flush=True)
        cache = subprocess.check_output(["mise", "cache", "path"], text=True, stderr=sys.stderr, creationflags=spawn_flags()).strip()
        manifest = json.loads((ROOT / "sdk/cpp/vcpkg.json").read_text())
        revision = manifest["builtin-baseline"]
        checkout = Path(cache) / "vcpkg" / revision[:12]

        # Bootstrap the Immutable Tool Checkout Outside the Repository.
        if not checkout.is_dir():
            checkout.parent.mkdir(parents=True, exist_ok=True)
            print("Prepare C++ Dependencies: Clone vcpkg", flush=True)
            subprocess.run(
                [
                    "git",
                    "clone",
                    "--filter=blob:none",
                    "--no-checkout",
                    "--depth",
                    "1",
                    "https://github.com/microsoft/vcpkg.git",
                    str(checkout),
                ],
                check=True,
                stdout=sys.stdout,
                stderr=sys.stderr,
                creationflags=spawn_flags(),
            )
            subprocess.run(
                ["git", "-C", str(checkout), "fetch", "--depth", "1", "origin", revision],
                check=True,
                stdout=sys.stdout,
                stderr=sys.stderr,
                creationflags=spawn_flags(),
            )
            subprocess.run(
                ["git", "-C", str(checkout), "checkout", "--detach", revision],
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
        print(f"Prepare C++ Dependencies: Completed After {perf_counter() - started:.2f}s; Checkout: {checkout}", flush=True)

        # Visual Studio Sets Its Bundled vcpkg Root; Restore the Verified Project Pin.
        environment["VCPKG_ROOT"] = str(checkout)
        if "CARGO_BUILD_JOBS" in environment:
            environment.setdefault("VCPKG_MAX_CONCURRENCY", environment["CARGO_BUILD_JOBS"])
    environment["PATH"] = os.pathsep.join(
        dict.fromkeys([*tool_directories, *environment["PATH"].split(os.pathsep), *tool_path.split(os.pathsep)])
    )
    return environment


def package(directory: Path, environment: dict[str, str]) -> Path:
    """Bundle the SDK Libraries of One Preset With Their Native Dependency Closure Into One Relocatable Archive."""
    cache = {}
    for line in (directory / "CMakeCache.txt").read_text(encoding="utf-8").splitlines():
        if "=" in line and not line.startswith(("//", "#")):
            key, value = line.split("=", 1)
            cache[key.split(":", 1)[0]] = value

    # A Dry Run Stops at the Library's Glob Re-Check, So a Current Build Proves Itself by Rewriting No Library.
    def libraries() -> dict[Path, tuple[int, int]]:
        return {path: (path.stat().st_size, path.stat().st_mtime_ns) for path in directory.rglob("*.lib" if os.name == "nt" else "*.a")}

    built = libraries()
    run(
        ["cmake", "--build", str(directory), "--target", "dokime_client"],
        action="Verify C++ SDK Build",
        cwd=ROOT / "sdk/cpp",
        env=environment,
        log=directory / "verify.log",
    )
    if not built or libraries() != built:
        raise RuntimeError("Package C++ SDK: Run mise -C sdk/cpp run build -- --preset " + directory.name + " Before Packaging")
    prefix = directory / "dokime-cpp"
    shutil.rmtree(prefix, ignore_errors=True)
    run(
        ["cmake", "--install", str(directory), "--prefix", str(prefix)],
        action="Install C++ SDK",
        cwd=ROOT / "sdk/cpp",
        env=environment,
        log=directory / "install.log",
    )

    # Bundle Native Libraries and Code Generators; Test-Enabled Builds Also Carry Catch2 for Test Consumers.
    triplet = cache["VCPKG_TARGET_TRIPLET"]
    installed = Path(cache["VCPKG_INSTALLED_DIR"])
    versions = {}
    depends = {}
    for paragraph in (installed / "vcpkg/status").read_text(encoding="utf-8").split("\n\n"):
        fields = dict(line.split(": ", 1) for line in paragraph.splitlines() if ": " in line)
        if fields.get("Architecture") != triplet or fields.get("Status") != "install ok installed":
            continue
        if "Feature" not in fields:
            versions[fields["Package"]] = fields["Version"]
        names = [name.split(":")[0].strip() for name in fields.get("Depends", "").split(",") if name.strip()]
        depends.setdefault(fields["Package"], set()).update(names)
    closure = set()
    pending = ["grpc", "protobuf", "openssl"]
    if cache.get("DOKIME_BUILD_TESTS", "").upper() in {"ON", "TRUE", "YES", "1"}:
        pending.append("catch2")
    while pending:
        name = pending.pop()
        if name not in closure and not name.startswith("vcpkg-"):
            closure.add(name)
            pending.extend(depends[name])
    notices = {"Harmos": directory / "_deps/harmos-src"}
    for name in sorted(closure):
        listing = installed / "vcpkg/info" / f"{name}_{versions[name]}_{triplet}.list"
        for line in listing.read_text(encoding="utf-8").splitlines():
            source = installed / line
            if line.endswith("/") or not source.is_file():
                continue
            destination = prefix / line.removeprefix(f"{triplet}/")
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        notices[f"{name} {versions[name]}"] = installed / triplet / "share" / name / "copyright"
    (prefix / "THIRD-PARTY-NOTICES").write_text(native_notices(notices), encoding="utf-8", newline="\n")
    shutil.copy2(ROOT / "LICENSE", prefix / "LICENSE")
    shutil.copy2(ROOT / "NOTICE", prefix / "NOTICE")

    # State the Toolchain and Runtime the Bundled Static Libraries Require.
    compiler = {}
    for line in next((directory / "CMakeFiles").glob("*/CMakeCXXCompiler.cmake")).read_text(encoding="utf-8").splitlines():
        for key in ("CMAKE_CXX_COMPILER_ID", "CMAKE_CXX_COMPILER_VERSION", "MSVC_TOOLSET_VERSION"):
            if line.startswith(f"set({key} "):
                compiler[key] = line.split(" ", 1)[1].rstrip(")").strip('"')
    target = "x86_64-pc-windows-msvc" if os.name == "nt" else "x86_64-unknown-linux-gnu"
    version = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]
    manifest = {
        "name": "dokime-cpp",
        "version": version,
        "target": target,
        "build_type": cache["CMAKE_BUILD_TYPE"],
        "cxx_standard": 20,
        "cmake_minimum": "4.2",
        "compiler": {"id": compiler["CMAKE_CXX_COMPILER_ID"], "version": compiler["CMAKE_CXX_COMPILER_VERSION"]},
        "runtime": "MultiThreadedDLL" if os.name == "nt" else "libstdc++ with the C++11 ABI",
        "libc": dict(zip(("name", "version"), platform.libc_ver(), strict=True)) if os.name != "nt" else None,
        "libraries": "static",
        "vcpkg_baseline": json.loads((ROOT / "sdk/cpp/vcpkg.json").read_text(encoding="utf-8"))["builtin-baseline"],
        "dependencies": {name: versions[name] for name in sorted(closure)},
    }
    if "MSVC_TOOLSET_VERSION" in compiler:
        manifest["compiler"]["toolset"] = compiler["MSVC_TOOLSET_VERSION"]
    elif os.name == "nt":
        manifest["compiler"]["toolset"] = environment["VCTOOLSVERSION"].rstrip("\\/")
    (prefix / "dokime-cpp.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")

    # Publish the Archive Beside Its Checksum File.
    archive = directory / f"dokime-cpp-{version}-{target}.tar.gz"
    output = tarfile.TarFile.gzopen(archive, "w")
    try:
        output.add(prefix, arcname="dokime-cpp")
    finally:
        output.close()
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(f"{archive.name}.sha256").write_text(f"{digest}  {archive.name}\n", encoding="utf-8", newline="\n")
    print(f"Package C++ SDK: {archive} ({archive.stat().st_size} Bytes, SHA-256 {digest})", flush=True)
    return archive


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "build", "test", "check", "format", "package", "consumer"))
    parser.add_argument("--preset", choices=("development", "tests", "release"))
    parser.add_argument("--sdk-package", type=Path, help="Build Only Consumers Against an Extracted SDK Package")
    parser.add_argument("--target")
    parser.add_argument("--exact")
    parser.add_argument("--repeat", type=int)
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--source", type=Path)
    options = parser.parse_args()
    options.preset = options.preset or ("release" if options.action in ("package", "consumer") else "tests")
    sdk = ROOT / "sdk/cpp"
    directory = ROOT / "target/cp" / options.preset
    directory.mkdir(parents=True, exist_ok=True)
    if options.sdk_package is not None and options.sdk_package.is_file():
        bundle = tarfile.open(options.sdk_package, "r:gz")
        try:
            bundle.extractall(directory / "sdk", filter="data")
        finally:
            bundle.close()
        options.sdk_package = directory / "sdk/dokime-cpp"
    if options.sdk_package is not None and not (options.sdk_package / "dokime-cpp.json").is_file():
        raise RuntimeError("Build C++ Package Consumer: Select an Extracted SDK or Its Published Archive With --sdk-package")
    child = (
        dict(os.environ)
        if options.action == "format"
        else environment(dependencies=options.action not in ("consumer", "test", "package") and options.sdk_package is None)
    )
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
    elif options.action == "package":
        package(directory, child)
        return
    elif options.action in ("check", "build"):
        if options.action == "build":
            command = ["cmake", "--build", "--preset", options.preset, "--parallel"]
            if "CARGO_BUILD_JOBS" in child:
                command.append(child["CARGO_BUILD_JOBS"])
            if options.target:
                command += ["--target", options.target]
            print(f"Build C++ SDK After Configure: {subprocess.list2cmdline(command)}", flush=True)
        configure = ["cmake", "--preset", options.preset, f"-DPython3_EXECUTABLE={sys.executable}"]
        if options.sdk_package is not None:
            bundle = options.sdk_package.resolve()
            build_type = "Release" if options.preset == "release" else "Debug"
            configure = [
                "cmake",
                "--fresh",
                "-S",
                str(sdk),
                "-B",
                str(directory),
                "-G",
                "Ninja",
                "-DDOKIME_BUILD_TESTS=ON",
                f"-DDOKIME_SDK_PACKAGE={bundle}",
                f"-DCMAKE_BUILD_TYPE={build_type}",
                f"-DPython3_EXECUTABLE={sys.executable}",
            ]
        cache = directory / "CMakeCache.txt"
        if cache.is_file() and options.sdk_package is None:
            configured = cache.read_text(encoding="utf-8").replace("\\", "/").splitlines()
            toolchain = f"Z_VCPKG_ROOT_DIR:INTERNAL={Path(child['VCPKG_ROOT']).as_posix()}"
            if toolchain not in configured:
                configure.insert(1, "--fresh")
        run(
            configure,
            action="Configure C++ SDK",
            cwd=sdk,
            env=child,
            log=directory / "configure.log",
        )
        if options.action != "build":
            return
    elif options.action == "test":
        command = ["ctest", "--preset", options.preset, "--output-junit", str(directory / "junit.xml")]
        if options.exact:
            command += ["-R", "^" + re.escape(options.exact) + "$"]
        if options.verbose:
            command.append("--verbose")
        if options.repeat:
            command += ["--repeat", f"until-fail:{options.repeat}"]
    elif options.action == "consumer":
        # Build the Source Against the Bundled Package Alone, Without the Package Manager.
        bundle = options.sdk_package or directory / "dokime-cpp"
        if not (bundle / "dokime-cpp.json").is_file():
            raise RuntimeError(f"Build C++ Package Consumer: Run mise -C sdk/cpp run package -- --preset {options.preset} First")
        source = (options.source or sdk / "examples/starter").resolve()
        consumer = directory / "consumers" / source.name
        build_type = json.loads((bundle / "dokime-cpp.json").read_text(encoding="utf-8"))["build_type"]
        run(
            [
                "cmake",
                "--fresh",
                "-S",
                str(source),
                "-B",
                str(consumer),
                "-G",
                "Ninja",
                f"-DCMAKE_PREFIX_PATH={bundle}",
                f"-DCMAKE_BUILD_TYPE={build_type}",
            ],
            cwd=source,
            env=child,
            action="Configure C++ Package Consumer",
            log=directory / "consumer-configure.log",
        )
        command = ["cmake", "--build", str(consumer), "--parallel"]
        if "CARGO_BUILD_JOBS" in child:
            command.append(child["CARGO_BUILD_JOBS"])
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
