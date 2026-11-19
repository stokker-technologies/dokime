# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Stage the exact unmodified Harmos C# SDK source."""

# === Imports === #

# Standard Library Imports
import argparse
import io
import os
import shutil
import subprocess
import tarfile
import urllib.request
import zipfile
from pathlib import Path

# Third Party Imports
# Local Imports
# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("dotnet")
parser.add_argument("--configuration", choices=("Debug", "Release"), default="Release")
parser.add_argument("--refresh-lock", action="store_true")
arguments = parser.parse_args()

revision: str = "dae69cea0850ce8dd3bda4ec9d87c5c6c5ab3912"
version: str = f"0.1.0-dev.{revision[:12]}"
root: Path = Path(__file__).resolve().parents[1]
feed: Path = root / "../../target/cs/feed"
feed.mkdir(parents=True, exist_ok=True)
output: Path = (root / "../../target/cs/h").resolve()
output.mkdir(parents=True, exist_ok=True)
source: str | None = os.environ.get("HARMOS_SOURCE")
if source:
    archive: bytes = subprocess.check_output(
        [
            "git",
            "-C",
            source,
            "archive",
            "--format=zip",
            revision,
            "sdk/csharp/src/Harmos",
            "sdk/csharp/tests/Harmos.Conformance",
            "sdk/csharp/Directory.Build.props",
            "sdk/tests/fixtures/conformance/proto",
            "proto",
        ]
    )
    files = zipfile.ZipFile(io.BytesIO(archive))
    try:
        files.extractall(output)
    finally:
        files.close()
else:
    url: str = f"https://github.com/stokker-technologies/harmos/archive/{revision}.tar.gz"
    response = urllib.request.urlopen(url)
    try:
        archive = response.read()
    finally:
        response.close()
    members = tarfile.open(fileobj=io.BytesIO(archive))
    try:
        for member in members.getmembers():
            relative = Path(*Path(member.name).parts[1:])
            if (
                relative.as_posix().startswith(
                    ("sdk/csharp/src/Harmos/", "sdk/csharp/tests/Harmos.Conformance/", "sdk/tests/fixtures/conformance/proto/", "proto/")
                )
                or relative.as_posix() == "sdk/csharp/Directory.Build.props"
            ):
                member.name = str(relative)
                members.extract(member, output, filter="data")
    finally:
        members.close()

shutil.copyfile(root / "tools/Harmos.csproj", output / "Harmos.csproj")
# The Package Identity Derives From the Revision; Solution Projects Reference This Staged Project.
properties: list[str] = [
    f"<Version>{version}</Version>",
    f"<SourceRevisionId>{revision}</SourceRevisionId>",
    f"<RepositoryCommit>{revision}</RepositoryCommit>",
    "<UseArtifactsOutput>true</UseArtifactsOutput>",
    "<ArtifactsPath>$(MSBuildThisFileDirectory)../harmos</ArtifactsPath>",
]
(output / "Directory.Build.props").write_bytes(
    "".join(["<Project>", "<PropertyGroup>", *properties, "</PropertyGroup>", "</Project>"]).encode("utf-8")
)
subprocess.run(
    [
        arguments.dotnet,
        "pack",
        "-p:RestoreLockedMode=" + ("false" if arguments.refresh_lock else "true"),
        str(output / "Harmos.csproj"),
        "--configuration",
        arguments.configuration,
        "--output",
        str(feed.resolve()),
    ],
    check=True,
)

# Explicit Lock Refresh Retires Only This SDK's Owned Library Package Caches.
if arguments.refresh_lock:
    for relative in ("packages/harmos", "generation/packages/harmos"):
        cache = (output.parent / relative).resolve()
        if not cache.is_relative_to(output.parent):
            raise ValueError("Refresh Library Feed: Cache Must Remain Under the SDK Output")
        shutil.rmtree(cache, ignore_errors=True)
