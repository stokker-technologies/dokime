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

# Connection Parameters
ENDPOINT = "https://127.0.0.1:8766"
CA_CERTIFICATE: Path | None = Path("server.pem")
KEY_FILE: Path | None = Path("remote.key")
ENV_FILE: Path | None = None

# Response Files
RESPONSE_DIRECTORY: Path | None = None

# === Types === #

# === Classes === #

# === Public Functions === #


async def run() -> int:
    # Reserve Optional Response Files Before Connecting
    if RESPONSE_DIRECTORY is not None:
        RESPONSE_DIRECTORY.mkdir(parents=True, exist_ok=False)

    async with DokimeClient(ENDPOINT, ca_certificate=CA_CERTIFICATE, key_file=KEY_FILE, env_file=ENV_FILE) as client:
        # Discover Device Operations
        for selected in await client.operations():
            if selected.operation.id.startswith("obc.device."):
                print(f"Operation: {selected.operation.id} Provider: {selected.owner}", flush=True)

        # Read, Power On And Observe
        response(await client.call("obc.device.read", {}), "Before", "before")
        response(await client.call("obc.device.set_power", {"enabled": True}), "Power On", "power-on")
        observed = response(await client.call("obc.device.read", {}), "Running", "running")
        if observed.get("enabled") is not True or observed.get("voltage_rms_v") != 230.0:
            raise ValueError("manual device did not report the expected powered state")

        # Power Off And Verify
        response(await client.call("obc.device.set_power", {"enabled": False}), "Power Off", "power-off")
        observed = response(await client.call("obc.device.read", {}), "After", "after")
        if observed.get("enabled") is not False or observed.get("voltage_rms_v") != 0.0:
            raise ValueError("manual device did not report the expected powered-off state")
    return 0


def main() -> int:
    try:
        return asyncio.run(run())
    except KeyboardInterrupt:
        print("Stopped Waiting; Read Device State Before Resending An Unconfirmed Command", file=sys.stderr)
        return 130
    except (DokimeError, OSError, ValueError) as error:
        print(f"Device Workflow Incomplete: {error}. No Command Was Retried", file=sys.stderr)
        return 1


# === Private Functions === #


def response(message: Message, label: str, filename: str) -> dict:
    observed = MessageToDict(message, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
    print(f"{label}: {json.dumps(observed)}", flush=True)
    if RESPONSE_DIRECTORY is not None:
        with (RESPONSE_DIRECTORY / f"{filename}.json").open("x", encoding="utf-8", newline="\n") as output:
            output.write(json.dumps(observed, indent=2) + "\n")
    return observed


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
