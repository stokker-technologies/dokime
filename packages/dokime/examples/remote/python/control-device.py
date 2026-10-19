# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Discover And Control The Manual Simulated Device"""

# === Imports === #

# Standard Library Imports
import argparse
import asyncio
import json
import sys
from pathlib import Path

# Third Party Imports
from google.protobuf.json_format import MessageToDict
from google.protobuf.message import Message

from dokime.client import DokimeClient, DokimeError

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def run(arguments: argparse.Namespace) -> int:
    directory: Path | None = arguments.response_directory
    if directory is not None:
        directory.mkdir(parents=True, exist_ok=False)

    async with DokimeClient(
        arguments.endpoint,
        key_file=arguments.key_file,
        env_file=arguments.env_file,
        ca_certificate=arguments.ca_certificate,
    ) as client:
        for selected in await client.operations():
            if selected.operation.id.startswith("obc.device."):
                print(f"Operation: {selected.operation.id} Provider: {selected.owner}", flush=True)

        state = await client.call("obc.device.read", {})
        response(state, "Before", "before", directory)
        powered = await client.call("obc.device.set_power", {"enabled": True})
        response(powered, "Power On", "power-on", directory)
        state = await client.call("obc.device.read", {})
        observed = response(state, "Running", "running", directory)
        if observed.get("enabled") is not True or observed.get("voltage_rms_v") != 230.0:
            raise ValueError("manual device did not report the expected powered state")

        powered = await client.call("obc.device.set_power", {"enabled": False})
        response(powered, "Power Off", "power-off", directory)
        state = await client.call("obc.device.read", {})
        observed = response(state, "After", "after", directory)
        if observed.get("enabled") is not False or observed.get("voltage_rms_v") != 0.0:
            raise ValueError("manual device did not report the expected powered-off state")
        return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--endpoint", required=True)
    parser.add_argument("--ca-certificate", type=Path)
    parser.add_argument("--key-file", type=Path)
    parser.add_argument("--env-file", type=Path, help="Read this explicit dotenv file; process variables take precedence")
    parser.add_argument(
        "--response-directory", type=Path, help="Save Five JSON Replies In A New Directory; Existing Directories Are Refused"
    )
    arguments = parser.parse_args()
    try:
        return asyncio.run(run(arguments))
    except KeyboardInterrupt:
        print("Stopped Waiting; A Dispatched Device Command May Have Taken Effect. Read Device State Before Resending", file=sys.stderr)
        return 130
    except (DokimeError, OSError, ValueError) as error:
        print(f"Device Workflow Incomplete: {error}", file=sys.stderr)
        print("No Command Was Retried. Read Device State Before Manually Resending An Unconfirmed Command", file=sys.stderr)
        return 1


# === Private Functions === #


def response(message: Message, label: str, filename: str, directory: Path | None) -> dict:
    observed = MessageToDict(message, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
    text = json.dumps(observed, indent=2) + "\n"
    print(f"{label}: {json.dumps(observed)}", flush=True)
    if directory is not None:
        destination = directory / f"{filename}.json"
        with destination.open("x", encoding="utf-8", newline="\n") as output:
            output.write(text)
    return observed


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
