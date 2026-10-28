# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build The Shared Test Archive And Its Fixtures Without Replacing Product Exports"""

import argparse
import subprocess
import sys
from time import perf_counter

from dokime_tools.build import cargo, configuration, frontend
from dokime_tools.package.staging import clear_output
from dokime_tools.test import archive, policy

ROOT = frontend.ROOT


def build(profile: str, *, extended: bool = False) -> list[str]:
    """Compile The Independent Canonical Test Archive And Its Metadata."""
    if profile not in configuration.PROFILES:
        raise ValueError("test archives require dev or release")
    features = configuration.features(profile, desktop=False, extended=extended)

    # Compile Test Targets Through The Shared Archive Owner.
    started = perf_counter()
    print(f"[build:tests] Archive started profile={profile} features={features}", flush=True)
    built = cargo.archive(ROOT, profile, extra=features)
    print(f"[build:tests] Archive finished in {perf_counter() - started:.2f}s", flush=True)

    metadata = built.with_name("metadata.json")
    with metadata.open("w", encoding="utf-8") as written:
        command = ["cargo", "metadata", "--locked", "--format-version", "1", *features]
        subprocess.run(
            command,
            cwd=ROOT,
            stdout=written,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stderr=sys.stderr,
        )
    staged = [built, built.with_name("compiler.json"), metadata]
    root = ROOT.resolve()
    resolved = [path.resolve() for path in staged]
    relative = {path.relative_to(root) for path in resolved}
    names = [path.as_posix() for path in relative]
    names.sort()
    clear_output(built.parent.parent / "fixtures")
    archive.stamp(ROOT, profile, names)
    return names


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=configuration.PROFILES, default="dev")
    parser.add_argument("--extended", action="store_true", help="include local-only fault-injection stories")
    options = parser.parse_args(arguments)
    try:
        build(options.profile, extended=options.extended)
    except (policy.TestError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build tests: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
