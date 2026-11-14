# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run the already-built SDK journeys with explicit artifact paths."""
# === Imports === #

import shutil
from pathlib import Path

# === Entry Point === #

root = Path(__file__).resolve().parents[3]
shutil.rmtree(root / "target/cs/packages/dokime", ignore_errors=True)
