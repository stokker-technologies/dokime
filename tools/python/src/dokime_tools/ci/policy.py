# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Select Required Native Validation Platforms"""

# === Imports === #

# Standard Library Imports
import json
import os
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #
REPOSITORY = "stokker-technologies/dokime"
FULL_PLATFORMS = ("linux", "windows")

# === Types === #

# === Classes === #

# === Public Functions === #


def required_platforms(event: dict) -> tuple[str, ...]:
    """Require Linux for company Dev and both platforms for other targets."""
    if "pull_request" in event:
        target = event["pull_request"].get("base", {})
        dev = (
            target.get("ref") == "dev"
            and target.get("repo", {}).get("full_name") == REPOSITORY
        )
    else:
        dev = (
            event.get("ref") == "refs/heads/dev"
            and event.get("repository", {}).get("full_name") == REPOSITORY
        )
    return ("linux",) if dev else FULL_PLATFORMS


def current_event() -> dict:
    event = json.loads(
        Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8")
    )
    if os.environ.get("GITHUB_EVENT_NAME") == "workflow_dispatch":
        event["ref"] = os.environ.get("GITHUB_REF", "")
    return event


def main() -> None:
    platforms = required_platforms(current_event())
    values = {
        "matrix": json.dumps({"os": platforms}),
        "fast": str(platforms == ("linux",)).lower(),
    }
    with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as output:
        output.writelines(f"{name}={value}\n" for name, value in values.items())


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
