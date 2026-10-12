# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Record contract helpers."""

# === Imports === #

# Standard Library Imports
import sys
from typing import Literal, Union

# Third Party Imports

# Local Imports
from dokime import _native
from dokime.core import record as _record
from dokime.core import records as _records

# Type Checking Imports

# === Public Exports === #

# === Constants === #

_parent = sys.modules.get("dokime")
if _parent is not None:
    setattr(_parent, "record", _record)
    setattr(_parent, "records", _records)

# === Public Types === #
ProductConnectionState = Union[Literal["connected"], Literal["disconnected"]]
ProductReadinessState = Union[
    Literal["ready"], Literal["not_ready"], Literal["busy"], Literal["unknown"]
]
ProductTargetLockState = Union[Literal["locked"], Literal["lost"], Literal["unknown"]]
ProductCalibrationKind = Union[Literal["field"], Literal["tilt"]]
ProductCalibrationPhase = Union[
    Literal["started"], Literal["activity"], Literal["completed"]
]

# === Types === #

# === Classes === #

# === Public Functions === #


def builtin_record_contracts() -> list[dict[str, object]]:
    return _native.builtin_record_contracts()


# === Private Functions === #

# === Entry Point === #
