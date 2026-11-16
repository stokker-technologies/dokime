# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Select Package Tasks from the Public Arguments."

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


def select(args: argparse.Namespace, languages: list[str], profile: list[str]) -> list[tuple[str, list[str]]]:
    tasks: list[tuple[str, list[str]]] = []
    tasks = []
    for tasks_language in languages:
        if tasks_language == "typescript":
            continue
        tasks.append((f"package:{tasks_language}", profile + (["--target", args.target] if args.target else [])))
    return tasks


# === Private Functions === #

# === Entry Point === #
