# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Bound Certificate Input Before Opening Transport"""

# === Imports === #

# Standard Library Imports
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client._transport import _certificate

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_ca_input_is_bounded(tmp_path: Path) -> None:
    path = tmp_path / "ca.pem"
    path.write_bytes(b"certificate")
    assert _certificate(path, len(b"certificate")) == b"certificate"
    with pytest.raises(ValueError):
        _certificate(path, len(b"certificate") - 1)

# === Private Functions === #

# === Entry Point === #
