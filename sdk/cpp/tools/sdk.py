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
import hashlib
import os
import shutil
import subprocess
import tarfile
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "action",
        choices=[
            "configure",
            "build",
            "test",
            "conformance",
            "starter",
            "archive",
            "check",
            "format",
        ],
    )
    parser.add_argument("--build-dir", type=Path)
    parser.add_argument("--harmos-source", type=Path, default=os.environ.get("HARMOS_SOURCE"))
    parser.add_argument("--harmos-cache", type=Path, default=os.environ.get("DOKIME_HARMOS_CACHE"))
    parser.add_argument("--fetch-dependencies", action="store_true")
    parser.add_argument("--fresh", action="store_true", help="Refresh the CMake Configuration While Preserving Compiled Outputs")
    parser.add_argument("--target")
    parser.add_argument("--host", type=Path)
    parser.add_argument("--version", default="0.1.0")
    parser.add_argument("--sdk-archive", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--source", type=Path, help="Format One Generated Source")
    args = parser.parse_args()
    sdk = Path(__file__).resolve().parents[1]
    root = sdk.parents[1]
    directory = (args.build_dir or root / "target/sdk/cpp").resolve()
    directory.mkdir(parents=True, exist_ok=True)
    configuration = "Debug"
    environment = dict(os.environ)
    creationflags = subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0

    if args.action == "format":
        formatter = shutil.which("clang-format")
        if not formatter and os.name == "nt":
            installer = Path(os.environ["ProgramFiles(x86)"]) / "Microsoft Visual Studio/Installer/vswhere.exe"
            installation = subprocess.check_output(
                [
                    str(installer),
                    "-latest",
                    "-products",
                    "*",
                    "-property",
                    "installationPath",
                ],
                text=True,
                creationflags=creationflags,
            ).strip()
            formatter = str(Path(installation) / "VC/Tools/Llvm/bin/clang-format.exe")
        if not formatter:
            parser.error("Install clang-format and Put It on PATH")
        sources = [args.source] if args.source else sorted([*sdk.rglob("*.cpp"), *sdk.rglob("*.hpp")])
        command = [formatter, "-style={BasedOnStyle: LLVM, ColumnLimit: 140}"]
        command += ["--dry-run", "--Werror"] if args.check else ["-i"]
        command += [str(source) for source in sources]
    elif args.action == "configure":
        command = [
            "cmake",
            "-S",
            str(sdk),
            "-B",
            str(directory),
            "-DCMAKE_BUILD_TYPE=Debug",
            "-DDOKIME_BUILD_CONFORMANCE=ON",
        ]
        if args.fresh:
            command += ["--fresh"]
        if os.name == "nt":
            command += ["-G", "Visual Studio 17 2022", "-A", "x64"]
        if args.harmos_source:
            command += [f"-DFETCHCONTENT_SOURCE_DIR_HARMOS={args.harmos_source.resolve()}"]
        if args.harmos_cache:
            command += [f"-DDOKIME_HARMOS_CACHE={args.harmos_cache.resolve()}"]
        if args.fetch_dependencies:
            command += ["-DDOKIME_FETCH_DEPENDENCIES=ON"]
    elif args.action == "build":
        command = [
            "cmake",
            "--build",
            str(directory),
            "--config",
            configuration,
            "--parallel",
            "2",
        ]
        if args.target:
            command += ["--target", args.target]
    elif args.action == "test":
        command = [
            "ctest",
            "--test-dir",
            str(directory),
            "-C",
            configuration,
            "--output-on-failure",
            "--output-junit",
            str(directory / "junit.xml"),
        ]
    elif args.action == "conformance":
        if not args.host:
            parser.error("--host Must Name the Built Harmos tools/conformance Executable")
        executable = (
            directory / "tests" / configuration / "dokime-conformance.exe" if os.name == "nt" else directory / "tests/dokime-conformance"
        )
        command = [str(args.host.resolve()), "--plugin", str(executable)]
    elif args.action == "check":
        presets = subprocess.run(
            ["cmake", "--list-presets", "-S", str(sdk / "starter")], capture_output=True, text=True, creationflags=creationflags
        )
        print(presets.stdout + presets.stderr)
        presets.check_returncode()
        command = ["cmake", "-S", str(sdk), "-B", str(directory)]
    elif args.action == "starter":
        command = [
            "cmake",
            "-S",
            str(sdk / "starter"),
            "-B",
            str(directory / "starter"),
            "-DDOKIME_BUILD_TESTS=OFF",
            "-DCMAKE_BUILD_TYPE=Debug",
        ]
        if args.sdk_archive:
            archive = args.sdk_archive.resolve()
            with archive.open("rb") as source:
                digest = hashlib.file_digest(source, "sha256").hexdigest()
            command += [
                f"-DDOKIME_SDK_URL={archive.as_posix()}",
                f"-DDOKIME_SDK_SHA256={digest}",
            ]
        else:
            command += [f"-DFETCHCONTENT_SOURCE_DIR_DOKIME={root}"]
        if os.name == "nt":
            command += ["-G", "Visual Studio 17 2022", "-A", "x64"]
        if args.harmos_source:
            command += [f"-DFETCHCONTENT_SOURCE_DIR_HARMOS={args.harmos_source.resolve()}"]
        if args.harmos_cache:
            command += [f"-DDOKIME_HARMOS_CACHE={args.harmos_cache.resolve()}"]
        configure_log = directory / "starter-configure.log"
        with configure_log.open("w", encoding="utf-8") as output:
            result = subprocess.run(command, env=environment, stdout=output, stderr=subprocess.STDOUT, creationflags=creationflags)
        print("\n".join(configure_log.read_text(encoding="utf-8", errors="replace").splitlines()[-60:]))
        result.check_returncode()
        command = [
            "cmake",
            "--build",
            str(directory / "starter"),
            "--config",
            configuration,
            "--parallel",
            "2",
        ]
    else:
        archive = directory / f"dokime-cpp-{args.version}.tar.gz"
        tracked = subprocess.check_output(
            [
                "git",
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "--",
                "sdk/cpp",
                "proto",
                "docs/sdk-cpp.md",
                "LICENSE",
                "NOTICE",
            ],
            cwd=root,
            text=True,
        ).splitlines()
        with tarfile.open(archive, "w:gz") as output:
            for relative in tracked:
                source = root / relative
                if source.is_file():
                    output.add(source, arcname=f"dokime-cpp-{args.version}/{relative}")
        with archive.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        archive.with_suffix(archive.suffix + ".sha256").write_text(f"{digest}  {archive.name}\n", encoding="utf-8")
        print(f"{archive}\nSHA256={digest}")
        return

    # Preserve Complete Tool Output and Show a Bounded Diagnostic Tail.
    log = directory / f"{args.action}.log"
    if log.exists():
        attempt = len(list(directory.glob(f"{args.action}-*.log"))) + 1
        shutil.copyfile(log, directory / f"{args.action}-{attempt}.log")
    with log.open("w", encoding="utf-8") as output:
        result = subprocess.run(command, env=environment, stdout=output, stderr=subprocess.STDOUT, creationflags=creationflags)
    lines = log.read_text(encoding="utf-8", errors="replace").splitlines()
    print("\n".join(lines[-60:]))
    print(f"Complete Output: {log}")
    result.check_returncode()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
