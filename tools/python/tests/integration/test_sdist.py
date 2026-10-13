# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Pure Source Archives Preserve Exact Prepared Inputs"""

# === Imports === #

# Standard Library Imports
import io
import tarfile
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.check.package import _verify_sdist_build_inputs

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize(
    "change",
    ["valid", "missing", "stale", "extra", "native", "traversal", "link", "duplicate"],
)
def test_sdist_inputs(tmp_path: Path, change: str) -> None:
    package = tmp_path / "packages/dokime"
    payload = {
        "src/dokime/__init__.py": b"# Client",
        "src/dokime/py.typed": b"",
        "examples/client.py": b"# Example",
        "pyproject.toml": b"# Prepared build",
        "README.md": b"# Dokime",
        "LICENSE-APACHE": b"Apache license",
    }
    for name, content in payload.items():
        source = package / name
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_bytes(content)
    if change == "missing":
        del payload["pyproject.toml"]
    elif change == "stale":
        payload["src/dokime/__init__.py"] = b"# Old client"
    elif change == "extra":
        payload["src/dokime/obsolete.py"] = b""
    elif change == "native":
        payload["Cargo.toml"] = b"[workspace]"
    elif change == "traversal":
        payload["../outside"] = b""
    payload["PKG-INFO"] = b"Name: dokime\nVersion: 0.1.0\n"
    archive = tmp_path / "dokime-0.1.0.tar.gz"
    with tarfile.open(archive, "w:gz") as contents:
        for name, content in payload.items():
            entry = tarfile.TarInfo(f"dokime-0.1.0/{name}")
            entry.size = len(content)
            contents.addfile(entry, io.BytesIO(content))
        if change == "link":
            entry = tarfile.TarInfo("dokime-0.1.0/link")
            entry.type = tarfile.SYMTYPE
            entry.linkname = "../outside"
            contents.addfile(entry)
        if change == "duplicate":
            entry = tarfile.TarInfo("dokime-0.1.0/README.md")
            contents.addfile(entry, io.BytesIO())
    if change == "valid":
        _verify_sdist_build_inputs(archive, tmp_path)
    else:
        with pytest.raises(SystemExit):
            _verify_sdist_build_inputs(archive, tmp_path)


# === Private Functions === #

# === Entry Point === #
