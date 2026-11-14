# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Benchmark Executables And Matching Native Hosts"""

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

# Third Party Imports
# Local Imports
from .. import temporary
from ..prepare import generated
from ..process import spawn_flags
from ..test import archive
from . import paths

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
        "bench",
        "--message-format=json-render-diagnostics",
        *extra,
    ]
    with temporary.directory() as staging:
        raw = staging / "compiler.json"
        with raw.open("w", encoding="utf-8") as written:
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
        for kind in ("bin", "bench"):
            (staging / kind).mkdir()

        events = []
        files = []
        benchmarks = False
        for line in raw.read_text(encoding="utf-8").splitlines():
            event = json.loads(line)
            executable = event.get("executable")
            kinds = event.get("target", {}).get("kind", [])
            kind = "bench" if "bench" in kinds else "bin" if kinds == ["bin"] and not event.get("profile", {}).get("test") else None
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
            raise ValueError("no benchmark target was built")
        for kind in ("bin", "bench"):
            paths.replace(directory, staging / kind, output / kind, origin=staging)
        messages.write_text("\n".join(events) + "\n", encoding="utf-8")
        files.append(messages.relative_to(directory).as_posix())
        archive.stamp(directory, "bench", files)
    return messages


def main() -> None:
    parser = argparse.ArgumentParser(description="Build Benchmarks And Matching Hosts In The Bench Profile")
    parser.add_argument("--binary")
    parser.add_argument("extra", nargs=argparse.REMAINDER)
    options = parser.parse_args()
    extra = options.extra[1:] if options.extra[:1] == ["--"] else options.extra
    if options.binary == "storage":
        print(build(Path.cwd(), extra, ("--package", "dokime-runtime-next", "--bench", "storage")))
        return
    subprocess.run(["mise", "run", "build:typescript", "--profile", "dev"], check=True, creationflags=spawn_flags())
    selection = (
        (
            "--package",
            "dokime-gui",
            "--package",
            "dokime-runtime-next",
            "--bin",
            "dokime-gui",
            "--bin",
            "dokime-host-next",
            "--bench",
            options.binary,
        )
        if options.binary
        else ("--workspace", "--bins", "--benches")
    )
    directory = Path.cwd()
    output = paths.output(directory, "build", "bench")
    environment = dict(os.environ)
    if options.binary in (None, "system-desktop"):
        # Build And Embed The Instrumented Surface In The Matching Native Shell.
        web = output / "web"
        subprocess.run(
            ["node", "benches/system/build.mjs", str(web)],
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
    modules = [path.relative_to(directory).as_posix() for path in (output / "generated").rglob("*") if path.is_file()]
    archive.stamp(directory, "bench", [*stamp["files"], *modules], stamp["metadata"])
    print(messages)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
