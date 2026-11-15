# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Stage the reference host from the immutable Harmos source revision."""

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

revision: str = "b15faaf202a005cf5077b3938557980f7f49fa37"
output: Path = Path(__file__).resolve().parents[1] / "../../target/cs/reference"
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
