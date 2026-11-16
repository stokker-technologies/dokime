# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Select Lint Tasks from the Public Arguments."

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


def select(args: argparse.Namespace, languages: list[str]) -> list[tuple[str, list[str]]]:
    tasks: list[tuple[str, list[str]]] = []
    if args.operation == "format" and not args.language:
        languages.append("toml")
    tasks = []
    for tasks_language in languages:
        tasks.append((f"{args.operation}:{tasks_language}", ["--check"] if args.check else []))
    return tasks


# === Private Functions === #

# === Entry Point === #
