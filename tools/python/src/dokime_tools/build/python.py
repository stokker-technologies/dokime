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

# Third Party Imports
# Local Imports
from dokime_tools.build import configuration, wheel

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = configuration.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Build The Client Distributions Without Tutorial Artifacts."""
    distributions = wheel.build(ROOT, profile)
    return configuration.relative(distributions)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
