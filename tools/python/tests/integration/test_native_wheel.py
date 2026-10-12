# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Native Wheel Release Refusals."""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable
import struct
from pathlib import Path
from unittest.mock import patch
from zipfile import ZipFile

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.check.native_wheel import (
    _verify_linux_report,
    _verify_tags_and_binary,
    verify_native_wheel,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

LINUX_TAG = "cp313-abi3-manylinux_2_28_x86_64"
WINDOWS_TAG = "cp313-abi3-win_amd64"

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("target", ["linux", "windows"])
def test_native_architecture(tmp_path: Path, target: str) -> None:
    wheel = _wheel(tmp_path, target)
    assert _verify_tags_and_binary(wheel) == target


@pytest.mark.parametrize("target", ["linux", "windows"])
@pytest.mark.parametrize("damage", ["arch", "truncated", "executable"])
def test_native_headers(tmp_path: Path, target: str, damage: str) -> None:
    binary = bytearray(_binary(target))
    if damage == "arch":
        struct.pack_into("<H", binary, 18 if target == "linux" else 68, 0xAA64)
    elif damage == "executable":
        struct.pack_into("<H", binary, 16 if target == "linux" else 86, 2)
    else:
        binary = binary[:30]
    wheel = _wheel(tmp_path, target, binary=bytes(binary))
    with pytest.raises(SystemExit, match="shared library"):
        _verify_tags_and_binary(wheel)


@pytest.mark.parametrize(
    "tag",
    [
        "cp313-cp313-manylinux_2_28_x86_64",
        "cp312-abi3-manylinux_2_28_x86_64",
        "cp313-abi3-linux_x86_64",
        "cp313-abi3-manylinux_2_39_x86_64",
        "cp313-abi3-manylinux_2_28_aarch64",
        "cp313-abi3-win32",
        "py3-none-any",
    ],
)
def test_release_tags(tmp_path: Path, tag: str) -> None:
    wheel = _wheel(tmp_path, "linux", tag=tag)
    with pytest.raises(SystemExit, match="Release wheels require"):
        _verify_tags_and_binary(wheel)


@pytest.mark.parametrize(
    "metadata_tag,pure", [(WINDOWS_TAG, "false"), (LINUX_TAG, "true")]
)
def test_wheel_disagree(tmp_path: Path, metadata_tag: str, pure: str) -> None:
    wheel = _wheel(tmp_path, "linux", metadata_tag=metadata_tag, pure=pure)
    with pytest.raises(SystemExit, match="metadata must match"):
        _verify_tags_and_binary(wheel)


@pytest.mark.parametrize("target", ["linux", "windows"])
def test_audit_platform(tmp_path: Path, target: str) -> None:
    wheel = _wheel(tmp_path, target)
    host = "win32" if target == "linux" else "linux"
    with patch("dokime_tools.check.native_wheel.sys.platform", host):
        with pytest.raises(SystemExit, match="native .* host"):
            verify_native_wheel(wheel)


@pytest.mark.parametrize(
    "field,value",
    [
        ("version", 2),
        ("wheel", "other.whl"),
        ("pure", True),
        ("overall_tag", "linux_x86_64"),
        ("overall_tag", "manylinux_2_39_x86_64"),
        ("overall_tag", "manylinux_2_28_aarch64"),
        ("unsupported_isa", True),
    ],
)
def test_linux_report(field: str, value: object) -> None:
    path = Path(f"dokime-0.1.0-{LINUX_TAG}.whl")
    report = _report(path)
    report[field] = value
    with pytest.raises(SystemExit, match="release contract"):
        _verify_linux_report(report, path)


def case_linux_portability() -> None:
    path = Path("dokime-0.1.0-cp313-abi3-manylinux_2_17_x86_64.whl")
    with pytest.raises(SystemExit, match="claimed Linux floor"):
        _verify_linux_report(_report(path), path)


def case_linux_dependencies() -> None:
    path = Path(f"dokime-0.1.0-{LINUX_TAG}.whl")
    report = _report(path)
    report["overall_tag"] = "manylinux_2_17_x86_64"
    _verify_linux_report(report, path)


# === Private Functions === #


def _binary(target: str) -> bytes:
    binary = bytearray(128)
    if target == "linux":
        binary[:7] = b"\x7fELF\x02\x01\x01"
        struct.pack_into("<HH", binary, 16, 3, 62)
    else:
        binary[:2] = b"MZ"
        struct.pack_into("<I", binary, 60, 64)
        binary[64:68] = b"PE\0\0"
        struct.pack_into("<H", binary, 68, 0x8664)
        struct.pack_into("<H", binary, 86, 0x2000)
        struct.pack_into("<H", binary, 88, 0x20B)
    return bytes(binary)


def _wheel(
    root: Path,
    target: str,
    *,
    binary: bytes | None = None,
    tag: str | None = None,
    metadata_tag: str | None = None,
    pure: str = "false",
) -> Path:
    tag = tag or (LINUX_TAG if target == "linux" else WINDOWS_TAG)
    path = root / f"dokime-0.1.0-{tag}.whl"
    with ZipFile(path, "w") as wheel:
        wheel.writestr(
            "dokime-0.1.0.dist-info/WHEEL",
            f"Wheel-Version: 1.0\nRoot-Is-Purelib: {pure}\nTag: {metadata_tag or tag}\n",
        )
        wheel.writestr(
            "dokime/_native.abi3.so" if target == "linux" else "dokime/_native.pyd",
            binary if binary is not None else _binary(target),
        )
    return path


def _report(path: Path) -> dict[str, object]:
    return {
        "version": 1,
        "wheel": path.name,
        "pure": False,
        "overall_tag": "manylinux_2_28_x86_64",
        "unsupported_isa": False,
    }


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_linux_portability, id="linux_portability"),
        pytest.param(case_linux_dependencies, id="linux_dependencies"),
    ],
)
def test_contract(scenario: Callable[..., None]) -> None:
    scenario()
