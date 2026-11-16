# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Generate and Transfer Frontend Protobuf Modules."

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import cargo, paths
from dokime_tools.package.staging import sync_output

from ..build import configuration
from ..process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #


MODULES = configuration.ROOT / "target/generated/typescript"

# === Types === #

# === Classes === #

# === Public Functions === #


def prepare() -> None:
    """Generate Frontend Message Modules From The Checked-In Protocol Descriptor."""
    status = cargo.focused(
        configuration.ROOT,
        "run",
        [
            "--locked",
            "--package",
            "dokime-gui",
            "--example",
            "export_preferences",
            "--",
            str(configuration.ROOT / "target/generated/preferences"),
        ],
    )
    if status:
        raise SystemExit(status)
    subprocess.run(
        ["node", str(configuration.ROOT / "tools/typescript/generate.mjs")],
        cwd=configuration.ROOT,
        check=True,
        creationflags=spawn_flags(),
    )
    subprocess.run(
        ["mise", "-C", str(configuration.ROOT / "crates/dokime-gui"), "run", "generate:host-frontend"],
        cwd=configuration.ROOT,
        check=True,
        creationflags=spawn_flags(),
    )


def stage(profile: str) -> Path:
    """Copy The Freshly Generated Modules Into The Build Output."""
    staged = paths.output(configuration.ROOT, "build", profile) / "generated/typescript"
    sync_output(MODULES, staged)
    sync_output(configuration.ROOT / "target/generated/host", staged.parent / "host")
    return staged


def restore(profile: str) -> None:
    """Replace The GUI's Generated Modules With The Staged Build Output."""
    output = paths.output(configuration.ROOT, "build", profile)
    staged = output / "generated/typescript"
    if not staged.is_dir():
        raise SystemExit(f"missing Product generated modules {staged}; run `mise run build`")
    host = staged.parent / "host"
    if not host.is_dir():
        raise SystemExit(f"missing Product host modules {host}; run `mise run build -- --language typescript`")
    sync_output(staged, MODULES)
    sync_output(host, configuration.ROOT / "target/generated/host")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=paths.PROFILES, default="dev")
    parser.add_argument("--generate", action="store_true", help="Generate Frontend Protobuf Modules")
    options = parser.parse_args()
    if options.generate:
        prepare()
    else:
        restore(options.profile)
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
