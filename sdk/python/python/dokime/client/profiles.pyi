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

# Third Party Imports
# Local Imports
from .._profiles import Document, Profile

# Type Checking Imports

# === Public Exports === #

__all__ = ["Document", "Profile", "configuration_path", "decode", "load", "remember_endpoint"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

def configuration_path(path: str | Path | None = None) -> Path: ...
def load(identifier: str, *, path: str | Path | None = None) -> Profile: ...
def decode(data: bytes) -> Document: ...
def remember_endpoint(profile: Profile, path: Path, endpoint: str) -> None: ...
def _profile(value: dict) -> Profile: ...
