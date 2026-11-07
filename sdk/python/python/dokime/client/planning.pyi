# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import Mapping
from dataclasses import dataclass

# Third Party Imports
# Local Imports
from ._transport import Transport
from .models import Backend, Value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

VALUE_DEPTH: int

# === Types === #

# === Classes === #

@dataclass(frozen=True, slots=True)
class TestCase:
    id: str
    suite_id: str
    template_id: str
    label: str
    definition_revision: str
    provider: str
    _parameters: bytes
    _transport: Transport
    _backend: Backend

    @property
    def parameters(self) -> dict[str, Value]: ...

@dataclass(frozen=True, slots=True)
class Template:
    id: str
    label: str
    _suite_id: str
    _template_id: str
    _transport: Transport
    _backend: Backend

    async def materialize(self, *, parameters: Mapping[str, Value] | None = None) -> tuple[TestCase, ...]: ...
