# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Select Check Tasks from the Public Arguments."

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
    if args.repository:
        tasks = [("check:repository", [])]
    elif args.bindings:
        tasks = [("check:python:bindings", profile)]
    elif args.supply:
        tasks = [("check:rust:supply", [])]
    else:
        tasks = [("check:repository", [])]
        tasks_items = []
        for tasks_language in languages:
            if tasks_language == "toml":
                continue
            tasks_items.append((f"check:{tasks_language}", []))
        tasks += tasks_items
    return tasks


# === Private Functions === #

# === Entry Point === #
