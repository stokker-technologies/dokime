# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Resolve Explicit Remote Credentials Without Changing Process Environment"""

# === Imports === #

# Standard Library Imports
import io
import os
import re
import stat
from pathlib import Path

# Third Party Imports
from dotenv.main import DotEnv
from dotenv.parser import parse_stream

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #
_ENVIRONMENT_BYTES = 64 * 1024
_KEY_BYTES = 4098
_NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_.]*\Z")

# === Types === #

# === Classes === #

# === Public Functions === #


def load_key(*, key_file: str | Path | None = None, env_file: str | Path | None = None) -> str | None:
    """Validate explicit files, then select key file, process key or dotenv key."""
    values = _environment(env_file) if env_file is not None else {}
    if key_file is not None:
        key = _read(Path(key_file), _KEY_BYTES).strip()
    elif "DOKIME_KEY" in os.environ:
        key = os.environ["DOKIME_KEY"]
    else:
        key = values.get("DOKIME_KEY")
    if key is not None and (not 32 <= len(key) <= 4096 or not all(33 <= ord(character) <= 126 for character in key)):
        raise ValueError("Remote key must contain 32–4096 visible ASCII characters.")
    return key


# === Private Functions === #


def _read(path: Path, limit: int) -> str:
    if not path.is_file():
        raise ValueError("The selected credential input must be a regular file.")
    with path.open("rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise ValueError("The selected credential input must be a regular file.")
        data = stream.read(limit + 1)
    if len(data) > limit:
        raise ValueError("The selected credential input exceeds its byte limit.")
    try:
        return data.decode("utf-8")
    except UnicodeError:
        raise ValueError("The selected credential input must be UTF-8 text.") from None


def _environment(path: str | Path) -> dict[str, str]:
    text = _read(Path(path), _ENVIRONMENT_BYTES)
    if "\0" in text:
        raise ValueError("The selected dotenv file contains an invalid entry.")
    # Validate Before Parsing Values; Default Dotenv Logging Tolerates Errors.
    for binding in parse_stream(io.StringIO(text)):
        if binding.error or binding.key is not None and (not _NAME.fullmatch(binding.key) or binding.value is None):
            raise ValueError(f"The selected dotenv file contains an invalid entry at line {binding.original.line}.")
    values = DotEnv(dotenv_path=None, stream=io.StringIO(text), override=False).dict()
    return {name: value for name, value in values.items() if value is not None}


# === Entry Point === #
