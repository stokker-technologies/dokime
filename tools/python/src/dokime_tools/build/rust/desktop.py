# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Desktop Build Task"""

# === Imports === #

# Standard Library Imports
import subprocess
import sys

# Third Party Imports
# Local Imports
from dokime_tools.build import cargo, configuration, frontend, outputs, paths
from dokime_tools.package import payload
from dokime_tools.package.staging import stage_file, sync_output

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = configuration.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Embed The Already-Built Frontend In The Shipping Desktop Executable."""
    output = paths.output(ROOT, "build", profile)
    if not (output / "web/index.html").is_file():
        raise ValueError("build the matching frontend before compiling the desktop")
    sync_output(output / "web", frontend.WEB_ROOT / "dist")
    messages = output / "product/desktop-compiler.json"
    arguments = [
        "--locked",
        "--package",
        "dokime-gui",
        "--profile",
        profile,
    ]
    arguments += configuration.features(profile)
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
    source = outputs.executables(messages, ("dokime-gui",))["dokime-gui"]
    destination = output / "bin" / source.name
    stage_file(source, destination)
    cargo.record_target(ROOT, messages, profile)
    payload.export(ROOT, profile)
    return configuration.relative([destination])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
