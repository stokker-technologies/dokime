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

# Third Party Imports
# Local Imports
from dokime_tools.build import configuration, frontend, organization
from dokime_tools.prepare import generated
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Generate The GUI Modules From The Product Build And Build The Live Frontend."""
    generated.prepare()
    generated.stage(profile)
    organization.generate_icons()
    subprocess.run(["pnpm", "run", "typecheck"], cwd=frontend.WEB_ROOT, check=True, creationflags=spawn_flags())
    web = frontend.build(profile)
    frontend.build_report([])
    files = [
        path
        for directory in (configuration.ROOT / "target/generated/typescript", configuration.ROOT / "target/generated/host", web)
        for path in directory.rglob("*")
        if path.is_file()
    ]
    return configuration.relative(files)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
