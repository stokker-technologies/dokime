# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run Python Client Lanes With The Independent Python Product Wheel"""

# === Imports === #

# Standard Library Imports
import argparse
from pathlib import Path
from tempfile import TemporaryDirectory

# Third Party Imports
# Local Imports
from dokime_tools.build import paths
from dokime_tools.test import policy, run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    options, arguments = parser.parse_known_args()
    directory = paths.output(policy.root(), "build", options.profile, "any") / "python"
    wheels = list(directory.glob("dokime_client-*.whl"))
    source_selected = "--category" in arguments and arguments[arguments.index("--category") + 1] in ("unit", "integration") and "-k" in arguments
    if not source_selected and len(wheels) != 1:
        raise SystemExit(f"expected one staged client wheel in {directory}; run `mise run build:python`")
    # Each Invocation Owns Its Fixtures; Pytest Must Not Clean Another Run's Links.
    with TemporaryDirectory(prefix="dokime-pytest-") as scratch:
        if "--" not in arguments:
            arguments.append("--")
        arguments.append(f"--basetemp={Path(scratch) / 'fixtures'}")
        return run.main(["test", "--profile", options.profile, "--language", "python", "--project", "packages/dokime-client", *arguments])


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
