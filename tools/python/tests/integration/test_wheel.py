# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Pure Wheel Release Refusals."""

# === Imports === #

# Standard Library Imports
import base64
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.check.wheel import verify_wheel

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize(
    "change",
    [
        "valid",
        "contracts",
        "descriptor",
        "nested-contracts",
        "nested-descriptor",
        "other-json",
        "tag",
        "pure",
        "native",
        "console",
        "commands",
        "traversal",
        "backslash",
        "duplicate",
        "record",
        "extra",
        "metadata",
    ],
)
def test_wheel_audit(tmp_path: Path, change: str) -> None:
    root = "dokime-0.1.0.dist-info"
    payload = {
        "dokime/__init__.py": b"# Client",
        f"{root}/METADATA": b"Metadata-Version: 2.4\nName: dokime\nVersion: 0.1.0\n",
        f"{root}/WHEEL": b"Wheel-Version: 1.0\nRoot-Is-Purelib: true\nTag: py3-none-any\n",
    }
    if change == "contracts":
        payload["dokime/contracts.json"] = b"{}"
    elif change == "descriptor":
        payload["dokime/descriptors.bin"] = b"descriptor"
    elif change == "nested-contracts":
        payload["dokime/client/contracts.json"] = b"{}"
    elif change == "nested-descriptor":
        payload["dokime/_wire/descriptors.bin"] = b"descriptor"
    elif change == "other-json":
        payload["dokime/other.json"] = b"{}"
    elif change == "pure":
        payload[f"{root}/WHEEL"] = payload[f"{root}/WHEEL"].replace(b"true", b"false")
    elif change == "native":
        payload["dokime/_native.abi3.so"] = b"native"
    elif change == "console":
        payload[f"{root}/entry_points.txt"] = (
            b"[console_scripts]\ndokime = dokime:main\n"
        )
    elif change == "commands":
        payload["dokime/client/commands/__init__.py"] = b""
    elif change == "traversal":
        payload["dokime/../outside.py"] = b""
    elif change == "backslash":
        payload["dokime/evil\\name.py"] = b""
    elif change == "extra":
        payload["other/__init__.py"] = b""
    elif change == "metadata":
        payload[f"{root}/METADATA"] += b"Name: other\n"
    rows = []
    for name, content in payload.items():
        digest = (
            base64.urlsafe_b64encode(hashlib.sha256(content).digest())
            .rstrip(b"=")
            .decode()
        )
        rows.append(f"{name},sha256={digest},{len(content)}\n")
    rows.append(f"{root}/RECORD,,\n")
    payload[f"{root}/RECORD"] = "".join(rows).encode()
    if change == "record":
        payload["dokime/__init__.py"] = b"tampered after hashing"
    tag = "cp313-abi3-linux_x86_64" if change == "tag" else "py3-none-any"
    path = tmp_path / f"dokime-0.1.0-{tag}.whl"
    with ZipFile(path, "w") as archive:
        for name, content in payload.items():
            archive.writestr(name, content)
        if change == "duplicate":
            with pytest.warns(UserWarning, match="Duplicate name"):
                archive.writestr("dokime/__init__.py", b"shadowed")
    if change in {"valid", "contracts", "descriptor"}:
        report = json.loads(verify_wheel(path)["wheel-audit"])
        assert report["sha256"] == hashlib.sha256(path.read_bytes()).hexdigest()
        assert report["pure"] is True
    else:
        with pytest.raises(ValueError):
            verify_wheel(path)


# === Private Functions === #

# === Entry Point === #
