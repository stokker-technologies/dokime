# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Build Benchmark Executables and Matching Native Hosts"

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import shutil
import subprocess
import sys
from collections.abc import Sequence
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import temporary
from ..prepare import generated
from ..process import spawn_flags
from ..test import archive
from . import cargo, paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(
    directory: Path,
    extra: Sequence[str] = (),
    selection: Sequence[str] = ("--workspace", "--bins", "--benches"),
    *,
    require_benchmark: bool = True,
    environment: dict[str, str] | None = None,
    compiler_profile: str = "bench",
) -> Path:
    output = paths.output(directory, "build", "bench")
    messages = output / "bench/compiler.json"
    messages.parent.mkdir(parents=True, exist_ok=True)
    messages.unlink(missing_ok=True)

    command = [
        "cargo",
        "build",
        "--locked",
        *selection,
        "--profile",
        compiler_profile,
        "--message-format=json-render-diagnostics",
        *extra,
    ]
    staging_directory = temporary.Directory()
    staging = staging_directory.path
    try:
        raw = staging / "compiler.json"
        written = raw.open("w", encoding="utf-8")
        try:
            subprocess.run(
                command,
                env=environment,
                stdin=sys.stdin,
                stderr=sys.stderr,
                creationflags=spawn_flags(),
                cwd=directory,
                stdout=written,
                check=True,
            )
        finally:
            written.close()
        for kind in ("bin", "bench"):
            (staging / kind).mkdir()

        events = []
        files = []
        benchmarks = False
        for line in raw.read_text(encoding="utf-8").splitlines():
            event = json.loads(line)
            executable = event.get("executable")
            kinds = event.get("target", {}).get("kind", [])
            kind = (
                "bench"
                if "bench" in kinds
                else "bin"
                if kinds in (["bin"], ["example"]) and not event.get("profile", {}).get("test")
                else None
            )
            if executable and kind:
                benchmarks = benchmarks or kind == "bench"
                source = Path(executable)
                destination = output / kind / source.name
                shutil.copy2(source, staging / kind / source.name)
                event["executable"] = destination.relative_to(directory).as_posix()
                event["manifest_path"] = Path(event["manifest_path"]).relative_to(directory).as_posix()
                files.append(event["executable"])
            events.append(json.dumps(event))

        if require_benchmark and not benchmarks:
            raise ValueError("Build Bench: no benchmark target was built")
        for kind in ("bin", "bench"):
            paths.replace(directory, staging / kind, output / kind, origin=staging)
        messages.write_text("\n".join(events) + "\n", encoding="utf-8")
        files.append(messages.relative_to(directory).as_posix())
        archive.stamp(directory, "bench", files)
    finally:
        staging_directory.close()
    return messages


def main() -> None:
    parser = argparse.ArgumentParser(description="Build Benchmarks and Matching Hosts in the Bench Profile")
    parser.add_argument("--binary")
    parser.add_argument("--compiler-profile", choices=("dev", "bench"), default="bench")
    parser.add_argument("extra", nargs=argparse.REMAINDER)
    options = parser.parse_args()
    extra = options.extra[1:] if options.extra[:1] == ["--"] else options.extra
    if options.compiler_profile != "bench" and options.binary not in {"system-sdk", "protocol"}:
        parser.error("--compiler-profile dev is supported for system-sdk and protocol")
    if options.binary == "protocol":
        generated.prepare()
        directory = Path.cwd()
        print(
            build(
                directory,
                extra,
                ("--package", "dokime-sdk-benches", "--bench", "protocol"),
                compiler_profile=options.compiler_profile,
                environment=cargo.compiler_environment(options.compiler_profile),
            )
        )
        return
    if options.binary == "system-sdk":
        generated.prepare()
        directory = Path.cwd()
        messages = build(
            directory,
            extra,
            ("--package", "dokime-sdk-benches", "--package", "dokime", "--bench", "system-sdk", "--example", "thermal"),
            compiler_profile=options.compiler_profile,
            environment=dict(cargo.compiler_environment(options.compiler_profile), CARGO_TARGET_DIR=str(directory / "target")),
        )
        output = paths.output(directory, "build", "bench")
        fixtures = output / "fixtures/host"
        fixtures.mkdir(parents=True, exist_ok=True)
        extension = ".exe" if os.name == "nt" else ""
        fixture = f"dokime-rust-thermal{extension}"
        shutil.copy2(output / "bin" / ("thermal" + extension), fixtures / fixture)
        stamp = json.loads((output / "fixtures/stamp.json").read_text(encoding="utf-8"))
        archive.stamp(directory, "bench", [*stamp["files"], (fixtures / fixture).relative_to(directory).as_posix()], stamp["metadata"])
        print(messages)
        return
    if options.binary == "storage":
        generated.prepare()
        print(build(Path.cwd(), extra, ("--package", "dokime-runtime", "--bench", "storage")))
        return
    subprocess.run(["mise", "run", "build:typescript", "--profile", "dev"], check=True, creationflags=spawn_flags())
    selection = (
        (
            "--package",
            "dokime-gui",
            "--package",
            "dokime-runtime",
            "--package",
            "dokime",
            "--package",
            "dokime-sdk-benches",
            "--bin",
            "dokime-gui",
            "--bin",
            "dokime-host",
            "--bench",
            options.binary,
            "--bench",
            "protocol",
            "--bench",
            "system-sdk",
            "--example",
            "thermal",
        )
        if options.binary
        else ("--workspace", "--bins", "--benches", "--example", "thermal")
    )
    directory = Path.cwd()
    output = paths.output(directory, "build", "bench")
    environment = dict(os.environ)
    if options.binary in (None, "system-desktop"):
        # Build and Embed the Instrumented Surface in the Matching Native Shell.
        web = output / "web"
        subprocess.run(
            ["node", "benches/system/build.ts", str(web)],
            cwd=directory / "crates/dokime-gui",
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        config = json.loads(environment.get("TAURI_CONFIG", "{}"))
        config.setdefault("build", {})["frontendDist"] = Path(os.path.relpath(web, directory / "crates/dokime-gui/src-tauri")).as_posix()
        environment["TAURI_CONFIG"] = json.dumps(config)
    messages = build(directory, ["--features", "dokime-gui/custom-protocol", *extra], selection, environment=environment)
    generated.stage("bench")
    stamp = json.loads((output / "fixtures/stamp.json").read_text(encoding="utf-8"))
    modules = []
    for modules_path in (output / "generated").rglob("*"):
        if not (modules_path.is_file()):
            continue
        modules.append(modules_path.relative_to(directory).as_posix())

    fixtures = output / "fixtures/host"
    fixtures.mkdir(parents=True, exist_ok=True)
    extension = ".exe" if os.name == "nt" else ""
    fixture = "dokime-rust-thermal" + extension
    shutil.copy2(output / "bin" / ("thermal" + extension), fixtures / fixture)
    archive.stamp(
        directory, "bench", [*stamp["files"], *modules, (fixtures / fixture).relative_to(directory).as_posix()], stamp["metadata"]
    )
    print(messages)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
