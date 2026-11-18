# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Rust Build Task"""

# === Imports === #

# Standard Library Imports
import os
import subprocess
import sys

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import cargo, configuration, outputs, paths
from dokime_tools.package.staging import clear_output, stage_file

from ...process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #


EXECUTABLES = ("dokime", "dokime-host", "dokime-gui", "plugin-obc")

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Build Shipping Native Host and Command-Line Bytes."""
    output = paths.output(configuration.ROOT, "build", profile)
    messages = output / "product/compiler.json"
    messages.parent.mkdir(parents=True, exist_ok=True)
    arguments = ["--locked", "--workspace", "--profile", profile, "--features", "dokime-gui/custom-protocol"]
    command = ["cargo", "build", *arguments, "--message-format=json-render-diagnostics"]
    pending = messages.with_name("compiler.pending.json")
    written = pending.open("w", encoding="utf-8")
    try:
        subprocess.run(
            command,
            cwd=configuration.ROOT,
            stdout=written,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    finally:
        written.close()
    pending.replace(messages)
    binaries = output / "bin"
    clear_output(binaries)
    staged = []
    for path in outputs.executables(messages, EXECUTABLES).values():
        stage_file(path, binaries / path.name)
        staged.append(binaries / path.name)

    # Bundle the Shipped Plugin With the Command-Line Tool Built Beside It
    extension = ".exe" if os.name == "nt" else ""
    project = output / "product/plugin-obc"
    project.mkdir(parents=True, exist_ok=True)
    stage_file(binaries / f"plugin-obc{extension}", project / f"plugin-obc{extension}")
    (project / "dokime.toml").write_text('executable = "plugin-obc"\n', encoding="utf-8")
    bundle = binaries / "plugin-obc.dokime"
    subprocess.run(
        [
            str(binaries / f"dokime{extension}"),
            "plugin",
            "bundle",
            str(project),
            "--out",
            str(bundle),
        ],
        cwd=configuration.ROOT,
        check=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stderr,
        stderr=sys.stderr,
    )
    staged.append(bundle)
    cargo.record_target(configuration.ROOT, messages, profile)
    return configuration.relative([*staged, messages, output / "product/target.txt"])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
