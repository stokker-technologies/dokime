# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

from __future__ import annotations

from typing import Literal

ProductConnectionState = Literal["connected", "disconnected"]
ProductReadinessState = Literal["ready", "not_ready", "busy", "unknown"]
ProductTargetLockState = Literal["locked", "lost", "unknown"]
ProductCalibrationKind = Literal["field", "tilt"]
ProductCalibrationPhase = Literal["started", "activity", "completed"]

def builtin_record_contracts() -> list[dict[str, object]]: ...
