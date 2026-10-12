# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Native Wheel Source Payloads"""

# === Imports === #

# Standard Library Imports
from pathlib import Path
from zipfile import ZipFile

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.check.package import _verify_wheel_payload

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("extension", ["_native.abi3.so", "_native.pyd"])
def test_wheel_exactly(tmp_path: Path, extension: str) -> None:
    payload = _sources(tmp_path)
    payload[f"dokime/{extension}"] = b"native binary fixture"
    wheel = _wheel(tmp_path, payload)

    _verify_wheel_payload(wheel, tmp_path)


@pytest.mark.parametrize("change", ["missing", "extra", "stale", "old-package"])
def test_wheel_sources(tmp_path: Path, change: str) -> None:
    payload = _sources(tmp_path)
    payload["dokime/_native.abi3.so"] = b"native binary fixture"
    message = "inventory mismatch"
    if change == "missing":
        del payload["dokime/client/api.pyi"]
    elif change == "extra":
        payload["dokime/removed.py"] = b"obsolete API"
    elif change == "stale":
        payload["dokime/client/api.pyi"] = b"old API"
        message = "Stale wheel source"
    else:
        payload["dokime_client/__init__.py"] = b""
        message = "Obsolete dokime_client"
    wheel = _wheel(tmp_path, payload)

    with pytest.raises(SystemExit, match=message):
        _verify_wheel_payload(wheel, tmp_path)


@pytest.mark.parametrize("change", ["missing", "empty", "ambiguous", "wrong-module"])
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

    with pytest.raises(SystemExit, match="one nonempty native extension"):
        _verify_wheel_payload(wheel, tmp_path)


def test_wheel_entries(tmp_path: Path) -> None:
    payload = _sources(tmp_path)
    payload["dokime/_native.abi3.so"] = b"native"
    wheel = _wheel(tmp_path, payload)
    with ZipFile(wheel, "a") as archive:
        with pytest.warns(UserWarning, match="Duplicate name"):
            archive.writestr("dokime/__init__.py", b"shadowed")

    with pytest.raises(SystemExit, match="Duplicate wheel entries"):
        _verify_wheel_payload(wheel, tmp_path)


# === Private Functions === #


def _sources(root: Path) -> dict[str, bytes]:
    payload = {
        "dokime/__init__.py": b"# Public API",
        "dokime/client/api.pyi": b"# Typed client",
        "dokime/py.typed": b"",
    }
    for name, content in payload.items():
        path = root / "crates/dokime-python/python" / name
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
