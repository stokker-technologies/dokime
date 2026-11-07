# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Preserve Python Values Across Portable Conversion"""

# === Imports === #

# Standard Library Imports
from dataclasses import dataclass
from enum import Enum

# Third Party Imports
import pytest

# Local Imports
from dokime.authoring import RegistrationError, _decode, _plain, _shape

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_portable_values_preserve_nested_optional_binary_and_enum() -> None:
    class Mode(Enum):
        READY = 1
        STOPPED = 2

    @dataclass
    class Reading:
        count: int
        payload: bytes | None
        modes: list[Mode]

    original = Reading(2**53 + 1, b"\x00\xff", [Mode.READY, Mode.STOPPED])
    encoded = _plain(original)
    decoded = _decode(Reading, encoded)
    assert decoded == original
    decoded.modes.clear()
    assert original.modes == [Mode.READY, Mode.STOPPED]
    assert _decode(Reading, {"count": -1, "payload": None, "modes": []}) == Reading(-1, None, [])


def test_portable_integer_refuses_fraction_and_schema_refuses_unknown_type() -> None:
    with pytest.raises(ValueError):
        _decode(int, 1.25)
    assert _decode(int, 17.0) == 17
    with pytest.raises(RegistrationError):
        _shape(object)


# === Private Functions === #

# === Entry Point === #
