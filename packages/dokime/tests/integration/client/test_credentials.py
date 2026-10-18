# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Credential Precedence And Preflight Without A Connection"""

# === Imports === #

# Standard Library Imports
import json
import os
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime.client import DokimeClient
from dokime.client.credentials import _environment, load_key

# Type Checking Imports


# === Public Exports === #

# === Constants === #
_CASES = json.loads((Path(__file__).resolve().parents[5] / "crates/dokime-client/tests/fixtures/credentials.json").read_text())

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("case", _CASES, ids=lambda case: case["name"])
def test_dotenv_contract(case, tmp_path):
    path = tmp_path / "selected.env"
    path.write_text(case["document"], encoding="utf-8")
    if case["valid"]:
        assert _environment(path).get("DOKIME_KEY") == case["key"]
        return
    with pytest.raises(ValueError) as error:
        _environment(path)
    assert "private" not in str(error.value)


def test_credential_precedence(tmp_path, monkeypatch):
    key_file = tmp_path / "remote.key"
    env_file = tmp_path / "selected.env"
    key_file.write_text("a" * 32 + "\r\n", encoding="utf-8")
    env_file.write_text("DOKIME_KEY=" + "c" * 32 + "\n", encoding="utf-8")
    monkeypatch.setenv("DOKIME_KEY", "b" * 32)
    assert load_key(key_file=key_file, env_file=env_file) == "a" * 32
    assert load_key(env_file=env_file) == "b" * 32
    assert load_key() == "b" * 32
    client = DokimeClient(key_file=key_file, env_file=env_file)
    assert client._transport._channel is None
    assert client._transport._metadata == (("authorization", "Bearer " + "a" * 32),)
    monkeypatch.delenv("DOKIME_KEY")
    assert load_key(env_file=env_file) == "c" * 32
    assert "DOKIME_KEY" not in os.environ


@pytest.mark.parametrize("data", [b"", b"short", b"\xff", b"a" * 4099, b"a" * 31 + b" "])
def test_bad_key_file_never_uses_environment(data, tmp_path, monkeypatch):
    key_file = tmp_path / "remote.key"
    key_file.write_bytes(data)
    monkeypatch.setenv("DOKIME_KEY", "b" * 32)
    with pytest.raises(ValueError):
        DokimeClient(key_file=key_file)


def test_all_explicit_files_are_validated(tmp_path, monkeypatch):
    key_file = tmp_path / "remote.key"
    env_file = tmp_path / "selected.env"
    key_file.write_text("a" * 32, encoding="utf-8")
    env_file.write_text("invalid private entry", encoding="utf-8")
    monkeypatch.setenv("DOKIME_KEY", "b" * 32)
    with pytest.raises(ValueError) as error:
        DokimeClient(key_file=key_file, env_file=env_file)
    assert "private" not in str(error.value)
    with pytest.raises((OSError, ValueError)):
        DokimeClient(key_file=tmp_path / "missing.key")
    with pytest.raises((OSError, ValueError)):
        DokimeClient(env_file=tmp_path / "missing.env")
    with pytest.raises(ValueError):
        DokimeClient(env_file=tmp_path)
    env_file.write_bytes(b"x" * (64 * 1024 + 1))
    with pytest.raises(ValueError):
        DokimeClient(key_file=key_file, env_file=env_file)


def test_no_implicit_dotenv_search(tmp_path, monkeypatch):
    monkeypatch.delenv("DOKIME_KEY", raising=False)
    (tmp_path / ".env").write_text("DOKIME_KEY=" + "c" * 32, encoding="utf-8")
    nested = tmp_path / "nested"
    nested.mkdir()
    monkeypatch.chdir(nested)
    assert load_key() is None


# === Private Functions === #

# === Entry Point === #
