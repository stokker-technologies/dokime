# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Capture The Desktop Journey Or Validate Its Published Images."""

# === Imports === #

# Standard Library Imports
import argparse
import json
import re
import struct
import zlib
from datetime import datetime
from pathlib import Path

# Third Party Imports
from harmos_tools.release import policy

# Local Imports

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
    """Validate PNG Chunk Boundaries And Checksums Before Reading Its Viewport."""
    if data[:8] != PNG:
        raise policy.ToolError("screenshot must be a PNG")
    offset = 8
    chunks = []
    size = None
    while offset < len(data):
        if offset + 12 > len(data):
            raise policy.ToolError("truncated PNG chunk")
        length = struct.unpack(">I", data[offset : offset + 4])[0]
        end = offset + 12 + length
        if end > len(data):
            raise policy.ToolError("truncated PNG payload")
        kind = data[offset + 4 : offset + 8]
        payload = data[offset + 8 : end - 4]
        checksum = struct.unpack(">I", data[end - 4 : end])[0]
        if zlib.crc32(kind + payload) != checksum:
            raise policy.ToolError("invalid PNG checksum")
        if not chunks:
            if kind != b"IHDR" or length != 13:
                raise policy.ToolError("screenshot must start with PNG IHDR")
            size = struct.unpack(">II", payload[:8])
        chunks.append(kind)
        offset = end
        if kind == b"IEND":
            if length or offset != len(data):
                raise policy.ToolError("invalid PNG end")
            break
    if not size or not all(size) or b"IDAT" not in chunks or chunks[-1:] != [b"IEND"]:
        raise policy.ToolError("incomplete PNG image")
    return size


def check(directory: Path, _: str = "check") -> None:
    """Check The Complete Published Set Without Starting Any Processes."""
    folder = directory / "docs/screenshots"
    try:
        manifest = json.loads((folder / "manifest.json").read_text(encoding="utf-8"))
        if not isinstance(manifest, list):
            raise policy.ToolError("screenshot manifest must be an array")
        expected = {f"{scene}-{theme}.png" for scene in SCENES for theme in THEMES}
        entries = {}
        for entry in manifest:
            if not isinstance(entry, dict):
                raise policy.ToolError("invalid screenshot manifest entry")
            scene, theme = entry["scene"], entry["theme"]
            image = entry["image"]
            if scene not in SCENES or theme not in THEMES or image != f"{scene}-{theme}.png" or image in entries:
                raise policy.ToolError(f"duplicate or inconsistent screenshot: {image}")
            viewport = (1920, 1200) if scene == "touch" else (1600, 1000)
            density = "touch" if scene == "touch" else "comfortable"
            if entry["viewport"] != dict(zip(("width", "height"), viewport, strict=True)) or entry["density"] != density:
                raise policy.ToolError(f"invalid scene viewport or density: {image}")
            if not re.fullmatch(r"[0-9a-f]{40,64}", entry["revision"]):
                raise policy.ToolError("revision requires a full commit hash")
            if datetime.fromisoformat(entry["captured_at"]).tzinfo is None:
                raise policy.ToolError("captured_at requires a timezone")
            target = folder / image
            if target.is_symlink() or not target.is_file() or target.stat().st_size > MAX_BYTES:
                raise policy.ToolError(f"screenshot must be a regular PNG at most 4 MiB: {image}")
            if dimensions(target.read_bytes()) != viewport:
                raise policy.ToolError(f"screenshot viewport mismatch: {image}")
            entries[image] = entry
        if set(entries) != expected:
            raise policy.ToolError(f"missing screenshots in manifest: {sorted(expected - entries.keys())}")
        unexpected = {path.name for path in folder.iterdir()} - expected - {"README.md", "manifest.json"}
        if unexpected:
            raise policy.ToolError(f"unexpected screenshot files: {sorted(unexpected)}")
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise policy.ToolError(f"invalid screenshot set: {error}") from error
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
