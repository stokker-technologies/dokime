# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Parse, start, and report errors for the installed client CLI."""

# === Imports === #

# Standard Library Imports
import argparse
import asyncio
import sys

# Third Party Imports

# Local Imports
from dokime.client import DokimeClient, DokimeError, commands

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def arguments(argv: list[str] | None = None) -> commands.Command:
    parser = argparse.ArgumentParser(
        description="Run A Remote Test And Save Its Evidence And Report."
    )
    return commands.arguments(parser, argv)


async def dispatch(args: commands.Command) -> int:
    token = (
        args.token_file.read_text(encoding="utf-8").strip() if args.token_file else None
    )
    client = DokimeClient(args.endpoint, token=token, ca_certificate=args.ca)
    try:
        return await commands.dispatch(client, args)
    finally:
        await client.close()


def main(argv: list[str] | None = None) -> int:
    args = arguments(argv)
    try:
        return asyncio.run(dispatch(args))
    except (ValueError, FileExistsError) as error:
        print(f"Invalid Configuration: {error}", file=sys.stderr)
        return 2
    except (DokimeError, OSError, RuntimeError, ExceptionGroup) as error:
        print(f"Capture Failed: {error}", file=sys.stderr)
        if args.output.exists():
            print(
                f"Partial Evidence: {args.output}. Retry With collect --output {args.output}",
                file=sys.stderr,
            )
        return 3
    except KeyboardInterrupt:
        print(
            "Disconnected; Inspect The Run Before Starting Another Test",
            file=sys.stderr,
        )
        return 3


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
