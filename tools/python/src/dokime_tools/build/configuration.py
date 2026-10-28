# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Shared Build Configuration And Task Invocation"""

import argparse
import subprocess
import sys
import tomllib
from collections.abc import Callable
from pathlib import Path
from time import perf_counter

from dokime_tools.build import frontend, paths
from dokime_tools.test import policy

ROOT = frontend.ROOT
PROFILES = policy.PROFILES


def features(profile: str, *, desktop: bool = True, extended: bool = False) -> list[str]:
    """Resolve The Repository-Owned Shipping Feature Configuration."""
    declaration = tomllib.loads((ROOT / "mise.toml").read_text(encoding="utf-8"))["vars"]
    selected = []
    if desktop:
        selected.append(declaration["rust_desktop_features"])
    if profile == "release":
        selected.append(declaration["rust_release_features"])
    if extended:
        selected.append(declaration["rust_extended_features"])
    return ["--features", ",".join(selected)] if selected else []


def messages(profile: str) -> Path:
    return paths.output(ROOT, "build", profile) / "product/compiler.json"


def relative(paths: list[Path]) -> list[str]:
    return sorted(path.resolve().relative_to(ROOT.resolve()).as_posix() for path in paths)


def invoke(operation: Callable[[str], object]) -> int:
    parser = argparse.ArgumentParser(description=operation.__doc__)
    parser.add_argument("--profile", choices=PROFILES, default="dev")
    options = parser.parse_args()
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
