# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Benchmark Leaves and Their Matching Hosts Without a Test Archive"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import os
import subprocess
import sys

# Third Party Imports
# Local Imports
from dokime_tools.build import bench, frontend, paths
from dokime_tools.package import plugins
from dokime_tools.prepare import generated
from dokime_tools.test import archive

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = frontend.ROOT

# === Types === #

# === Classes === #

# === Public Functions === #


def build(binary: str | None = None) -> None:
    """Build Benchmarks and Their Matching Native Fixtures."""
    extra = ("--features", "plugin-obc/benchmarks,plugin-host/runtime")
    if binary in (None, "system-desktop"):
        output = paths.output(ROOT, "build", "bench")
        # Prepare Authoritative Bindings Before The Bench Frontend On Fresh Checkouts.
        prepared = bench.build(
            ROOT, selection=("--package", "cargo-dokime", "--package", "dokime-server", "--bins", "--lib"), require_benchmark=False
        )
        generated.generate(prepared, check=True)
        checked = subprocess.run(
            ["node", "node_modules/typescript/bin/tsc", "--noEmit", "--project", "benches/system/tsconfig.json"],
            cwd=ROOT / "crates/dokime-gui",
            check=False,
            capture_output=True,
            text=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        )
        print(checked.stdout, end="")
        if checked.returncode:
            raise ValueError(f"desktop benchmark type check exited {checked.returncode}: {checked.stderr}")
        result = subprocess.run(
            ["node", "benches/system/build.mjs", str(output / "desktop-web")],
            cwd=ROOT / "crates/dokime-gui",
            check=False,
            capture_output=True,
            text=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        )
        print(result.stdout, end="")
        if result.returncode:
            raise ValueError(f"desktop benchmark frontend exited {result.returncode}: {result.stderr}")
        # generate_context! Does Not Track Embedded Frontend Bytes.
        subprocess.run(
            ["cargo", "clean", "--locked", "--package", "dokime-gui", "--profile", "bench"],
            cwd=ROOT,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        )
        # Only This Bench Host Embeds The Instrumented Frontend.
        # A Windows Drive Prefix Parses As A URL; Resolve Assets From Tauri's Config Directory.
        frontend_dist = os.path.relpath(output / "desktop-web", ROOT / "crates/dokime-gui/src-tauri")
        os.environ["TAURI_CONFIG"] = json.dumps({"build": {"frontendDist": frontend_dist}})
        extra = (*extra, "--features", "tauri/custom-protocol")
    if binary is None:
        messages = bench.build(ROOT, extra=extra)
    else:
        metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=ROOT))
        owners = [
            package["name"]
            for package in metadata["packages"]
            if any(target["name"] == binary and "bench" in target["kind"] for target in package["targets"])
        ]
        if len(owners) != 1:
            raise ValueError("benchmark selection must identify one native target")
        packages = sorted({*owners, "plugin-host", *plugins.FIXTURES, "cargo-dokime"})
        selection = ["--bins", "--bench", binary]
        for package in packages:
            selection.extend(("--package", package))
        messages = bench.build(ROOT, extra=extra, selection=selection)
    output = paths.output(ROOT, "build", "bench")
    directory = output / "bench"
    binaries = output / "bin"
    tool = binaries / f"cargo-dokime{'.exe' if sys.platform == 'win32' else ''}"

    staged = plugins.prepare(
        ROOT, messages, "bench", tool=tool, output=output / "fixtures/plugins", packages=("plugin-host", *plugins.FIXTURES)
    )

    if binary in (None, "integration-plots", "system-desktop"):
        generated.generate(messages, check=True)
        modules = generated.stage("bench")
        staged += [path for path in modules.rglob("*") if path.is_file()]

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
    parser.add_argument("--binary")
    options = parser.parse_args(arguments)
    try:
        build(options.binary)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build bench: {error}", file=sys.stderr)
        return 1
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    sys.exit(main())
