# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Install the Pure Wheel in a Fresh Environment and Run the Starter."

# === Imports === #

# Standard Library Imports
import argparse
import os
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
from tempfile import mkdtemp

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ..build import paths
from ..build.protocol import inputs
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
    if options.stubs:
        inputs(root)
    stub_requirements = tomllib.loads((root / "sdk/python/pyproject.toml").read_text(encoding="utf-8"))["dependency-groups"]["stubs"]
    wheels = list((paths.output(root, "build", options.profile, "any") / "python").glob("*.whl"))
    if len(wheels) != 1:
        raise ValueError("Run Wheel Tests: build:python must export exactly one Python wheel")
    environment = dict(os.environ)
    environment["DOKIME_TEST_HOST"] = str(
        paths.output(root, "build", options.profile) / "fixtures/host" / ("dokime-host.exe" if os.name == "nt" else "dokime-host")
    )
    environment.pop("PYTHONPATH", None)
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    environment.pop("VIRTUAL_ENV", None)
    install = Path(mkdtemp(prefix="dokime-wheel-"))
    try:
        python = install / ".venv" / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        example = install / "test"
        shutil.copytree(root / "sdk/python/examples/starter", example, ignore=shutil.ignore_patterns(".venv", "__pycache__"))
        commands = [
            ["uv", "venv", "--python", sys.executable, str(install / ".venv")],
            ["uv", "pip", "install", "--python", str(python), str(wheels[0]), *(stub_requirements if options.stubs else [])],
            [
                str(python),
                "-I",
                "-c",
                (
                    "from importlib.metadata import distribution\n"
                    "from pathlib import Path\n"
                    "import dokime_sdk\n"
                    "from importlib.util import find_spec\n"
                    "package = Path(dokime_sdk.__file__).parent\n"
                    "installed = distribution('dokime-sdk')\n"
                    "assert package.joinpath('py.typed').is_file()\n"
                    "assert package.joinpath('LICENSE').is_file()\n"
                    "assert package.joinpath('NOTICE').is_file()\n"
                    "assert list(package.rglob('*.pyi'))\n"
                    "assert find_spec('dokime') is None\n"
                    "assert 'Root-Is-Purelib: true' in installed.read_text('WHEEL')\n"
                    "print('Installed Pure Wheel:', installed.version, package)\n"
                ),
            ],
        ]
        commands.append(["uv", "pip", "install", "--python", str(python), "--no-deps", str(example)])
        if not options.generated_only:
            commands.append([str(python), "-m", "unittest", "discover", "-s", "tests", "-v"])
        if options.stubs:
            commands.append([str(python), "-I", str(root / "sdk/python/tools/generated.py")])
            commands.append([str(python), "-I", str(root / "sdk/python/tools/stubs.py")])
            commands.append([str(python), "-I", "-m", "mypy", "--strict", "--no-incremental", str(root / "sdk/python/tests/generation")])
        for command in commands:
            result = subprocess.run(
                command, cwd=example, env=environment, creationflags=spawn_flags(), capture_output=True, text=True, check=False
            )
            print(result.stdout, end="", flush=True)
            print(result.stderr, end="", file=sys.stderr, flush=True)
            result.check_returncode()
    finally:
        shutil.rmtree(install)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
