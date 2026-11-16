# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Check Guide Public Surfaces Independently of Repository and SDK Checks."""

# === Imports === #

# Standard Library Imports
import re
from pathlib import Path

# isort: split

# Third Party Imports
# isort: split

# Local Imports
from .. import policy

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def check(root: Path) -> None:
    wire = re.compile(r"\b_wire\b|\w+_wire\b|\b(?:Harmos|Dokime)\.Internal\b|\b(?:[A-Za-z_]\w*(?:\.|::))+v\d+\b|[/\\]v\d+[/\\]")
    grpc_path = re.compile(r"/?[a-zA-Z_]\w*(?:\.\w+)*\.v\d+(?:\.\w+)*/[A-Za-z_]\w*")
    candidates = [root / "README.md", *sorted((root / "docs").glob("*.md"))]
    findings = []
    for path in candidates:
        if not path.is_file() or path.relative_to(root).as_posix() in {"docs/architecture.md", "docs/execution-host.md"}:
            continue
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            line = grpc_path.sub("", line)
            line = re.sub(r"\]\([^)]*\)", "]()", line)
            if wire.search(line):
                findings.append(f"{path.relative_to(root).as_posix()}:{number}: Public Surface Names a Wire Package")
    if findings:
        raise ValueError("Check Guide Public Surfaces: Use the Public SDK Barrels:\n" + "\n".join(findings))
    print("Guide Public Surfaces Passed")


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    check(policy.root())
