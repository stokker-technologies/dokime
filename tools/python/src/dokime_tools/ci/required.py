# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Run the Required Workflow Operation."

# === Imports === #

# Standard Library Imports
import json
import os

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


def run() -> None:
    results = json.loads(os.environ["RESULTS"])
    allowed_skips = set(json.loads(os.environ.get("OPTIONAL_JOBS", "[]")))
    failed = []
    for failed_name, failed_job in results.items():
        if failed_job["result"] == "success":
            continue
        if failed_name in allowed_skips and failed_job["result"] == "skipped":
            continue
        failed.append(failed_name)

    if failed:
        raise ValueError("required CI jobs failed or skipped: " + ", ".join(failed))


# === Private Functions === #

# === Entry Point === #
