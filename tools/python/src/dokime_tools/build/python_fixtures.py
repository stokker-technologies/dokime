# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Only The Native Inputs For Python Client Acceptance."""

# === Imports === #

# Standard Library Imports
import subprocess
import sys

# Third Party Imports
from harmos_tools.build import paths
from harmos_tools.process import spawn_flags

# Local Imports
from dokime_tools.build import configuration, outputs, storage
from dokime_tools.package import plugins
from dokime_tools.package.staging import clear_output, stage_file

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = configuration.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Build The CLI And Plugin Fixtures Without Desktop Or Test Harnesses."""
    output = paths.output(ROOT, "build", profile) / "fixtures/python-client"
    clear_output(output)
    output.mkdir(parents=True, exist_ok=True)
    messages = output / "compiler.json"
    selectors = ["--locked", "--profile", profile, "--bins", "--features", "plugin-host/runtime"]
    for package in ("dokime-cli", "cargo-dokime", "plugin-host", "plugin-obc"):
        selectors.extend(("-p", package))
    storage.preflight(ROOT)
    with messages.open("w", encoding="utf-8") as written:
        subprocess.run(
            ["cargo", "build", *selectors, "--message-format=json-render-diagnostics"],
            cwd=ROOT,
            check=True,
            creationflags=spawn_flags(),
            stdout=written,
            stderr=sys.stderr,
        )
    binaries = outputs.executables(messages, ("dokime", "cargo-dokime"))
    staged = [messages]
    for binary in binaries.values():
        destination = output / "bin" / binary.name
        stage_file(binary, destination)
        staged.append(destination)
    staged += plugins.prepare(ROOT, messages, profile, tool=binaries["cargo-dokime"], output=output / "plugins", packages=("plugin-host",))
    staged += plugins.prepare(
        ROOT, messages, profile, tool=binaries["cargo-dokime"], output=output / "fixtures/plugins", packages=("plugin-obc",)
    )
    return configuration.relative(staged)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
