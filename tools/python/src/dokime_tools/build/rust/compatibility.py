# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compatibility Build Task"""

# === Imports === #

# Standard Library Imports

# Third Party Imports
# Local Imports
from dokime_tools.build import cargo, configuration

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = configuration.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Verify The Complete Feature Matrix For Explicit Local Extended Validation."""
    compiler_profile = "release-tests" if profile == "release" else profile
    for selected in (
        ["--workspace", "--all-targets", "--all-features"],
        ["--workspace", "--lib", "--bins", "--no-default-features"],
        ["--package", "dokime-domain", "--target", "wasm32-unknown-unknown"],
    ):
        cargo.clippy(ROOT, ["--locked", "--profile", compiler_profile, *selected])
    return []


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
