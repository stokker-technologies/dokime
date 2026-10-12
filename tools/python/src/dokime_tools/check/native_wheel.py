# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Native Python Wheel Release Contracts."""

# === Imports === #

# Standard Library Imports
import json
import re
import struct
import subprocess
import sys
from email.parser import Parser
from pathlib import Path
from zipfile import ZipFile

# Third Party Imports
from packaging.tags import parse_tag
from packaging.utils import parse_wheel_filename

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

GLIBC_FLOOR = (2, 28)

# === Types === #

# === Classes === #

# === Public Functions === #


def verify_native_wheel(path: Path) -> dict[str, str]:
    """Audit the exact release wheel on its native build platform."""
    target = _verify_tags_and_binary(path)
    expected = {"linux": "linux", "win32": "windows"}.get(sys.platform)
    if target != expected:
        raise SystemExit(f"Audit {path.name} on its native {target} host.")

    abi3 = subprocess.run(
        [sys.executable, "-m", "abi3audit", "--strict", "--report", str(path)],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    reports = {"abi3-audit": abi3.stdout}
    print(abi3.stdout, end="")
    if target == "linux":
        audit = subprocess.run(
            [sys.executable, "-m", "auditwheel", "show", "--json", str(path)],
            check=True,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
        report = json.loads(audit.stdout)
        _verify_linux_report(report, path)
        print(audit.stdout, end="")
        reports["linux-audit"] = audit.stdout
    return reports


# === Private Functions === #


def _linux_floor(tag: str) -> tuple[int, int] | None:
    match = re.fullmatch(r"manylinux_(\d+)_(\d+)_x86_64", tag)
    return (int(match[1]), int(match[2])) if match else None


def _verify_tags_and_binary(path: Path) -> str:
    name, _, _, tags = parse_wheel_filename(path.name)
    if name != "dokime" or any(
        tag.interpreter != "cp313" or tag.abi != "abi3" for tag in tags
    ):
        raise SystemExit("Release wheels require dokime CPython 3.13 ABI3 tags.")
    platforms = {tag.platform for tag in tags}
    if platforms == {"win_amd64"}:
        target = "windows"
    elif platforms and all(
        (floor := _linux_floor(platform)) is not None and floor <= GLIBC_FLOOR
        for platform in platforms
    ):
        target = "linux"
    else:
        raise SystemExit("Release wheels require x86_64 Windows or manylinux <= 2.28.")

    with ZipFile(path) as wheel:
        names = wheel.namelist()
        metadata = [name for name in names if name.endswith(".dist-info/WHEEL")]
        if len(names) != len(set(names)) or len(metadata) != 1:
            raise SystemExit(
                "Expected unique wheel entries and one WHEEL metadata file."
            )
        message = Parser().parsestr(wheel.read(metadata[0]).decode())
        declared = {
            tag for value in message.get_all("Tag", []) for tag in parse_tag(value)
        }
        if declared != tags or message.get("Root-Is-Purelib", "").lower() != "false":
            raise SystemExit("WHEEL metadata must match the native filename tags.")
        extensions = [
            name
            for name in names
            if name.startswith("dokime/_native.") and name.endswith((".so", ".pyd"))
        ]
        expected_extension = (
            "dokime/_native.abi3.so" if target == "linux" else "dokime/_native.pyd"
        )
        if extensions != [expected_extension]:
            raise SystemExit("Expected exactly the platform's ABI3 native extension.")
        _verify_binary(wheel.read(expected_extension), target)

    return target


def _verify_binary(binary: bytes, target: str) -> None:
    if target == "linux":
        valid = (
            len(binary) >= 64
            and binary[:7] == b"\x7fELF\x02\x01\x01"
            and struct.unpack_from("<HH", binary, 16) == (3, 62)
        )
    else:
        valid = False
        if len(binary) >= 64 and binary[:2] == b"MZ":
            offset = struct.unpack_from("<I", binary, 60)[0]
            if offset <= len(binary) - 26 and binary[offset : offset + 4] == b"PE\0\0":
                machine = struct.unpack_from("<H", binary, offset + 4)[0]
                flags = struct.unpack_from("<H", binary, offset + 22)[0]
                magic = struct.unpack_from("<H", binary, offset + 24)[0]
                valid = machine == 0x8664 and bool(flags & 0x2000) and magic == 0x20B
    if not valid:
        raise SystemExit(f"Native extension is not a {target} x86_64 shared library.")


def _verify_linux_report(report: dict, path: Path) -> None:
    floor = _linux_floor(str(report.get("overall_tag", "")))
    if (
        report.get("version") != 1
        or report.get("wheel") != path.name
        or report.get("pure") is not False
        or floor is None
        or floor > GLIBC_FLOOR
        or report.get("unsupported_isa") is not False
    ):
        raise SystemExit(
            "Native dependencies or ISA exceed the manylinux_2_28_x86_64 release contract."
        )
    # An older filename tag also has to be true, even if 2.28 would pass.
    _, _, _, tags = parse_wheel_filename(path.name)
    if any(
        (claimed := _linux_floor(tag.platform)) is None or floor > claimed
        for tag in tags
    ):
        raise SystemExit("Native dependencies exceed the wheel's claimed Linux floor.")


# === Entry Point === #
