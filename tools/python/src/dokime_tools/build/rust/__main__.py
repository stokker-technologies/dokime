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
import subprocess
import sys

# Third Party Imports
from harmos_tools.build import paths

# Local Imports
from dokime_tools.build import cargo, configuration, outputs
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = configuration.ROOT
EXECUTABLES = ("dokime-next", "dokime-host-next", "dokime-gui")

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Build Shipping Native Host And Command-Line Bytes."""
    output = paths.output(ROOT, "build", profile)
    messages = output / "product/compiler.json"
    messages.parent.mkdir(parents=True, exist_ok=True)
    arguments = ["--locked", "--workspace", "--profile", profile, "--features", "dokime-gui/custom-protocol"]
    command = ["cargo", "build", *arguments, "--message-format=json-render-diagnostics"]
    pending = messages.with_name("compiler.pending.json")
    with pending.open("w", encoding="utf-8") as written:
        subprocess.run(
            command,
            cwd=ROOT,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    pending.replace(messages)
    binaries = output / "bin"
    clear_output(binaries)
    staged = []
    for path in outputs.executables(messages, EXECUTABLES).values():
        stage_file(path, binaries / path.name)
        staged.append(binaries / path.name)
    cargo.record_target(ROOT, messages, profile)
    return configuration.relative(staged)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
