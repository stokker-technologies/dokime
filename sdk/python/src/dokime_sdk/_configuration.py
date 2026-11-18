# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Working Directory Connection Configuration"""

# === Imports === #

# Standard Library Imports
import os
import re
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def load() -> dict[str, str]:
    path = Path.cwd() / ".env"
    try:
        content = path.read_text(encoding="utf-8-sig")
    except FileNotFoundError:
        return {}
    except UnicodeError as error:
        raise ValueError(f"Read {path}:1: Save the File as UTF-8") from error
    except OSError as error:
        raise OSError(f"Read {path}:1: Make the Configuration File Readable") from error
    return parse(content, path)


def resolve(key: str, explicit: str | None, values: dict[str, str]) -> str:
    default = "127.0.0.1:0" if key == "DOKIME_PLUGIN_ADDRESS" else ""
    value = explicit if explicit is not None else os.environ.get(key, values.get(key, default))
    if not value:
        raise ValueError(f"Configure Connection: Set {key} to a Nonempty Address")
    return value


def parse(content: str, path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for number, raw in enumerate(content.removeprefix("\ufeff").split("\n"), 1):
        line = raw.removesuffix("\r").strip(" \t")
        if "\x00" in line or "\r" in line:
            raise _malformed(path, number)
        if not line or line.startswith("#"):
            continue
        match = re.fullmatch(r"([A-Za-z_][A-Za-z0-9_]*)[ \t]*=[ \t]*(.*)", line)
        if match is None or match[1] in values:
            raise _malformed(path, number)
        key, value = match.groups()
        if not value.startswith(('"', "'")):
            values[key] = re.split(r"(?:^[#]|[ \t]+[#])", value, maxsplit=1)[0].strip(" \t")
            continue
        quote = value[0]
        decoded: list[str] = []
        index = 1
        while index < len(value) and value[index] != quote:
            character = value[index]
            if character == "\\" and quote == '"':
                index += 1
                escapes = {"\\": "\\", '"': '"', "n": "\n", "r": "\r", "t": "\t"}
                if index == len(value) or value[index] not in escapes:
                    raise _malformed(path, number)
                character = escapes[value[index]]
            decoded.append(character)
            index += 1
        if index == len(value) or not re.fullmatch(r"[ \t]*(?:#.*)?", value[index + 1 :]):
            raise _malformed(path, number)
        values[key] = "".join(decoded)
    return values


# === Private Functions === #


def _malformed(path: Path, number: int) -> ValueError:
    return ValueError(f"Read {path}:{number}: Use One Unique KEY=VALUE Assignment with Balanced Quotes and Supported Escapes")


# === Entry Point === #
