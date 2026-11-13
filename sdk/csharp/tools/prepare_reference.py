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

revision = "b5f595087967a085662689654bfadeff3d77ffa1"
output = Path(__file__).resolve().parents[1] / "../../target/csharp-reference"
output.mkdir(parents=True, exist_ok=True)
source = os.environ.get("HARMOS_SOURCE")
if source:
    archive = subprocess.check_output(["git", "-C", source, "archive", "--format=zip", revision])
    with zipfile.ZipFile(io.BytesIO(archive)) as files:
        files.extractall(output)
else:
    with urllib.request.urlopen(f"https://github.com/stokker-technologies/harmos/archive/{revision}.tar.gz") as response:
        archive = response.read()
    with tarfile.open(fileobj=io.BytesIO(archive)) as files:
        for member in files.getmembers():
            member.name = str(Path(*Path(member.name).parts[1:]))
            if member.name != ".":
                files.extract(member, output, filter="data")
