# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Rust Build Task"""

import subprocess
import sys

from dokime_tools.build import cargo, configuration, outputs, paths
from dokime_tools.package import plugins
from dokime_tools.package.staging import clear_output, stage_file

ROOT = configuration.ROOT
EXECUTABLES = ("dokime", "cargo-dokime")


def build(profile: str) -> list[str]:
    """Build Shipping Native Bytes Before Compiling The Frontend-Embedding Desktop."""
    output = paths.output(ROOT, "build", profile)
    messages = output / "product/compiler.json"
    messages.parent.mkdir(parents=True, exist_ok=True)
    arguments = [
        "--locked",
        "--workspace",
        "--exclude",
        "dokime-gui",
        "--profile",
        profile,
    ]
    arguments += configuration.features(profile, desktop=False)
    cargo.clippy(ROOT, arguments)
    command = ["cargo", "build", *arguments, "--message-format=json-render-diagnostics"]
    with messages.open("w", encoding="utf-8") as written:
        subprocess.run(
            command,
            cwd=ROOT,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    binaries = output / "bin"
    clear_output(binaries)
    staged = []
    for path in outputs.executables(messages, EXECUTABLES).values():
        stage_file(path, binaries / path.name)
        staged.append(binaries / path.name)
    tool = binaries / f"cargo-dokime{'.exe' if sys.platform == 'win32' else ''}"
    staged += plugins.prepare(ROOT, messages, profile, tool=tool, output=output / "plugins")
    cargo.record_target(ROOT, messages, profile)
    return configuration.relative(staged)


if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
