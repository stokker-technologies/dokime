# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Shared Build Configuration And Task Invocation"""

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
import sys
from collections.abc import Callable
from pathlib import Path
from time import perf_counter

# Third Party Imports
from harmos_tools.build import paths
from harmos_tools.test import policy

# Local Imports
from dokime_tools.build import cargo, frontend

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT
PROFILES = policy.PROFILES

# === Types === #

# === Classes === #

# === Public Functions === #


def messages(profile: str) -> Path:
    return paths.output(ROOT, "build", profile) / "product/compiler.json"


def relative(paths: list[Path]) -> list[str]:
    return sorted(path.resolve().relative_to(ROOT.resolve()).as_posix() for path in paths)


def invoke(operation: Callable[[str], object]) -> int:
    parser = argparse.ArgumentParser(description=operation.__doc__)
    parser.add_argument("--profile", choices=PROFILES, default="dev")
    options = parser.parse_args()
    os.environ.update(cargo.compiler_environment(options.profile))
    started = perf_counter()
    print(f"[build] {operation.__doc__} Started profile={options.profile}", flush=True)
    try:
        operation(options.profile)
    except (policy.TestError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build: {error}", file=sys.stderr)
        return 1
    finally:
        print(f"[build] {operation.__doc__} Finished in {perf_counter() - started:.2f}s", flush=True)
    return 0


# === Private Functions === #

# === Entry Point === #
