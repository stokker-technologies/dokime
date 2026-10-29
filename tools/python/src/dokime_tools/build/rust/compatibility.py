# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compatibility Build Task"""

from dokime_tools.build import cargo, configuration

ROOT = configuration.ROOT


def build(profile: str) -> list[str]:
    """Verify The Complete Feature Matrix For Explicit Local Extended Validation."""
    for selected in (
        ["--workspace", "--all-targets", "--all-features"],
        ["--workspace", "--lib", "--bins", "--no-default-features"],
        ["--package", "dokime-domain", "--target", "wasm32-unknown-unknown"],
    ):
        cargo.clippy(ROOT, ["--locked", "--profile", "test", *selected])
    return []


if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
