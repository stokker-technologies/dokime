# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Shared Profile Conformance And Explicit Selection Scenarios"""

import json
from pathlib import Path

import pytest

from dokime_client.client import DokimeClient
from dokime_client.client.profiles import configuration_path, decode, load

_CASES = json.loads((Path(__file__).resolve().parents[5] / "crates/dokime-client/tests/fixtures/profiles.json").read_text())


@pytest.mark.parametrize("case", _CASES, ids=lambda case: case["name"])
def test_profile_documents(case):
    if case["valid"]:
        decode(case["document"].encode())
    else:
        with pytest.raises(ValueError):
            decode(case["document"].encode())


def test_named_profile_selection(tmp_path, monkeypatch):
    path = tmp_path / "backends.json"
    path.write_text(_CASES[0]["document"])
    monkeypatch.setenv("DOKIME_BACKEND", "unknown")
    monkeypatch.setenv("DOKIME_PROFILES_FILE", str(path))

    # A Named Profile Ignores The Default Selection And Opens No Channel
    profile = load("local")
    client = DokimeClient.from_profile("local")
    assert client._transport.backend is None
    assert client._transport._channel is None
    assert client._transport._expected_backend_id == "machine-1"

    # Loaded Profiles Are Snapshots Of The Document
    path.write_text('{"version":1,"profiles":[]}')
    assert profile.expected_backend_id == "machine-1"
    assert client._transport._expected_backend_id == "machine-1"
    with pytest.raises(ValueError):
        load("local")


def test_profile_secret_supplied(tmp_path, monkeypatch):
    path = tmp_path / "backends.json"
    path.write_text(_CASES[2]["document"])
    monkeypatch.setenv("DOKIME_PROFILES_FILE", str(path))
    with pytest.raises(ValueError, match="pairing"):
        DokimeClient.from_profile("local")


def test_profile_explicit_path(tmp_path, monkeypatch):
    path = tmp_path / "config.json"
    monkeypatch.setenv("DOKIME_PROFILES_FILE", "")
    assert configuration_path(path) == path
    with pytest.raises(ValueError):
        configuration_path()


def test_profile_read_bound(tmp_path):
    path = tmp_path / "config.json"
    path.write_bytes(b" " * (1024 * 1024 + 1))
    with pytest.raises(ValueError, match="exceeds"):
        load("local", path=path)
