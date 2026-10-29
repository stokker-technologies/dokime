# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Python Build Task"""

from dokime_tools.build import configuration, wheel

ROOT = configuration.ROOT


def build(profile: str) -> list[str]:
    """Build The Client Distributions Without Tutorial Artifacts."""
    distributions = wheel.build(ROOT, profile)
    return configuration.relative(distributions)


if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
