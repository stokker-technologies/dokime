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
import re
import stat
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# Third Party Imports
# Local Imports
from ._transport import _target

# Type Checking Imports


# === Public Exports === #

# === Constants === #
_MAX_BYTES = 1024 * 1024
_NAME = re.compile(r"[A-Za-z0-9_.-]{1,64}\Z")


# === Types === #

# === Classes === #


@dataclass(frozen=True)
class Profile:
    """An immutable selection; tokens are supplied separately by the caller."""

    name: str
    endpoint: str
    expected_backend_id: str | None
    ca_certificate_pem: str | None
    credential: str


@dataclass(frozen=True)
class Document:
    default: str | None
    profiles: tuple[Profile, ...]

    def get(self, name: str) -> Profile:
        _name(name)
        for profile in self.profiles:
            if profile.name == name:
                return profile
        raise ValueError("requested backend profile does not exist")


# === Public Functions === #


def configuration_path(path: str | Path | None = None) -> Path:
    """Locate config without creating it. Explicit override precedes environment."""
    if path is not None:
        return Path(path)
    if "DOKIME_PROFILES_FILE" in os.environ:
        value = os.environ["DOKIME_PROFILES_FILE"]
        if not value:
            raise ValueError("DOKIME_PROFILES_FILE is empty")
        return Path(value)
    if sys.platform == "win32":
        base = _environment_path("APPDATA")
    elif sys.platform == "darwin":
        base = _environment_path("HOME") / "Library/Application Support"
    else:
        value = os.environ.get("XDG_CONFIG_HOME")
        base = Path(value) if value else _environment_path("HOME") / ".config"
    if not base.is_absolute():
        raise ValueError("configuration root must be absolute")
    return base / "dokime/backends.json"


def load(name: str, *, path: str | Path | None = None) -> Profile:
    """Read one explicitly named profile. No default selection or secret access."""
    _name(name)
    source = configuration_path(path)
    # Bound Both The Read And The Decoded Document.
    if not source.is_file():
        raise ValueError("profile configuration must be a regular file")
    with source.open("rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise ValueError("profile configuration must be a regular file")
        data = stream.read(_MAX_BYTES + 1)
    return decode(data).get(name)


def decode(data: bytes) -> Document:
    if len(data) > _MAX_BYTES:
        raise ValueError("backend profile file exceeds 1 MiB")
    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=_object)
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid backend profile JSON") from error
    value = _fields(value, {"version", "default", "profiles"}, {"version", "profiles"})
    if type(value["version"]) is not int or value["version"] != 1:
        raise ValueError("unsupported backend profile version")
    items = value["profiles"]
    if not isinstance(items, list) or len(items) > 128:
        raise ValueError("invalid backend profile inventory")
    try:
        profiles = tuple(_profile(item) for item in items)
    except TypeError as error:
        raise ValueError("invalid backend profile field type") from error
    if len({profile.name for profile in profiles}) != len(profiles):
        raise ValueError("duplicate backend profile name")
    document = Document(value.get("default"), profiles)
    if document.default is not None:
        document.get(document.default)
    return document


# === Private Functions === #


def _profile(value: object) -> Profile:
    value = _fields(
        value,
        {"name", "endpoint", "expected_backend_id", "ca_certificate_pem", "credential"},
        {"name", "endpoint", "credential"},
    )
    name = _name(value["name"])
    endpoint = value["endpoint"]
    if not isinstance(endpoint, str):
        raise TypeError("invalid backend endpoint")
    _, tls = _target(endpoint)
    identity = value.get("expected_backend_id")
    if identity is not None and (
        not isinstance(identity, str)
        or not identity
        or len(identity.encode("utf-8")) > 256
        or any(ord(char) < 32 or 127 <= ord(char) <= 159 for char in identity)
    ):
        raise ValueError("invalid expected backend identity")
    ca = value.get("ca_certificate_pem")
    if ca is not None and (not isinstance(ca, str) or not ca or len(ca.encode("utf-8")) > _MAX_BYTES or "\0" in ca or not tls):
        raise ValueError("invalid CA certificate association")
    credential = value["credential"]
    if credential not in ("none", "keyring"):
        raise ValueError("invalid credential association")
    return Profile(name, endpoint, identity, ca, credential)


def _fields(value: object, allowed: set[str], required: set[str]) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) - allowed or required - set(value):
        raise ValueError("profile document does not match the version 1 schema")
    return value


def _object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON member in backend profiles")
        result[key] = value
    return result


def _name(value: object) -> str:
    if not isinstance(value, str) or not _NAME.fullmatch(value):
        raise ValueError("invalid backend profile name")
    return value


def _environment_path(name: str) -> Path:
    value = os.environ.get(name)
    if not value:
        raise ValueError("standard configuration root is unavailable; provide an explicit profile file")
    return Path(value)


# === Entry Point === #
