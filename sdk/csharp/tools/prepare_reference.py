# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Stage and build the reference host from the immutable Harmos source revision."""

# === Imports === #

# Standard Library Imports
import io
import os
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

revision: str = "c2cfb153ab136159f07741d1061fcb3a1191e86c"
# Each Revision Stages and Builds Apart: Cargo Fingerprints Path Sources by Location and Time, So a Shared Source
# or Target Directory Could Reuse Another Revision's Artifacts.
target: Path = (Path(__file__).resolve().parents[1] / "../../target").resolve()
output: Path = target / "cs/reference" / revision
output.mkdir(parents=True, exist_ok=True)
source: str | None = os.environ.get("HARMOS_SOURCE")
if source:
    archive: bytes = subprocess.check_output(["git", "-C", source, "archive", "--format=zip", revision])
    files = zipfile.ZipFile(io.BytesIO(archive))
    try:
        files.extractall(output)
    finally:
        files.close()
else:
    response = urllib.request.urlopen(f"https://github.com/stokker-technologies/harmos/archive/{revision}.tar.gz")
    try:
        archive = response.read()
    finally:
        response.close()
    members = tarfile.open(fileobj=io.BytesIO(archive))
    try:
        for member in members.getmembers():
            member.name = str(Path(*Path(member.name).parts[1:]))
            if member.name != ".":
                members.extract(member, output, filter="data")
    finally:
        members.close()
subprocess.run(
    [
        "cargo",
        "install",
        "--path",
        str(output / "tools/conformance"),
        "--locked",
        "--debug",
        "--force",
        "--root",
        str(target / "reference-host"),
        "--bin",
        "harmos-conformance",
        "--jobs",
        "1",
        "--target-dir",
        str(target / "reference-host/build" / revision),
    ],
    check=True,
)
