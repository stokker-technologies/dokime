# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Typescript Build Task"""

# === Imports === #

# Standard Library Imports
import subprocess
import sys

# Third Party Imports
# Local Imports
from dokime_tools.build import configuration, frontend
from dokime_tools.prepare import bindings, generated

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = configuration.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Generate The GUI Message Modules From The Server Export And Build The Live Frontend."""
    messages = configuration.messages(profile)
    bindings.generate(messages, check=True)
    subprocess.run(
        ["node", str(frontend.WEB_ROOT / "tools/bindings.mjs"), str(messages), "--check"],
        cwd=ROOT,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    generated.stage(profile)
    built = frontend.build(profile)
    return configuration.relative([path for path in built.rglob("*") if path.is_file()])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
