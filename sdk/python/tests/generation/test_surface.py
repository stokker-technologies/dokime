# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compile the Generated Public Client and Plugin Call Shapes."""

# === Imports === #

# Standard Library Imports

# Third Party Imports

# Local Imports
from dokime import FileReference, FileSource, Output, Window
from dokime.client import Client
from dokime.plugin import Context

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def generated_surface(client: Client, context: Context) -> None:
    """Type-Check Content Calls Without Starting a Product Host."""
    await client.analyses.list()
    await context.host.files.capture(FileSource(), "output")
    await context.host.content.artifacts.create(FileReference(), "capture.bin", "application/octet-stream")
    await context.host.evidence.statistics("stream", [], Window(), list[Output](), "field", "step")


# === Private Functions === #

# === Entry Point === #
