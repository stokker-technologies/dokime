# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Benchmark Leaves And Their Matching Hosts Without A Test Archive"""

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
import sys

# Third Party Imports
# Local Imports
from dokime_tools.build import bench, frontend, paths
from dokime_tools.package import plugins
from dokime_tools.test import archive

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build() -> None:
    """Use The Shared Bench Producer And Stage The Host And OBC Fixtures Its System Stories Measure."""
    extra = ("--exclude", "dokime-gui", "--features", "plugin-obc/benchmarks,plugin-host/runtime")
    messages = bench.build(ROOT, extra=extra)
    output = paths.output(ROOT, "build", "bench")
    directory = output / "bench"
    binaries = output / "bin"
    tool = binaries / f"cargo-dokime{'.exe' if sys.platform == 'win32' else ''}"

    staged = plugins.prepare(ROOT, messages, "bench", tool=tool, output=output / "fixtures/plugins", packages=("plugin-host", *plugins.FIXTURES))

    staged += [path for path in directory.rglob("*") if path.is_file()]
    staged += [path for path in binaries.iterdir() if path.is_file()]

    root = ROOT.resolve()
    resolved = [path.resolve() for path in staged]
    relative = [path.relative_to(root) for path in resolved]
    names = [path.as_posix() for path in relative]
    names.sort()
    archive.stamp(ROOT, "bench", names)


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.parse_args(arguments)
    try:
        build()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build bench: {error}", file=sys.stderr)
        return 1
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
