# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Run the Mirror Workflow Operation."

# === Imports === #

# Standard Library Imports
import os
import shlex
import subprocess
import tempfile
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ..process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def run() -> None:
    if os.environ["GITHUB_REF_NAME"] not in ("dev", "main") or os.environ["GITHUB_REPOSITORY"] != "stokker-technologies/dokime":
        raise ValueError("Mirror Published Branch: mirror requires a published Dokime branch")
    sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True, creationflags=spawn_flags()).strip()
    if sha != os.environ["GITHUB_SHA"]:
        raise ValueError("Mirror Published Branch: mirror checkout differs from published revision")
    key = os.environ["GITLAB_MIRROR_SSH_KEY"]
    hosts = os.environ["GITLAB_MIRROR_KNOWN_HOSTS"]
    if not key.strip() or not hosts.strip():
        raise ValueError("Mirror Published Branch: mirror SSH credentials and known hosts are required")
    temporary_directory = tempfile.TemporaryDirectory(prefix="dokime-mirror-")
    temporary = temporary_directory.name
    try:
        secret = Path(temporary) / "key"
        known = Path(temporary) / "hosts"
        secret.touch(mode=0o600)
        secret.write_text(key + "\n", encoding="utf-8")
        known.write_text(hosts + "\n", encoding="utf-8")
        ssh = f"ssh -i {shlex.quote(str(secret))} -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile={shlex.quote(str(known))}"
        subprocess.run(
            [
                "git",
                "push",
                "--no-force",
                "--no-follow-tags",
                "git@gitlab.com:stokker-technologies/open-source/dokime.git",
                f"HEAD:refs/heads/{os.environ['GITHUB_REF_NAME']}",
            ],
            env=os.environ | {"GIT_SSH_COMMAND": ssh},
            check=True,
            creationflags=spawn_flags(),
        )
    finally:
        temporary_directory.cleanup()


# === Private Functions === #

# === Entry Point === #
