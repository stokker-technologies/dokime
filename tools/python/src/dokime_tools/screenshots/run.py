# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Capture the Desktop Journey or Validate Its Published Images."

# === Imports === #

# Standard Library Imports
import argparse
import json
import re
import struct
import zlib
from datetime import datetime
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PNG = b"\x89PNG\r\n\x1a\n"
MAX_BYTES = 4 * 1024 * 1024
SCENES = ("cycle", "plots", "report", "history", "library", "cycles", "touch")
THEMES = ("light", "dark")

# === Types === #

# === Classes === #

# === Public Functions === #


def dimensions(data: bytes) -> tuple[int, int]:
    """Validate PNG Chunk Boundaries and Checksums Before Reading Its Viewport."""
    if data[:8] != PNG:
        raise policy.ToolError("Capture Screenshots: screenshot must be a PNG")
    offset = 8
    chunks = []
    size = None
    while offset < len(data):
        if offset + 12 > len(data):
            raise policy.ToolError("Capture Screenshots: truncated PNG chunk")
        length = struct.unpack(">I", data[offset : offset + 4])[0]
        end = offset + 12 + length
        if end > len(data):
            raise policy.ToolError("Capture Screenshots: truncated PNG payload")
        kind = data[offset + 4 : offset + 8]
        payload = data[offset + 8 : end - 4]
        checksum = struct.unpack(">I", data[end - 4 : end])[0]
        if zlib.crc32(kind + payload) != checksum:
            raise policy.ToolError("Capture Screenshots: invalid PNG checksum")
        if not chunks:
            if kind != b"IHDR" or length != 13:
                raise policy.ToolError("Capture Screenshots: screenshot must start with PNG IHDR")
            size = struct.unpack(">II", payload[:8])
        chunks.append(kind)
        offset = end
        if kind == b"IEND":
            if length or offset != len(data):
                raise policy.ToolError("Capture Screenshots: invalid PNG end")
            break
    if not size or not all(size) or b"IDAT" not in chunks or chunks[-1:] != [b"IEND"]:
        raise policy.ToolError("Capture Screenshots: incomplete PNG image")
    return size


def check(directory: Path, _: str = "check") -> None:
    """Check the Complete Published Set Without Starting Any Processes."""
    folder = directory / "docs/guides/screenshots"
    try:
        manifest = json.loads((folder / "manifest.json").read_text(encoding="utf-8"))
        if not isinstance(manifest, list):
            raise policy.ToolError("Capture Screenshots: screenshot manifest must be an array")
        expected = set()
        for expected_scene in SCENES:
            for expected_theme in THEMES:
                expected.add(f"{expected_scene}-{expected_theme}.png")

        entries = {}
        for entry in manifest:
            if not isinstance(entry, dict):
                raise policy.ToolError("Capture Screenshots: invalid screenshot manifest entry")
            scene, theme = entry["scene"], entry["theme"]
            image = entry["image"]
            if scene not in SCENES or theme not in THEMES or image != f"{scene}-{theme}.png" or image in entries:
                raise policy.ToolError(f"Capture Screenshots: duplicate or inconsistent screenshot: {image}")
            viewport = (1920, 1200) if scene == "touch" else (1600, 1000)
            density = "touch" if scene == "touch" else "comfortable"
            if entry["viewport"] != dict(zip(("width", "height"), viewport, strict=True)) or entry["density"] != density:
                raise policy.ToolError(f"Capture Screenshots: invalid scene viewport or density: {image}")
            if not re.fullmatch(r"[0-9a-f]{40,64}", entry["revision"]):
                raise policy.ToolError("Capture Screenshots: revision requires a full commit hash")
            if datetime.fromisoformat(entry["captured_at"]).tzinfo is None:
                raise policy.ToolError("Capture Screenshots: captured_at requires a timezone")
            target = folder / image
            if target.is_symlink() or not target.is_file() or target.stat().st_size > MAX_BYTES:
                raise policy.ToolError(f"Capture Screenshots: screenshot must be a regular PNG at most 4 MiB: {image}")
            if dimensions(target.read_bytes()) != viewport:
                raise policy.ToolError(f"Capture Screenshots: screenshot viewport mismatch: {image}")
            entries[image] = entry
        if set(entries) != expected:
            raise policy.ToolError(f"Capture Screenshots: missing screenshots in manifest: {sorted(expected - entries.keys())}")
        unexpected = {path.name for path in folder.iterdir()} - expected - {"manifest.json"}
        if unexpected:
            raise policy.ToolError(f"Capture Screenshots: unexpected screenshot files: {sorted(unexpected)}")
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise policy.ToolError(f"Capture Screenshots: invalid screenshot set: {error}") from error
    print(f"screenshots: {len(entries)} images checked")


def capture(directory: Path, scene: str | None = None) -> None:
    arguments = ["node", "crates/dokime-gui/e2e/screenshots.mjs"]
    if scene:
        arguments += ["--scene", scene]
    policy.execute(directory, arguments)
    if scene is None:
        check(directory)


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scene", choices=SCENES)
    parser.add_argument("--check", action="store_true")
    options = parser.parse_args(arguments)
    if options.check and options.scene:
        parser.error("--check validates the complete set; omit --scene")
    try:
        directory = policy.root()
        if options.check:
            check(directory)
        else:
            capture(directory, options.scene)
    except policy.ToolError as error:
        parser.exit(1, f"screenshots: {error}\n")
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
