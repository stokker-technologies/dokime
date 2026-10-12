# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from pathlib import Path
import runpy

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

EXAMPLES_DIR = Path(__file__).resolve().parents[2] / "examples"

# === Types === #

# === Classes === #

# === Public Functions === #


def test_python_small():
    examples = sorted(EXAMPLES_DIR.glob("*.py"))

    assert examples, "expected at least one Python authoring example"
    for example in examples:
        assert len(example.read_text(encoding="utf-8").splitlines()) < 500
        runpy.run_path(str(example), run_name="__main__")


# === Private Functions === #

# === Entry Point === #
