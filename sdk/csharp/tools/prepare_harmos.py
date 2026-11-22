# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Stage the Local Library Main Checkout for Integration Only."""

# === Imports === #

# Standard Library Imports
import argparse
import os
import shutil
import subprocess
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

root = Path(__file__).resolve().parents[1]
# Integration Only: Stage Current Sources Without Writing in the Library Checkout.
source = Path(os.environ.get("HARMOS_SOURCE") or root / "../../../harmos").resolve()
revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
version = f"0.1.0-dev.{revision[:12]}"
feed = root / "../../target/cs/feed"
feed.mkdir(parents=True, exist_ok=True)
output = (root / "../../target/cs/h").resolve()
output.mkdir(parents=True, exist_ok=True)
for relative in (
    "sdk/csharp/src/Harmos",
    "sdk/csharp/tests/Harmos.Conformance",
    "sdk/tests/fixtures/conformance/proto",
):
    destination = (output / relative).resolve()
    if not destination.is_relative_to(output):
        raise ValueError("Stage Integration Library: Keep Sources Under the SDK Output")
    if destination.exists():
        shutil.rmtree(destination)
    shutil.copytree(source / relative, destination)
shutil.copyfile(source / "sdk/csharp/Directory.Build.props", output / "sdk/csharp/Directory.Build.props")

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
