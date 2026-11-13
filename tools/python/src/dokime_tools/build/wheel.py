# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build The Pure Python SDK From Authoritative Protobuf Sources."""

# === Imports === #

# Standard Library Imports
import email.parser
import hashlib
import json
import os
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools import policy
from dokime_tools.build import paths
from dokime_tools.package.staging import clear_output
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(root: Path, profile: str) -> list[Path]:
    """Build One Platform-Independent Wheel In The Selected Product Output."""
    root = root.resolve()
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
        raise ValueError(f"expected one pure Python wheel in {output}")
    print(f"Python Wheel: {wheels[0].name} ({wheels[0].stat().st_size} bytes)", flush=True)
    return [*wheels, *sorted(output.glob("*.tar.gz"))]


def distributions(root: Path, output: Path, version: str) -> list[Path]:
    """Select The Single Pure SDK Wheel And Source Distribution."""
    expected = [output / f"dokime_sdk-{version}-py3-none-any.whl", output / f"dokime_sdk-{version}.tar.gz"]
    actual = sorted(path for path in output.iterdir() if path.is_file() and path.name != ".gitignore") if output.is_dir() else []
    if actual != sorted(expected) or any(path.is_symlink() for path in actual):
        raise policy.ToolError(f"expected one dokime pure wheel and source distribution for {version} in {output}")
    return expected


def verify(root: Path, output: Path, version: str) -> list[Path]:
    """Verify Package Identity, Generated Protocol Sources And Pure Wheel Contents."""
    selected = distributions(root, output, version)
    parser = email.parser.Parser()
    with zipfile.ZipFile(selected[0]) as archive:
        names = archive.namelist()
        metadata = parser.parsestr(archive.read(f"dokime_sdk-{version}.dist-info/METADATA").decode())
        wheel = parser.parsestr(archive.read(f"dokime_sdk-{version}.dist-info/WHEEL").decode())
        if metadata["Name"] != "dokime-sdk" or metadata["Version"] != version:
            raise policy.ToolError("Python distribution metadata differs from release identity")
        if wheel["Root-Is-Purelib"] != "true" or wheel.get_all("Tag") != ["py3-none-any"]:
            raise policy.ToolError("Python wheel must be pure py3-none-any")
        if any(name.lower().endswith((".so", ".pyd", ".dll", ".dylib", ".a", ".lib")) for name in names):
            raise policy.ToolError("Python wheel contains a native extension")
        sources = sorted((root / "proto").rglob("*.proto"))
        manifest = json.loads(archive.read("dokime/v1/sources.json"))
        expected = {path.relative_to(root / "proto").as_posix(): hashlib.sha256(path.read_bytes()).hexdigest() for path in sources}
        if not expected or manifest != expected:
            raise policy.ToolError("Python wheel protocol sources differ from proto/")
        required = {"dokime/v1/surface.pb"}
        for path in sources:
            if path.relative_to(root / "proto").parts[0] != "dokime":
                continue
            stem = path.relative_to(root / "proto").with_suffix("").as_posix()
            required.update(stem + suffix for suffix in ("_pb2.py", "_pb2.pyi", "_pb2_grpc.py"))
        if not required.issubset(names):
            raise policy.ToolError("Python wheel is missing generated protocol modules")
    with tarfile.open(selected[1], "r:gz") as archive:
        for path in sources:
            member = f"dokime_sdk-{version}/proto/{path.relative_to(root / 'proto').as_posix()}"
            content = archive.extractfile(member)
            if content is None or content.read() != path.read_bytes():
                raise policy.ToolError("Python source distribution is missing authoritative protocol sources")
    return selected


# === Private Functions === #

# === Entry Point === #
