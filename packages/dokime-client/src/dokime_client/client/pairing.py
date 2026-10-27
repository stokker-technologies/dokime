# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Bounded Version 1 Pairing Input And Certificate Trust Validation"""

# === Imports === #

# Standard Library Imports
import ipaddress
import json
import os
import re
import ssl
import stat
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit

from ._transport import _target

# Third Party Imports
# Local Imports
from .profiles import _fields, _object
from .secrets import _validate_key

# Type Checking Imports


# === Public Exports === #

# === Constants === #

PAIRING_BYTES = 1_048_576

# === Types === #

# === Classes === #

# === Public Functions === #


def read(path: str | Path) -> dict[str, Any]:
    source = Path(path)
    if not source.is_file():
        raise ValueError("pairing input must be a regular file")
    descriptor = os.open(source, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
    with os.fdopen(descriptor, "rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise ValueError("pairing input must be a regular file")
        data = stream.read(PAIRING_BYTES + 1)
    if len(data) > PAIRING_BYTES:
        raise ValueError("pairing input exceeds 1 MiB")
    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=_object)
    except (UnicodeError, json.JSONDecodeError):
        raise ValueError("invalid pairing JSON") from None
    fields = {"version", "endpoint", "backend_identifier", "label", "ca_certificate_pem", "access_key"}
    value = _fields(value, fields, fields)
    if type(value["version"]) is not int or value["version"] != 1:
        raise ValueError("unsupported pairing version")
    identifier = value["backend_identifier"]
    if not isinstance(identifier, str) or not identifier.strip() or len(identifier.encode("utf-8")) > 128:
        raise ValueError("invalid backend identity")
    if any(ord(char) < 32 or 127 <= ord(char) <= 159 for char in identifier):
        raise ValueError("invalid backend identity")
    label = value["label"]
    if (
        not isinstance(label, str)
        or not label.strip()
        or len(label.encode("utf-8")) > 256
        or any(ord(char) < 32 or 127 <= ord(char) <= 159 for char in label)
    ):
        raise ValueError("invalid pairing label")
    endpoint = value["endpoint"]
    if not isinstance(endpoint, str) or not endpoint:
        raise ValueError("invalid pairing endpoint")
    _, tls = _target(endpoint)
    origin = urlsplit(endpoint)
    host = origin.hostname
    if origin.port is None or origin.port == 0 or origin.path or any(char in endpoint for char in ("%", "\\")):
        raise ValueError("pairing endpoint requires an explicit port and no path")
    try:
        address = ipaddress.ip_address(host)
    except ValueError:
        if (
            not host
            or len(host) > 253
            or not all(re.fullmatch(r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?", part) for part in host.split("."))
        ):
            raise ValueError("invalid pairing endpoint host") from None
    else:
        if address.is_unspecified or address.is_multicast:
            raise ValueError("pairing endpoint must be concrete")
    ca = value["ca_certificate_pem"]
    if tls:
        if not isinstance(ca, str) or not ca or len(ca.encode("utf-8")) > 262_144:
            raise ValueError("HTTPS pairing requires a CA certificate")
        try:
            ssl.create_default_context(cadata=ca)
        except (ssl.SSLError, ValueError):
            raise ValueError("invalid pairing CA certificate") from None
    elif ca is not None:
        raise ValueError("HTTP pairing cannot include a CA certificate")
    key = value["access_key"]
    _validate_key(key)
    return value


# === Private Functions === #

# === Entry Point === #
