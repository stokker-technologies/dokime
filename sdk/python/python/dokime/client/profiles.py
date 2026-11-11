# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Explicit Named Backend Profile Loading Without Network Or Secret Store IO"""

# === Imports === #

# Standard Library Imports
import json
import os
from dataclasses import asdict
from pathlib import Path

# Third Party Imports
# Local Imports
from .. import _native
from .._profiles import Document, Profile

# Type Checking Imports

# === Public Exports === #

__all__ = ["Document", "Profile", "configuration_path", "decode", "load", "remember_endpoint"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def configuration_path(path: str | Path | None = None) -> Path:
    # Python Retains Empty Environment Entries That Windows Removes From The OS Block.
    if path is None and "DOKIME_PROFILES_FILE" in os.environ:
        value = os.environ["DOKIME_PROFILES_FILE"]
        if not value:
            raise ValueError("DOKIME_PROFILES_FILE is empty")
        path = Path(value)
    return _native.profile_path(Path(path) if path is not None else None)


def load(identifier: str, *, path: str | Path | None = None) -> Profile:
    return _profile(json.loads(_native.load_profile(identifier, configuration_path(path))))


def decode(data: bytes) -> Document:
    value = json.loads(_native.decode_profiles(data))
    return Document(default=value.get("default"), profiles=tuple(_profile(item) for item in value["profiles"]))


def remember_endpoint(profile: Profile, path: Path, endpoint: str) -> None:
    _native.remember_endpoint(json.dumps(asdict(profile)), path, endpoint)


# === Private Functions === #


def _profile(value: dict) -> Profile:
    return Profile(
        identifier=value["identifier"],
        label=value["label"],
        endpoint=value["endpoint"],
        endpoints=tuple(value.get("endpoints", ())),
        expected_backend_id=value.get("expected_backend_id"),
        ca_certificate_pem=value.get("ca_certificate_pem"),
        secret=value["secret"],
    )


# === Entry Point === #
