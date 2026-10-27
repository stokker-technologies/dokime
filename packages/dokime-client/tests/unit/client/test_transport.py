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
import json
from pathlib import Path

# Third Party Imports
import pytest

from dokime_client.client import DokimeClient

# Local Imports
from dokime_client.client._transport import Transport
from dokime_client.client.pairing import read

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_ca_input_is_bounded(monkeypatch) -> None:
    monkeypatch.setattr("dokime_client.client._transport.CA_BYTES", 11)
    monkeypatch.setattr("dokime_client.client._transport.ssl.create_default_context", lambda **kwargs: None)
    transport = Transport("https://localhost:8765", None, 30, connect_timeout_ms=1000, ca_certificate_pem="certificate")
    assert transport._ca == b"certificate"
    with pytest.raises(ValueError, match="byte limit"):
        Transport("https://localhost:8765", None, 30, connect_timeout_ms=1000, ca_certificate_pem="certificate!")


@pytest.mark.parametrize(
    "case",
    json.loads((Path(__file__).resolve().parents[5] / "crates/dokime-client/tests/fixtures/pairing.json").read_text()),
    ids=lambda case: case["name"],
)
def test_pairing_conformance(case, tmp_path, monkeypatch):
    monkeypatch.setenv("DOKIME_KEY", "ambient-key-must-not-enter-pairing-client")
    path = tmp_path / "lab.dokime-pair"
    path.write_text(case["document"], encoding="utf-8")
    if not case["valid"]:
        with pytest.raises(ValueError):
            DokimeClient.from_pairing(path)
        return
    pairing = read(path)
    client = DokimeClient.from_pairing(path)
    assert client._transport._channel is None
    assert client._transport.backend is None
    assert client._transport._expected_backend_id == pairing["backend_identifier"]
    expected = (("authorization", "Bearer " + pairing["access_key"]),) if pairing["access_key"] else ()
    assert tuple(client._transport._metadata) == expected
    assert str(pairing["access_key"]) not in repr(client)


def test_pairing_read_bound_and_utf8(tmp_path):
    path = tmp_path / "lab.dokime-pair"
    for data in [b" " * (1024 * 1024 + 1), b"\xff"]:
        path.write_bytes(data)
        with pytest.raises(ValueError):
            DokimeClient.from_pairing(path)


# === Private Functions === #

# === Entry Point === #
