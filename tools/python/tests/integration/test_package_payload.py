# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Pure Wheel Source Payloads"""

# === Imports === #

# Standard Library Imports
from email.message import Message
from pathlib import Path
from zipfile import ZipFile

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.check.package import (
    _verify_required_dependencies,
    _verify_wheel_payload,
)

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_wheel_exactly(tmp_path: Path) -> None:
    payload = _sources(tmp_path)
    wheel = _wheel(tmp_path, payload)

    _verify_wheel_payload(wheel, tmp_path)


@pytest.mark.parametrize(
    "change",
    [
        "missing",
        "extra",
        "stale",
        "old-package",
        "missing-descriptors",
        "stale-contracts",
    ],
)
def test_wheel_sources(tmp_path: Path, change: str) -> None:
    payload = _sources(tmp_path)
    message = "inventory mismatch"
    if change == "missing":
        del payload["dokime/client/api.pyi"]
    elif change == "extra":
        payload["dokime/removed.py"] = b"obsolete API"
    elif change == "stale":
        payload["dokime/client/api.pyi"] = b"old API"
        message = "Stale wheel source"
    elif change == "missing-descriptors":
        del payload["dokime/descriptors.bin"]
    elif change == "stale-contracts":
        payload["dokime/contracts.json"] = b"old contracts"
        message = "Stale wheel source"
    else:
        payload["dokime_client/__init__.py"] = b""
        message = "Obsolete dokime_client"
    wheel = _wheel(tmp_path, payload)

    with pytest.raises(SystemExit, match=message):
        _verify_wheel_payload(wheel, tmp_path)


@pytest.mark.parametrize("change", ["empty", "ambiguous", "wrong-module"])
def test_wheel_extension(tmp_path: Path, change: str) -> None:
    payload = _sources(tmp_path)
    if change == "empty":
        payload["dokime/_native.abi3.so"] = b""
    elif change == "ambiguous":
        payload["dokime/_native.abi3.so"] = b"native"
        payload["dokime/_native.pyd"] = b"native"
    elif change == "wrong-module":
        payload["dokime/other.abi3.so"] = b"native"
    wheel = _wheel(tmp_path, payload)

    with pytest.raises(SystemExit, match="Native payload"):
        _verify_wheel_payload(wheel, tmp_path)


def test_wheel_entries(tmp_path: Path) -> None:
    payload = _sources(tmp_path)
    payload["dokime/_native.abi3.so"] = b"native"
    wheel = _wheel(tmp_path, payload)
    with (
        ZipFile(wheel, "a") as archive,
        pytest.warns(UserWarning, match="Duplicate name"),
    ):
        archive.writestr("dokime/__init__.py", b"shadowed")

    with pytest.raises(SystemExit, match="Duplicate wheel entries"):
        _verify_wheel_payload(wheel, tmp_path)


@pytest.mark.parametrize(
    "change", ["valid", "missing", "weaker", "marker", "extra", "duplicate"]
)
def test_runtime_dependencies(change: str) -> None:
    expected = ("grpcio>=1.84.0,<2", "protobuf>=7.36.2,<8")
    values = ["grpcio<2,>=1.84.0", "protobuf<8,>=7.36.2"]
    if change == "missing":
        values.pop()
    elif change == "weaker":
        values[0] = "grpcio>=1"
    elif change == "marker":
        values[0] += '; sys_platform == "linux"'
    elif change == "extra":
        values.append("maturin>=1")
    elif change == "duplicate":
        values.append(values[0])
    metadata = Message()
    for value in values:
        metadata["Requires-Dist"] = value
    if change == "valid":
        _verify_required_dependencies(metadata, "dokime", "0.1.0", expected)
    else:
        with pytest.raises(SystemExit, match="dependency inventory mismatch"):
            _verify_required_dependencies(metadata, "dokime", "0.1.0", expected)


# === Private Functions === #


def _sources(root: Path) -> dict[str, bytes]:
    payload = {
        "dokime/__init__.py": b"# Public API",
        "dokime/client/api.pyi": b"# Typed client",
        "dokime/py.typed": b"",
        "dokime/contracts.json": b'{"operations": []}',
        "dokime/descriptors.bin": b"canonical descriptors",
    }
    for name, content in payload.items():
        path = root / "packages/dokime/src" / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
    return payload


def _wheel(root: Path, payload: dict[str, bytes]) -> Path:
    wheel = root / "dokime-0.1.0-fixture.whl"
    with ZipFile(wheel, "w") as archive:
        for name, content in payload.items():
            archive.writestr(name, content)
    return wheel


# === Entry Point === #
