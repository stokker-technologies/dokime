# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Check That Every Workflow Job Running a Release Task Receives a GitHub Token."""

# === Imports === #

# Standard Library Imports
import re
import sys
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

GUARDED = re.compile(r"mise run (ci:review|ci:release-mode|release:[\w:-]+)")
TOKEN = re.compile(r"^\s*GH_TOKEN:", re.MULTILINE)
JOB = re.compile(r"^  ([\w-]+):\s*$")

# === Types === #

# === Classes === #

# === Public Functions === #


def findings(text: str) -> list[str]:
    """Jobs of One Workflow That Run a Guarded Task Without GH_TOKEN in the Job or Workflow Environment."""
    lines = text.splitlines()
    if "jobs:" not in lines:
        return []
    start = lines.index("jobs:")
    # The Workflow Environment Precedes `jobs:` in Every Authored Workflow.
    inherited = TOKEN.search("\n".join(lines[:start])) is not None
    jobs: dict[str, list[str]] = {}
    current = None
    for line in lines[start + 1 :]:
        if line and not line.startswith(" "):
            current = None
        elif match := JOB.match(line):
            current = match.group(1)
            jobs[current] = []
        elif current is not None:
            jobs[current].append(line)
    selected = []
    for name, body in jobs.items():
        block = "\n".join(body)
        tasks = sorted(set(GUARDED.findall(block)))
        if tasks and not inherited and TOKEN.search(block) is None:
            selected.append(f"job {name} runs {', '.join(tasks)} without GH_TOKEN; add GH_TOKEN: ${{{{ github.token }}}} to its env")
    return selected


def check(root: Path) -> list[str]:
    selected = []
    for path in sorted((root / ".github/workflows").glob("*.y*ml")):
        for finding in findings(path.read_text(encoding="utf-8")):
            selected.append(f"{path.relative_to(root).as_posix()}: {finding}")
    return selected


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    errors = check(Path.cwd())
    for error in errors:
        print(error, file=sys.stderr)
    raise SystemExit(bool(errors))
