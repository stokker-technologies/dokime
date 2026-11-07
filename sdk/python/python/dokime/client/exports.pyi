# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from pathlib import Path
from typing import Literal

# Third Party Imports
# Local Imports
from ._transport import Transport
from .history import Download
from .models import Backend

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

async def export_evidence(
    transport: Transport,
    backend: Backend,
    kind: Literal["Cycle", "Execution"],
    identity: str,
    destination: str | Path,
    *,
    overwrite: bool = False,
) -> Download: ...
def _destination(path: Path, overwrite: bool) -> None: ...
