# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify The Installed Native Wheel"""

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

# Third Party Imports
from harmos_tools.build import paths
from harmos_tools.process import spawn_flags

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="dev")
    parser.add_argument("--stubs", action="store_true")
    options = parser.parse_args()
    root = Path.cwd()
    wheels = list((paths.output(root, "build", options.profile) / "python").glob("*.whl"))
    if len(wheels) != 1:
        raise ValueError("build:python must export exactly one native wheel")
    environment = dict(os.environ)
    environment.pop("PYTHONPATH", None)
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    environment.pop("VIRTUAL_ENV", None)
    with TemporaryDirectory(prefix="dokime-wheel-") as scratch:
        install = Path(scratch)
        python = install / ".venv" / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        commands = [
            ["uv", "init", "--bare", "--name", "dokime-wheel-proof", "--python", sys.executable],
            [
                "uv",
                "add",
                "--python",
                sys.executable,
                str(wheels[0]),
            ],
            [
                str(python),
                "-I",
                "-c",
                (
                    "import dokime, dokime._native, importlib.machinery; from pathlib import Path; "
                    "native = Path(dokime._native.__file__); "
                    "assert any(str(native).endswith(s) for s in importlib.machinery.EXTENSION_SUFFIXES); "
                    "assert native.parent.joinpath('py.typed').is_file(); "
                    "print('Installed Native Wheel:', dokime.__version__, native)"
                ),
            ],
        ]
        if options.stubs:
            commands.append(["uv", "add", "--dev", "mypy==2.4.1", "types-protobuf", "polars>=2.1,<3", "pandas>=3,<4", "pandas-stubs"])
            commands.append([str(python), "-I", str(Path(__file__).with_name("stubs.py")), "dokime"])
        for command in commands:
            result = subprocess.run(
                command, cwd=install, env=environment, creationflags=spawn_flags(), capture_output=True, text=True, check=False
            )
            print(result.stdout, end="", flush=True)
            print(result.stderr, end="", file=sys.stderr, flush=True)
            result.check_returncode()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
