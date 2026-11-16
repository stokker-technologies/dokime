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
import io
import os
import shutil
import subprocess
import sys
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

revision: str = "efc261fe9431bb3ad2fd3287ecb51a7f1d8da325"
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
subprocess.run(
    [
        sys.argv[1],
        "pack",
        # The Package Identity Derives From the Revision, So Changed Bytes Never Reuse a Locked Version.
        f"-p:Version={version}",
        f"-p:SourceRevisionId={revision}",
        f"-p:RepositoryCommit={revision}",
        "-p:RestorePackagesWithLockFile=true",
        f"-p:NuGetLockFilePath={root / 'tools/Harmos.packages.lock.json'}",
        "-p:RestoreLockedMode=" + ("false" if "--refresh-lock" in sys.argv else "true"),
        str(output / "Harmos.csproj"),
        "--configuration",
        "Release",
        "--artifacts-path",
        str(output.parent / "harmos"),
        "--output",
        str(feed.resolve()),
    ],
    check=True,
)

# Explicit Lock Refresh Retires Only This SDK's Owned Library Package Caches.
if "--refresh-lock" in sys.argv:
    for relative in ("packages/harmos", "generation/packages/harmos"):
        cache = (output.parent / relative).resolve()
        if not cache.is_relative_to(output.parent):
            raise ValueError("Refresh Library Feed: Cache Must Remain Under the SDK Output")
        shutil.rmtree(cache, ignore_errors=True)
