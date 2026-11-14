# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""A Dokime Plugin Passes The Pinned Harmos Reference Host's Protocol Rules."""

# === Imports === #

# Standard Library Imports
import asyncio
import os
import sys
import unittest
from importlib import import_module
from pathlib import Path
from typing import cast

from harmos.plugin import Plugin as LibraryPlugin

# Third Party Imports
import dokime

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ConformanceStory(unittest.IsolatedAsyncioTestCase):
    async def test_reference_host_accepts_remote_plugin(self) -> None:
        # Compose The Language-Neutral Harmos Fixture Through Dokime's Plugin.
        source = os.environ.get("HARMOS_CONFORMANCE_FIXTURE")
        if source is None or not (Path(source) / "__init__.py").is_file():
            self.fail("Run Conformance: Set HARMOS_CONFORMANCE_FIXTURE to the Pinned Library Test Fixture Directory")
        sys.path.insert(0, str(Path(source).parent))
        try:
            fixture = cast(LibraryPlugin, import_module(Path(source).name).plugin())
        finally:
            sys.path.pop(0)
        remote = dokime.Plugin(
            fixture.id,
            fixture.version,
            requires=fixture.requires,
            sections=fixture.sections,
            messages=fixture.messages,
            operations=fixture.instance,
        )
        endpoint = await remote.start("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)

        # Run The Reference Host Selected By The Test's Process Owner.
        binary = os.environ.get("HARMOS_CONFORMANCE_HOST")
        if not binary or not Path(binary).is_file():
            self.fail("Set HARMOS_CONFORMANCE_HOST to the harmos-conformance executable built from the pinned Harmos checkout")
        process = await asyncio.create_subprocess_exec(
            binary,
            "--address",
            endpoint.url,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            creationflags=0x08000000 if os.name == "nt" else 0,
        )
        try:
            stdout, stderr = await process.communicate()
        finally:
            if process.returncode is None:
                process.kill()
                await process.wait()
        self.assertEqual(process.returncode, 0, (stdout + stderr).decode())
        print(stdout.decode(), end="")


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
