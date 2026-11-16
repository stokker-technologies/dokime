# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Python Build Task"""

# === Imports === #

# Standard Library Imports

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import configuration, wheel

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Build the Pure Python SDK Wheel."""
    distributions = wheel.build(configuration.ROOT, profile)
    return configuration.relative(distributions)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
