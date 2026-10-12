# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Compose the client command tree and dispatch typed arguments."""

# === Imports === #

# Standard Library Imports
import argparse

# Third Party Imports

# Local Imports
from dokime.client import DokimeClient
from dokime.client.commands import collect, run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

type Command = run.Args | collect.Args

# === Classes === #

# === Public Functions === #


def arguments(parser: argparse.ArgumentParser, argv: list[str] | None) -> Command:
    subcommands = parser.add_subparsers(dest="command", required=True)
    run.configure(subcommands.add_parser("run", help="Run A Test And Capture Evidence"))
    collect.configure(
        subcommands.add_parser(
            "collect", help="Retry An Existing Capture Without Starting A New Test"
        )
    )
    values = vars(parser.parse_args(argv))
    command = values.pop("command")
    if command == "run":
        return run.Args(**values)
    return collect.Args(**values)


async def dispatch(client: DokimeClient, args: Command) -> int:
    if isinstance(args, run.Args):
        return await run.run(client, args)
    return await collect.run(client, args)


# === Private Functions === #

# === Entry Point === #
