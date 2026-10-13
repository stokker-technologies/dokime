# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Audit The Prepared Pure Python Client Wheel."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations

import base64
import csv
import hashlib
import io
import json
import stat
from email.message import Message
from email.parser import Parser
from pathlib import Path, PurePosixPath
from zipfile import ZipFile

# Third Party Imports
from packaging.utils import parse_wheel_filename

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def verify_wheel(path: Path) -> dict[str, str]:
    name, version, build, tags = parse_wheel_filename(path.name)
    if name != "dokime" or build or {str(tag) for tag in tags} != {"py3-none-any"}:
        raise ValueError("client wheel requires the dokime py3-none-any tag")
    metadata_root = f"dokime-{version}.dist-info"
    with ZipFile(path) as archive:
        entries = archive.infolist()
        names = [entry.filename for entry in entries]
        if len(names) != len(set(names)):
            raise ValueError("duplicate client wheel entries")
        for entry in entries:
            relative = PurePosixPath(entry.filename)
            mode = entry.external_attr >> 16
            if (
                entry.is_dir()
                or not relative.parts
                or relative.is_absolute()
                or relative.as_posix() != entry.filename
                or ".." in relative.parts
                or "\\" in entry.filename
                or stat.S_ISLNK(mode)
                or (stat.S_IFMT(mode) and not stat.S_ISREG(mode))
                or entry.flag_bits & 1
            ):
                raise ValueError("invalid client wheel entry")
            if relative.parts[0] not in {"dokime", metadata_root}:
                raise ValueError("unexpected client wheel package")
            if relative.parts[0] == "dokime" and (
                relative.suffix not in {".py", ".pyi"}
                and relative.name != "py.typed"
                and entry.filename
                not in {"dokime/contracts.json", "dokime/descriptors.bin"}
            ):
                raise ValueError("unexpected client wheel source payload")
            if (
                relative.suffix.lower()
                in {".so", ".pyd", ".dll", ".dylib", ".exe", ".a", ".lib"}
                or relative.parts[:2] in {("dokime", "_native"), ("dokime", "core")}
                or entry.filename
                in {"dokime/client/cli.py", "dokime/client/_console.py"}
                or relative.parts[:3] == ("dokime", "client", "commands")
            ):
                raise ValueError("native or obsolete client wheel payload")
        wheel = _metadata(archive, f"{metadata_root}/WHEEL")
        metadata = _metadata(archive, f"{metadata_root}/METADATA")
        if (
            wheel.get_all("Root-Is-Purelib") != ["true"]
            or wheel.get_all("Tag") != ["py3-none-any"]
            or wheel.get_all("Wheel-Version") != ["1.0"]
            or metadata.get_all("Name") != ["dokime"]
            or metadata.get_all("Version") != [str(version)]
            or f"{metadata_root}/entry_points.txt" in names
        ):
            raise ValueError(
                "client wheel metadata or entry points disagree with purity"
            )
        _record(archive, metadata_root)
        if archive.testzip() is not None:
            raise ValueError("client wheel checksum failure")
    report = {
        "version": 1,
        "wheel": path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "tag": "py3-none-any",
        "pure": True,
        "files": len(entries),
        "native_payload": False,
        "entry_points": False,
    }
    return {"wheel-audit": json.dumps(report, sort_keys=True) + "\n"}


# === Private Functions === #


def _metadata(archive: ZipFile, name: str) -> Message:
    if name not in archive.namelist() or archive.getinfo(name).file_size > 1_048_576:
        raise ValueError("missing or excessive client wheel metadata")
    return Parser().parsestr(archive.read(name).decode("utf-8"))


def _record(archive: ZipFile, metadata_root: str) -> None:
    name = f"{metadata_root}/RECORD"
    if name not in archive.namelist() or archive.getinfo(name).file_size > 1_048_576:
        raise ValueError("missing or excessive client wheel RECORD")
    rows = list(csv.reader(io.StringIO(archive.read(name).decode("utf-8"))))
    if any(len(row) != 3 for row in rows):
        raise ValueError("invalid client wheel RECORD row")
    if len(rows) != len({row[0] for row in rows}):
        raise ValueError("duplicate client wheel RECORD entries")
    if {row[0] for row in rows} != set(archive.namelist()):
        raise ValueError("client wheel RECORD inventory mismatch")
    for filename, checksum, size in rows:
        if filename == name:
            if checksum or size:
                raise ValueError("client wheel RECORD must leave its own hash empty")
            continue
        content = archive.read(filename)
        digest = (
            base64.urlsafe_b64encode(hashlib.sha256(content).digest())
            .rstrip(b"=")
            .decode("ascii")
        )
        if checksum != f"sha256={digest}" or size != str(len(content)):
            raise ValueError("client wheel RECORD content mismatch")


# === Entry Point === #
