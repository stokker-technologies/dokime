# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Telemetry contract helpers."""

# === Imports === #

# Standard Library Imports

import sys
import types

# Third Party Imports

# Local Imports

from dokime.core import telemetry as _telemetry_expression


# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class _TelemetryModule(types.ModuleType):
    def __call__(self, stream: str, field_name: str) -> object:
        return _telemetry_expression(stream, field_name)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

# Keep the top-level authoring expression helper callable even after this
# submodule is imported.
_parent = sys.modules.get("dokime")
if _parent is not None:
    setattr(_parent, "telemetry", _telemetry_expression)
sys.modules[__name__].__class__ = _TelemetryModule
