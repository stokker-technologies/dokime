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

from dokime.client import DokimeClient, DokimeError

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def run(arguments: argparse.Namespace) -> int:
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
        print("Before:", json.dumps(MessageToDict(state, always_print_fields_with_no_presence=True)), flush=True)
        powered = await client.call("obc.device.set_power", {"enabled": True})
        print("Power On:", json.dumps(MessageToDict(powered, always_print_fields_with_no_presence=True)), flush=True)
        state = await client.call("obc.device.read", {})
        observed = MessageToDict(state, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
        if observed.get("enabled") is not True or observed.get("voltage_rms_v") != 230.0:
            raise ValueError("manual device did not report the expected powered state")

        powered = await client.call("obc.device.set_power", {"enabled": False})
        print("Power Off:", json.dumps(MessageToDict(powered, always_print_fields_with_no_presence=True)), flush=True)
        state = await client.call("obc.device.read", {})
        observed = MessageToDict(state, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
        if observed.get("enabled") is not False or observed.get("voltage_rms_v") != 0.0:
            raise ValueError("manual device did not report the expected powered-off state")
        print("After:", json.dumps(observed), flush=True)
        return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--endpoint", required=True)
    parser.add_argument("--ca-certificate", type=Path)
    parser.add_argument("--key-file", type=Path)
    parser.add_argument("--env-file", type=Path, help="Read this explicit dotenv file; process variables take precedence")
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

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
