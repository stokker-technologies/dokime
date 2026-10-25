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

from dokime_client.client import DokimeClient, DokimeError

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# Connection Parameters
ENDPOINT = "https://127.0.0.1:8766"
CA_CERTIFICATE: Path | None = Path("server.pem")
KEY_FILE: Path | None = Path("remote.key")
ENV_FILE: Path | None = None

# === Types === #

# === Classes === #

# === Public Functions === #


async def run() -> int:
    async with DokimeClient(ENDPOINT, ca_certificate=CA_CERTIFICATE, key_file=KEY_FILE, env_file=ENV_FILE) as client:
        # Discover Device Operations
        operations = await client.operations()
        print("Operations:", operations, flush=True)

        # Read The Manual Device Before Changing Its Power
        reply = await client.call("obc.device.read", {})
        observed = MessageToDict(reply, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
        print("Before:", json.dumps(observed), flush=True)
        Path("before.json").write_text(json.dumps(observed, indent=2) + "\n", encoding="utf-8")

        # Enable The Manual Device; This Is Separate From Test Evidence
        reply = await client.call("obc.device.set_power", {"enabled": True})
        observed = MessageToDict(reply, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
        print("Power On:", json.dumps(observed), flush=True)
        Path("power-on.json").write_text(json.dumps(observed, indent=2) + "\n", encoding="utf-8")

        # Read Back The Powered State: 230 V RMS
        reply = await client.call("obc.device.read", {})
        observed = MessageToDict(reply, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
        print("Running:", json.dumps(observed), flush=True)
        Path("running.json").write_text(json.dumps(observed, indent=2) + "\n", encoding="utf-8")
        if observed.get("enabled") is not True or observed.get("voltage_rms_v") != 230.0:
            raise ValueError("manual device did not report the expected powered state")

        # Disable The Manual Device
        reply = await client.call("obc.device.set_power", {"enabled": False})
        observed = MessageToDict(reply, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
        print("Power Off:", json.dumps(observed), flush=True)
        Path("power-off.json").write_text(json.dumps(observed, indent=2) + "\n", encoding="utf-8")

        # Read Back The Powered-Off State: Zero Volts
        reply = await client.call("obc.device.read", {})
        observed = MessageToDict(reply, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
        print("After:", json.dumps(observed), flush=True)
        Path("after.json").write_text(json.dumps(observed, indent=2) + "\n", encoding="utf-8")
        if observed.get("enabled") is not False or observed.get("voltage_rms_v") != 0.0:
            raise ValueError("manual device did not report the expected powered-off state")
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        raise SystemExit(asyncio.run(run()))
    except KeyboardInterrupt:
        print("Stopped Waiting; Read Device State Before Resending An Unconfirmed Command", file=sys.stderr)
        raise SystemExit(130)
    except (DokimeError, OSError, ValueError) as error:
        print(f"Device Workflow Incomplete: {error}. No Command Was Retried", file=sys.stderr)
        raise SystemExit(1)
