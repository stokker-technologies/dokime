# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"""Generated Backend Profiles From Domain Declarations"""

from __future__ import annotations

# === Imports === #
# Standard Library Imports
from dataclasses import dataclass
from typing import Literal

# Third Party Imports
# Local Imports
# Type Checking Imports

# === Public Exports === #
# === Constants === #
# === Types === #

type Secret = Literal["none", "keyring"]

# === Classes === #


@dataclass(frozen=True, kw_only=True)
class Profile:
    ca_certificate_pem: str | None
    endpoint: str
    endpoints: tuple[str, ...]
    expected_backend_id: str | None
    identifier: str
    label: str
    secret: Secret


@dataclass(frozen=True, kw_only=True)
class Document:
    default: str | None
    profiles: tuple[Profile, ...]
    version: int


# === Public Functions === #
# === Private Functions === #
# === Entry Point === #
