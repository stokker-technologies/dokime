# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Install The Pure Wheel In A Fresh Environment And Run The Starter."""

# === Imports === #

# Standard Library Imports
import argparse
import os
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
from tempfile import TemporaryDirectory

# Third Party Imports
from harmos_tools.build import paths

# Local Imports
from ..process import spawn_flags

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
    parser.add_argument("--generated-only", action="store_true")
    options = parser.parse_args()
    root = Path.cwd()
    stub_requirements = tomllib.loads((root / "sdk/python/pyproject.toml").read_text(encoding="utf-8"))["dependency-groups"]["stubs"]
    wheels = list((paths.output(root, "build", options.profile, "any") / "python").glob("*.whl"))
    if len(wheels) != 1:
        raise ValueError("build:python must export exactly one Python wheel")
    environment = dict(os.environ)
    environment["DOKIME_TEST_HOST"] = str(
        paths.output(root, "build", options.profile) / "fixtures/host" / ("dokime-host-next.exe" if os.name == "nt" else "dokime-host-next")
    )
    environment.pop("PYTHONPATH", None)
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    environment.pop("VIRTUAL_ENV", None)
    with TemporaryDirectory(prefix="dokime-wheel-") as scratch:
        install = Path(scratch)
        python = install / ".venv" / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        starter = install / "starter"
        shutil.copytree(root / "sdk/python/examples/starter", starter)
        commands = [
            ["uv", "venv", "--python", sys.executable, str(install / ".venv")],
            ["uv", "pip", "install", "--python", str(python), str(wheels[0]), *(stub_requirements if options.stubs else [])],
            [
                str(python),
                "-I",
                "-c",
                (
                    "import dokime; from pathlib import Path; from importlib.metadata import distribution; "
                    "package = Path(dokime.__file__).parent; "
                    "assert package.joinpath('py.typed').is_file(); "
                    "assert distribution('dokime-sdk').read_text('WHEEL').find('Root-Is-Purelib: true') >= 0; "
                    "print('Installed Pure Wheel:', dokime.__version__, package)"
                ),
            ],
        ]
        if not options.generated_only:
            commands.append([str(python), "-m", "unittest", "discover", "-s", "tests", "-v"])
        if options.stubs:
            commands.append([str(python), "-I", str(root / "sdk/python/tools/generated.py")])
            commands.append([str(python), "-I", str(root / "sdk/python/tools/stubs.py")])
        for command in commands:
            result = subprocess.run(
                command, cwd=starter, env=environment, creationflags=spawn_flags(), capture_output=True, text=True, check=False
            )
            print(result.stdout, end="", flush=True)
            print(result.stderr, end="", file=sys.stderr, flush=True)
            result.check_returncode()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
