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
from dokime_tools import temporary
from dokime_tools.build import dependencies, frontend, paths, product, starter
from dokime_tools.package.staging import clear_output, stage_file, sync_output
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
    staged = product.files(profile)
    sync_output(output / "web", frontend.WEB_ROOT / "dist")
    features = "tauri/custom-protocol,dokime/fault-injection"

    # Compile Test Targets Through The Shared Archive Owner.
    built = archive.build(ROOT, profile, ["--features", features])

    # Carry Client Inputs In The Tests Archive.
    modules = output / "fixtures/generated/typescript"
    sync_output(output / "generated/typescript", modules)
    module_files = [path for path in modules.rglob("*") if path.is_file()]
    staged.extend(module_files)
    wheels = paths.output(ROOT, "build", profile, "any") / "python"
    clear_output(output / "fixtures/python")
    for wheel in wheels.glob("dokime_client-*.whl"):
        destination = output / "fixtures/python" / wheel.name
        stage_file(wheel, destination)
        staged.append(destination)

    metadata = built.with_name("metadata.json")
    with metadata.open("w", encoding="utf-8") as written:
        command = ["cargo", "metadata", "--locked", "--format-version", "1", "--features", features]
        subprocess.run(command, cwd=ROOT, stdout=written, check=True)
    staged.append(metadata)
    subprocess.run(
        [
            "cargo",
            "nextest",
            "run",
            "--locked",
            "--workspace",
            "--all-targets",
            "--cargo-profile",
            profile,
            "--features",
            features,
            "-E",
            "binary_id(dokime-macros::compile)",
        ],
        cwd=ROOT,
        check=True,
    )
    # Compile The Starter Here; Ordinary Stories Consume Already Built Inputs.
    with temporary.directory() as directory:
        dependencies.cargo(ROOT, directory)
        dependencies.wheelhouse(ROOT, directory)
        starter.check(ROOT, profile, directory)

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
