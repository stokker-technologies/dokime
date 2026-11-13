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
# Third Party Imports
# Local Imports
from dokime_tools.build import configuration
from dokime_tools.prepare import generated

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
    files = [path for path in (configuration.ROOT / "target/generated/typescript").rglob("*") if path.is_file()]
    return configuration.relative(files)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
