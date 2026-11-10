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
from dokime import connect
from dokime.client import DokimeClient

# Local Imports
from dokime.client.pairing import read

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize(
    "case",
    json.loads((Path(__file__).resolve().parents[5] / "crates/dokime-client/tests/fixtures/pairing.json").read_text()),
    ids=lambda case: case["name"],
)
def test_pairing_conformance(case, tmp_path, monkeypatch):
    monkeypatch.setenv("DOKIME_KEY", "ambient-key-must-not-enter-pairing-client")
    path = tmp_path / "lab.pair"
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
    assert bool(client._transport._access_key) == bool(expected)
    assert str(pairing["access_key"]) not in repr(client)


def test_pairing_read_bound_and_utf8(tmp_path):
    path = tmp_path / "lab.pair"
    for data in [b" " * (1024 * 1024 + 1), b"\xff"]:
        path.write_bytes(data)
        with pytest.raises(ValueError):
            DokimeClient.from_pairing(path)


def test_pairing_environment_and_explicit_precedence(tmp_path, monkeypatch):
    file = tmp_path / "environment.pair"
    value = {
        "version": 1,
        "endpoints": ["http://127.0.0.1:8765"],
        "backend_identifier": "environment-backend",
        "label": "Lab",
        "certificate_pem": None,
        "access_key": None,
    }
    file.write_text(json.dumps(value), encoding="utf-8")
    monkeypatch.setenv("DOKIME_REMOTE", str(file))
    assert read()["backend_identifier"] == "environment-backend"
    assert connect()._transport._expected_backend_id == "environment-backend"
    explicit = tmp_path / "explicit.pair"
    explicit.write_text(json.dumps({**value, "backend_identifier": "explicit-backend"}), encoding="utf-8")
    assert read(explicit)["backend_identifier"] == "explicit-backend"
    assert connect(explicit)._transport._expected_backend_id == "explicit-backend"
    monkeypatch.delenv("DOKIME_REMOTE")
    with pytest.raises(ValueError):
        read()


# === Private Functions === #

# === Entry Point === #
