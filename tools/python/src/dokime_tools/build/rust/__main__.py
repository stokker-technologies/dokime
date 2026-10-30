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
# Local Imports
from dokime_tools.build import cargo, configuration, frontend, outputs, paths
from dokime_tools.package import plugins
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = configuration.ROOT
EXECUTABLES = ("dokime", "cargo-dokime")

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Build Shipping Native Bytes Before Compiling The Frontend-Embedding Desktop."""
    output = paths.output(ROOT, "build", profile)
    messages = output / "product/compiler.json"
    messages.parent.mkdir(parents=True, exist_ok=True)
    arguments = ["--locked", "--workspace", "--profile", profile, *configuration.features(profile)]
    # The desktop embeds the frontend at compile time. Building it here against a placeholder keeps one
    # feature resolution for both steps, so the desktop step recompiles only dokime-gui.
    dist = frontend.WEB_ROOT / "dist"
    if not dist.is_dir():
        dist.mkdir(parents=True)
        (dist / "index.html").write_text("", encoding="utf-8")
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


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
