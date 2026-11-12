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

revision = "de07fc54f38ece9fcd660af3146876aaf651fe3c"
root = Path(__file__).resolve().parents[1]
output = root / "obj" / "harmos"
output.mkdir(parents=True, exist_ok=True)
source = os.environ.get("HARMOS_SOURCE")
if source:
    archive = subprocess.check_output(
        ["git", "-C", source, "archive", "--format=zip", revision, "sdk/csharp/Harmos", "sdk/csharp/Directory.Build.props", "proto"]
    )
    with zipfile.ZipFile(io.BytesIO(archive)) as files:
        files.extractall(output)
else:
    url = f"https://github.com/stokker-technologies/harmos/archive/{revision}.tar.gz"
    with urllib.request.urlopen(url) as response:
        archive = response.read()
    with tarfile.open(fileobj=io.BytesIO(archive)) as files:
        for member in files.getmembers():
            relative = Path(*Path(member.name).parts[1:])
            if (
                relative.as_posix().startswith(("sdk/csharp/Harmos/", "proto/"))
                or relative.as_posix() == "sdk/csharp/Directory.Build.props"
            ):
                member.name = str(relative)
                files.extract(member, output, filter="data")
