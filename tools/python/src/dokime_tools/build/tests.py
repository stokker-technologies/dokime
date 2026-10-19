# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build The Shared Test Archive And Its Fixtures Without Replacing Product Exports"""

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
import sys

# Third Party Imports
# Local Imports
from dokime_tools.build import dependencies, frontend, paths, product
from dokime_tools.package.staging import sync_output
from dokime_tools.test import archive, policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> None:
    """Consume Matching Product Hosts And Compile The Shared Test Archive."""
    if profile not in product.PROFILES:
        raise ValueError("test archives require dev or release")
    output = paths.output(ROOT, "build", profile)
    fixtures = output / "fixtures"
    staged = product.files(profile)
    sync_output(output / "web", frontend.WEB_ROOT / "dist")
    features = "tauri/custom-protocol,dokime-runtime/fault-injection"

    # Compile Test Targets Through The Shared Archive Owner.
    archive.build(ROOT, profile, ["--features", features])
    directory = fixtures / "dependencies"
    staged += dependencies.cargo(ROOT, directory)
    staged += dependencies.wheelhouse(ROOT, directory)

    root = ROOT.resolve()
    resolved = [path.resolve() for path in staged]
    relative = {path.relative_to(root) for path in resolved}
    names = [path.as_posix() for path in relative]
    names.sort()
    archive.stamp(ROOT, profile, names)


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=product.PROFILES, default="dev")
    options = parser.parse_args(arguments)
    try:
        build(options.profile)
    except (policy.TestError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build tests: {error}", file=sys.stderr)
        return 1
    return 0


# === Private Functions === #


# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
