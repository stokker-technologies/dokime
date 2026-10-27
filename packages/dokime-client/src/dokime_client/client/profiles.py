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
from pathlib import Path
from typing import Any

from .._profiles import Document, Profile

# Third Party Imports
# Local Imports
from ._transport import _target

# Type Checking Imports

# === Public Exports === #

# === Constants === #

CA_BYTES = 1048576
PROFILE_BYTES = 1048576
PROFILE_ID_BYTES = 256
PROFILE_IDENTIFIER_BYTES = 128
PROFILES = 128

_IDENTIFIER = re.compile(r"[A-Za-z0-9_.-]+\Z")

# === Types === #

# === Classes === #


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


def load(identifier: str, *, path: str | Path | None = None) -> Profile:
    """Read one explicitly named profile. No default selection or secret access."""
    _identifier(identifier)
    source = configuration_path(path)
    # Bound Both The Read And The Decoded Document.
    if not source.is_file():
        raise ValueError("profile configuration must be a regular file")
    with source.open("rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise ValueError("profile configuration must be a regular file")
        data = stream.read(PROFILE_BYTES + 1)
    return _get(decode(data), identifier)


def decode(data: bytes) -> Document:
    if len(data) > PROFILE_BYTES:
        raise ValueError("backend profile file exceeds the configured byte limit")
    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=_object)
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid backend profile JSON") from error
    value = _fields(value, {"version", "default", "profiles"}, {"version", "profiles"})
    if type(value["version"]) is not int or value["version"] != 1:
        raise ValueError("unsupported backend profile version")
    items = value["profiles"]
    if not isinstance(items, list) or len(items) > PROFILES:
        raise ValueError("invalid backend profile inventory")
    try:
        profiles = tuple(_profile(item) for item in items)
    except TypeError as error:
        raise ValueError("invalid backend profile field type") from error
    if len({profile.identifier for profile in profiles}) != len(profiles):
        raise ValueError("duplicate backend profile identifier")
    document = Document(version=value["version"], default=value.get("default"), profiles=profiles)
    if document.default is not None:
        _get(document, document.default)
    return document


# === Private Functions === #


def _get(document: Document, identifier: str) -> Profile:
    _identifier(identifier)
    for profile in document.profiles:
        if profile.identifier == identifier:
            return profile
    raise ValueError("requested backend profile does not exist")


def _profile(value: object) -> Profile:
    value = _fields(
        value,
        {"identifier", "label", "endpoint", "expected_backend_id", "ca_certificate_pem", "secret"},
        {"identifier", "label", "endpoint", "secret"},
    )
    identifier = _identifier(value["identifier"])
    label = value["label"]
    if (
        not isinstance(label, str)
        or not label.strip()
        or len(label.encode("utf-8")) > 256
        or any(ord(char) < 32 or 127 <= ord(char) <= 159 for char in label)
    ):
        raise ValueError("invalid backend profile label")
    endpoint = value["endpoint"]
    if not isinstance(endpoint, str):
        raise TypeError("invalid backend endpoint")
    _, tls = _target(endpoint)
    identity = value.get("expected_backend_id")
    if identity is not None and (
        not isinstance(identity, str)
        or not identity
        or len(identity.encode("utf-8")) > PROFILE_ID_BYTES
        or any(ord(char) < 32 or 127 <= ord(char) <= 159 for char in identity)
    ):
        raise ValueError("invalid expected backend identity")
    ca = value.get("ca_certificate_pem")
    if ca is not None and (not isinstance(ca, str) or not ca or len(ca.encode("utf-8")) > CA_BYTES or "\0" in ca or not tls):
        raise ValueError("invalid CA certificate association")
    if ca is not None:
        import ssl

        try:
            ssl.create_default_context(cadata=ca)
        except (ssl.SSLError, ValueError):
            raise ValueError("invalid CA certificate") from None
    secret = value["secret"]
    if secret not in ("none", "keyring"):
        raise ValueError("invalid secret association")
    return Profile(
        identifier=identifier,
        label=label,
        endpoint=endpoint,
        expected_backend_id=identity,
        ca_certificate_pem=ca,
        secret=secret,
    )


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


def _identifier(value: object) -> str:
    if (
        not isinstance(value, str)
        or value in (".", "..")
        or not _IDENTIFIER.fullmatch(value)
        or len(value.encode("utf-8")) > PROFILE_IDENTIFIER_BYTES
    ):
        raise ValueError("invalid backend profile identifier")
    return value


def _environment_path(identifier: str) -> Path:
    value = os.environ.get(identifier)
    if not value:
        raise ValueError("standard configuration root is unavailable; provide an explicit profile file")
    return Path(value)


# === Entry Point === #
