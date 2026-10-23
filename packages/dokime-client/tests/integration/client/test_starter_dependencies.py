# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Locked Starter Wheel Selection And Integrity"""

# === Imports === #

# Standard Library Imports
import hashlib
import io
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.build import dependencies
from packaging.tags import Tag

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_compatible_wheel(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(dependencies, "sys_tags", lambda: iter([Tag("cp315", "cp315", "win_amd64"), Tag("py3", "none", "any")]))
    portable = {"url": "https://example.invalid/example-1.0-py3-none-any.whl"}
    native = {"url": "https://example.invalid/example-1.0-cp315-cp315-win_amd64.whl"}
    foreign = {"url": "https://example.invalid/example-1.0-cp315-cp315-manylinux_2_17_x86_64.whl"}
    assert dependencies._compatible([portable, foreign, native]) == native
    with pytest.raises(ValueError, match="no compatible"):
        dependencies._compatible([foreign])


@pytest.mark.parametrize("valid", [True, False])
def test_locked_wheel_bytes(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, valid: bool) -> None:
    wheel = b"locked wheel bytes"
    output = tmp_path / "wheels"
    output.mkdir()
    destination = output / "example-1.0-py3-none-any.whl"
    destination.write_bytes(b"retained candidate")
    monkeypatch.setattr(dependencies.urllib.request, "urlopen", lambda *_args, **_kwargs: io.BytesIO(wheel))
    digest = hashlib.sha256(wheel if valid else b"other bytes").hexdigest()
    if valid:
        dependencies._fetch(f"https://example.invalid/{destination.name}", f"sha256:{digest}", output)
        assert destination.read_bytes() == wheel
    else:
        with pytest.raises(ValueError, match="hash differs"):
            dependencies._fetch(f"https://example.invalid/{destination.name}", f"sha256:{digest}", output)
        assert destination.read_bytes() == b"retained candidate"


# === Private Functions === #

# === Entry Point === #
