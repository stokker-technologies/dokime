# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Start a test and capture its evidence."""

# === Imports === #

# Standard Library Imports
import argparse
from dataclasses import dataclass
import json
from pathlib import Path

# Third Party Imports

# Local Imports
from dokime.client import Capture, DokimeClient, Value
from dokime.client._console import CaptureArgs, add_arguments, capture_run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(kw_only=True)
class Args(CaptureArgs):
    suite: str
    template: str
    parameters: str | None = None
    parameters_file: Path | None = None


# === Public Functions === #


def configure(parser: argparse.ArgumentParser) -> None:
    add_arguments(parser)
    parser.add_argument("--suite", required=True)
    parser.add_argument("--template", required=True)
    parameters = parser.add_mutually_exclusive_group()
    parameters.add_argument("--parameters", help="Inline JSON Object")
    parameters.add_argument("--parameters-file", type=Path, help="JSON Object File")


async def run(client: DokimeClient, args: Args) -> int:
    capture = await Capture.prepare(
        client,
        output=args.output,
        suite=args.suite,
        template=args.template,
        parameters=read_parameters(args),
    )
    request = await capture.start()
    return await capture_run(client, request, capture, args)


# === Private Functions === #


def read_parameters(args: Args) -> dict[str, Value]:
    text = (
        args.parameters_file.read_text(encoding="utf-8")
        if args.parameters_file
        else args.parameters or "{}"
    )

    def reject_constant(value: str) -> None:
        raise ValueError(f"non-finite JSON number: {value}")

    parameters = json.loads(text, parse_constant=reject_constant)
    if not isinstance(parameters, dict):
        raise ValueError("parameters must be a JSON object")
    # Also reject overflow such as 1e999 before creating a cycle.
    json.dumps(parameters, allow_nan=False)
    return parameters


# === Entry Point === #
