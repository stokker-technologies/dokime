# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Select Build Tasks from the Public Arguments."

# === Imports === #

# Standard Library Imports
import argparse

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def select(args: argparse.Namespace, profile: list[str]) -> list[tuple[str, list[str]]]:
    tasks: list[tuple[str, list[str]]] = []
    if args.artifact == "bench":
        tasks = [("build:rust:bench", ["--binary", args.binary] if args.binary else [])]
    elif args.artifact == "tests":
        tasks = [("build:rust:tests", profile + (["--timings"] if args.timings else []))]
    elif args.language:
        tasks = [(f"build:{args.language}", profile)]
    else:
        tasks = [("build:product", profile)]
    return tasks


# === Private Functions === #

# === Entry Point === #
