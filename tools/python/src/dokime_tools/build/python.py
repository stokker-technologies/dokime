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
from dokime_tools.build import configuration, paths, wheel
from dokime_tools.package import examples

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = configuration.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Build Tracked Client Distributions And Their Exact Portable Tutorial."""
    distributions = wheel.build(ROOT, profile)
    wheels = [path for path in distributions if path.suffix == ".whl"]
    if len(wheels) != 1:
        raise ValueError("the tutorial requires exactly one matching built wheel")
    destination = paths.output(ROOT, "build", profile, "any") / "python/examples/remote"
    staged = examples.stage(ROOT, destination, wheel=wheels[0])
    return configuration.relative([*distributions, *staged])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
