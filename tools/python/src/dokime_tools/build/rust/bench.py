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
import contextlib
import shutil
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import bench, cargo, frontend, outputs, paths
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
    extra = ("--exclude", "dokime-gui", "--features", "obc-plugin/benchmarks,dokime-host/runtime")
    messages = bench.build(ROOT, extra=extra)
    output = paths.output(ROOT, "build", "bench")
    directory = output / "bench"
    binaries = output / "bin"
    tool = binaries / f"cargo-dokime{'.exe' if sys.platform == 'win32' else ''}"

    fixtures = output / "fixtures/plugins"
    staged = plugins.prepare(ROOT, messages, "bench", tool=tool, output=fixtures, packages=("dokime-host",))

    fixture_messages = _obc_fixture(output, binaries)
    bundles = plugins.prepare(ROOT, fixture_messages, "bench", tool=tool, output=output / "fixtures/obc-bundles", packages=plugins.FIXTURES)
    for bundle in bundles:
        destination = fixtures / bundle.name
        shutil.copy2(bundle, destination)
        staged.append(destination)
    staged += [path for path in fixture_messages.parent.iterdir() if path.is_file()]

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


def _obc_fixture(output: Path, binaries: Path) -> Path:
    # A Shared Host Runtime Feature Would Link Two Sidecar Declarations Into OBC.
    directory = output / "fixtures/obc-build"
    directory.mkdir(parents=True, exist_ok=True)
    raw = directory / "compiler.raw"
    messages = directory / "compiler.json"
    arguments = [
        "--locked",
        "-p",
        "obc-plugin",
        "--bin",
        "obc-plugin",
        "--features",
        "benchmarks",
        "--profile",
        "bench",
        "--message-format=json-render-diagnostics",
    ]

    with raw.open("w", encoding="utf-8") as written, contextlib.redirect_stdout(written):
        status = cargo.focused(ROOT, "build", arguments)
    lines = raw.read_text(encoding="utf-8").splitlines()
    messages.write_text("\n".join(line for line in lines if line.startswith("{")) + "\n", encoding="utf-8")
    for line in (line for line in lines if not line.startswith("{")):
        print(line)
    raw.unlink()
    if status:
        raise subprocess.CalledProcessError(status, ["cargo", "build", *arguments])

    for executable in outputs.executables(messages, plugins.FIXTURES).values():
        shutil.copy2(executable, binaries / executable.name)
    return messages


# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
