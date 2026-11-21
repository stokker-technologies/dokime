# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Collect Product Protocol Encoded Sizes"""

# === Imports === #

# Standard Library Imports
import json
import shutil
from pathlib import Path

# isort: split

# Third Party Imports
# isort: split

# Local Imports
from .. import policy
from ..build import paths
from ..release import reports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def collect(directory: Path, home: Path, identity: dict, threshold: float, *, refresh: bool, success: bool) -> None:
    sizes = home / "encoded-sizes.json"
    stored = home / "protocol-dev.json"
    if sizes.is_symlink() or stored.is_symlink():
        raise ValueError("Collect Protocol Bench: Keep Native Measurements Unlinked")
    if not sizes.is_file():
        raise ValueError("Collect Protocol Bench: Run the Protocol Target to Record Encoded Sizes")
    previous = json.loads(stored.read_text(encoding="utf-8")) if stored.is_file() else {}
    baselines = {row["name"]: row["value"] for row in previous.get("benchmarks", [])}
    revision = policy.execute(directory, ["git", "rev-parse", "HEAD"], capture=True).strip()
    rows = []
    for name, value in json.loads(sizes.read_text(encoding="utf-8")).items():
        if type(value) is not int or value < 0:
            raise ValueError(f"Collect Protocol Bench: Record a Nonnegative Encoded Size for {name}")
        rows.append(
            {
                "name": name,
                **reports.identity("rust/protocol", paths.target(), name, "Protobuf Encoded Message Size"),
                "label": "Encoded Size [bytes]",
                "unit": "bytes",
                "value": value,
                "baseline_value": baselines.get(name),
                "revision": revision,
                "threshold_percent": threshold,
            }
        )
    summary = {"benchmarks": rows, "identity": identity}
    policy.write_json(home / "protocol-new.json", summary)
    policy.write_json(directory / "target/benchmarks/benchmarks-rust-protocol-sizes.json", summary)
    if success and refresh:
        shutil.copyfile(home / "protocol-new.json", stored)


# === Private Functions === #

# === Entry Point === #
