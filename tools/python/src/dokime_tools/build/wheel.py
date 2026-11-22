# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Build the Pure Python SDK from Authoritative Protobuf Sources."

# === Imports === #

# Standard Library Imports
import json
import os
import runpy
import subprocess
import sys
import tarfile
from email.parser import Parser
from pathlib import Path
from zipfile import ZipFile

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.build import paths
from dokime_tools.build.protocol import inputs
from dokime_tools.package.staging import clear_output
from dokime_tools.policy import ToolError
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(root: Path, profile: str) -> list[Path]:
    """Build One Platform-Independent Wheel in the Selected Product Output."""
    root = root.resolve()
    inputs(root)
    output = paths.output(root, "build", profile, "any") / "python"
    clear_output(output)
    output.mkdir(parents=True, exist_ok=True)
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    environment.pop("UV_PROJECT_ENVIRONMENT", None)
    command = [
        "uv",
        "build",
        "--out-dir",
        str(output),
        "sdk/python",
    ]
    subprocess.run(
        command,
        cwd=root,
        check=True,
        env=environment,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    wheels = sorted(output.glob("*.whl"))
    if len(wheels) != 1 or not wheels[0].name.endswith("-py3-none-any.whl"):
        raise ValueError(f"Build Wheel: expected one pure Python wheel in {output}")
    selected = verify(root, output, policy.config(root)["version"]["value"])
    print(f"Python Wheel: {wheels[0].name} ({wheels[0].stat().st_size} bytes)", flush=True)
    return selected


def distributions(root: Path, output: Path, version: str) -> list[Path]:
    """Select the Single Pure SDK Wheel and Source Distribution."""
    expected = [output / f"dokime_sdk-{version}-py3-none-any.whl", output / f"dokime_sdk-{version}.tar.gz"]
    actual = sorted(path for path in output.iterdir() if path.is_file() and path.name != ".gitignore") if output.is_dir() else []
    if actual != sorted(expected) or any(path.is_symlink() for path in actual):
        raise ToolError(f"Build Wheel: expected one dokime pure wheel and source distribution for {version} in {output}")
    return expected


def verify(root: Path, output: Path, version: str) -> list[Path]:
    """Verify Package Identity, Generated Protocol Sources and Pure Wheel Contents."""
    selected = distributions(root, output, version)
    parser = Parser()
    archive = ZipFile(selected[0])
    try:
        names = archive.namelist()
        metadata = parser.parsestr(archive.read(f"dokime_sdk-{version}.dist-info/METADATA").decode())
        wheel = parser.parsestr(archive.read(f"dokime_sdk-{version}.dist-info/WHEEL").decode())
        if metadata["Name"] != "dokime-sdk" or metadata["Version"] != version:
            raise ToolError("Build Wheel: Python distribution metadata differs from release identity")
        if wheel["Root-Is-Purelib"] != "true" or wheel.get_all("Tag") != ["py3-none-any"]:
            raise ToolError("Build Wheel: Python wheel must be pure py3-none-any")
        if any(name.lower().endswith((".so", ".pyd", ".dll", ".dylib", ".a", ".lib")) for name in names):
            raise ToolError("Build Wheel: Python wheel contains a native extension")
        protocols = inputs(root)
        source_inventory = runpy.run_path(str(Path(__file__).resolve().parents[5] / "sdk/python/tools/generate.py"))["source_inventory"]
        expected = source_inventory(protocols)
        manifest = json.loads(archive.read("dokime_sdk/_wire/sources.json"))
        if not expected or manifest != expected:
            differences = sorted(name for name in manifest.keys() | expected.keys() if manifest.get(name) != expected.get(name))
            raise ToolError(
                "Build Wheel: Regenerate With mise run generate -- --protocol --language python, Then Rebuild; "
                "Python wheel protocol sources differ from target/protocol: " + ", ".join(differences)
            )
        required = {"dokime_sdk/_wire/surface.pb"}
        for name in expected:
            if not name.startswith("dokime/"):
                continue
            stem = name.removesuffix(".proto").replace("dokime/", "dokime_sdk/_wire/", 1)
            required.update(stem + suffix for suffix in ("_pb2.py", "_pb2.pyi", "_pb2_grpc.py"))
        if not required.issubset(names):
            raise ToolError("Build Wheel: Python wheel is missing generated protocol modules")
    finally:
        archive.close()
    archive = tarfile.open(selected[1], "r:gz")
    try:
        for name in expected:
            member = f"dokime_sdk-{version}/proto/{name}"
            content = archive.extractfile(member)
            if content is None or content.read() != (protocols / name).read_bytes():
                raise ToolError("Build Wheel: Python source distribution is missing authoritative protocol sources")
    finally:
        archive.close()
    return selected


# === Private Functions === #

# === Entry Point === #
