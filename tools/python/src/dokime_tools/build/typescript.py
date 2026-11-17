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

# isort: split

# Third Party Imports

# isort: split

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
    """Generate the GUI Modules From the Product Build and Build the Live Frontend."""
    generated.prepare()
    generated.stage(profile)
    organization.generate_icons()
    subprocess.run(["pnpm", "run", "typecheck"], cwd=frontend.WEB_ROOT, check=True, creationflags=spawn_flags())
    web = frontend.build(profile)
    files = []
    for files_directory in (
        configuration.ROOT / "target/generated/typescript",
        configuration.ROOT / "target/generated/host",
        configuration.ROOT / "target/generated/report",
        web,
    ):
        for files_path in files_directory.rglob("*"):
            if not (files_path.is_file()):
                continue
            files.append(files_path)

    return configuration.relative(files)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
