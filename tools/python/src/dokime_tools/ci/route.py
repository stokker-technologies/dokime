# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Admit Stabilization And Declared Published-Tag Hotfixes"""

# === Imports === #

# Standard Library Imports
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

# Third Party Imports

# Local Imports
from dokime_tools.ci.artifacts import REPOSITORY, _request

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def version(root: Path) -> str:
    candidate = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))[
        "workspace"
    ]["package"]["version"]
    if not isinstance(candidate, str) or not re.fullmatch(r"\d+\.\d+\.\d+", candidate):
        raise ValueError("release candidates require a stable version")
    return candidate


def verify(event: dict, root: Path) -> None:
    request = event["pull_request"]
    if request["base"]["ref"] != "main":
        return
    head = request["head"]
    if (
        head["repo"]["full_name"] != REPOSITORY
        or request["base"]["repo"]["full_name"] != REPOSITORY
    ):
        raise ValueError("release promotion requires the configured repository")
    candidate = version(root)
    if head["ref"] == f"release/{candidate}":
        return
    settings = tomllib.loads((root / "stkr.toml").read_text(encoding="utf-8"))
    if not head["ref"].startswith("fix/") or not isinstance(
        settings.get("release", {}).get("hotfix"), dict
    ):
        raise ValueError(
            "only stabilization or declared hotfix branches may target main"
        )
    target = request["base"]["sha"]
    if request.get("merged") is True and request["merge_commit_sha"] != head["sha"]:
        target = subprocess.check_output(
            ["git", "rev-parse", request["merge_commit_sha"] + "^1"],
            cwd=root,
            text=True,
            encoding="utf-8",
        ).strip()
    base = subprocess.check_output(
        ["git", "merge-base", target, head["sha"]],
        cwd=root,
        text=True,
        encoding="utf-8",
    ).strip()
    tags = subprocess.check_output(
        ["git", "tag", "--points-at", base, "--list", "v*"],
        cwd=root,
        text=True,
        encoding="utf-8",
    ).splitlines()
    tags = [tag for tag in tags if re.fullmatch(r"v\d+\.\d+\.\d+", tag)]
    if len(tags) != 1:
        raise ValueError("hotfix must start from one published stable tag on main")
    remote = _request(f"releases/tags/{tags[0]}")
    previous = [int(part) for part in tags[0][1:].split(".")]
    previous[2] += 1
    if (
        remote.get("tag_name") != tags[0]
        or remote.get("draft") is not False
        or remote.get("prerelease") is not False
        or candidate != ".".join(map(str, previous))
    ):
        raise ValueError("hotfix must be the next patch of an actual published release")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    verify(
        json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8")),
        Path.cwd(),
    )
