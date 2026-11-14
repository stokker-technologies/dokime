# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Published Screenshot Set Validation"""

# === Imports === #

# Standard Library Imports
import json
import struct
import tempfile
import unittest
import zlib
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
from harmos_tools.release import policy

# Local Imports
from dokime_tools.screenshots import run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ScreenshotSet(unittest.TestCase):
    def test_published_set_rejects_missing_corrupt_and_inconsistent_assets(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            folder = root / "docs/screenshots"
            folder.mkdir(parents=True)
            entries = []
            for scene in run.SCENES:
                width, height = (1920, 1200) if scene == "touch" else (1600, 1000)
                for theme in run.THEMES:
                    image = f"{scene}-{theme}.png"
                    data = png(width, height)
                    (folder / image).write_bytes(data)
                    entries.append(
                        {
                            "image": image,
                            "scene": scene,
                            "theme": theme,
                            "viewport": {"width": width, "height": height},
                            "density": "touch" if scene == "touch" else "comfortable",
                            "revision": "1234567890abcdef" * 2 + "12345678",
                            "captured_at": "2026-01-01T00:00:00+00:00",
                        }
                    )
            manifest = folder / "manifest.json"
            manifest.write_text(json.dumps(entries), encoding="utf-8")
            target = folder / "cycle-light.png"
            original = target.read_bytes()

            # Validate Offline And Refuse Broken Image Contents.
            with patch.object(policy, "execute", side_effect=AssertionError("offline validation launched a process")):
                run.check(root)
                target.unlink()
                with self.assertRaises(policy.ToolError):
                    run.check(root)
                for invalid in (original[:-7], b"not a PNG", png(800, 600), original[:40] + b"broken" + original[46:]):
                    target.write_bytes(invalid)
                    with self.assertRaises(policy.ToolError):
                        run.check(root)
                target.write_bytes(original)

                # Refuse Duplicate, Missing And Inconsistent Metadata.
                for invalid in (
                    entries[:-1],
                    entries + [entries[0]],
                    [{**entries[0], "theme": "sepia"}, *entries[1:]],
                    [{**entries[0], "density": "touch"}, *entries[1:]],
                ):
                    manifest.write_text(json.dumps(invalid), encoding="utf-8")
                    with self.assertRaises(policy.ToolError):
                        run.check(root)
                manifest.write_text(json.dumps(entries), encoding="utf-8")
                (folder / "old-scene.png").write_bytes(original)
                with self.assertRaises(policy.ToolError):
                    run.check(root)
                (folder / "old-scene.png").unlink()
                run.check(root)


# === Public Functions === #

# === Private Functions === #


def png(width: int, height: int) -> bytes:
    def chunk(kind: bytes, payload: bytes) -> bytes:
        return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", zlib.crc32(kind + payload))

    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    pixels = zlib.compress((b"\0" + b"\x80\x90\xa0" * width) * height)
    return run.PNG + chunk(b"IHDR", header) + chunk(b"IDAT", pixels) + chunk(b"IEND", b"")


# === Entry Point === #
