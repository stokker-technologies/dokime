# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Recover an existing capture without starting another test."""

# === Imports === #

# Standard Library Imports
import argparse
from dataclasses import dataclass

# Third Party Imports

# Local Imports
from dokime.client import Capture, DokimeClient
from dokime.client._console import CaptureArgs, add_arguments, capture_run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(kw_only=True)
class Args(CaptureArgs):
    pass


# === Public Functions === #


def configure(parser: argparse.ArgumentParser) -> None:
    add_arguments(parser)


async def run(client: DokimeClient, args: Args) -> int:
    capture = await Capture.open(client, output=args.output)
    request = await capture.start()
    return await capture_run(client, request, capture, args)


# === Private Functions === #

# === Entry Point === #
