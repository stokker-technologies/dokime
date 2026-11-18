# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Serve an Independent Typed Plugin Operation."""

# === Imports === #

# Standard Library Imports
# Third Party Imports
import argparse
import asyncio

from google.protobuf.wrappers_pb2 import StringValue

# Local Imports
from operations import plugin

from dokime.client import PLUGIN_STATE_READY, PluginPlacement, PluginRemote, PluginValue
from dokime.connection import connect

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


# === Public Functions === #


async def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", help="Host Endpoint or Pairing File; Defaults to DOKIME_REMOTE")
    parser.add_argument("text")
    options = parser.parse_args()
    client = await connect(options.host)
    endpoint = await plugin.start("127.0.0.1:0")
    try:
        created = await client.plugins.create(
            "example.echo", PluginPlacement(remote=PluginRemote(url=endpoint.url)), PluginValue(enabled=True)
        )
        try:
            entry = created.entry
            watching = client.plugins.watch(name=entry.name)
            try:
                while entry.state != PLUGIN_STATE_READY:
                    if entry.HasField("failure"):
                        raise RuntimeError(f"Start Echo Plugin: Resolve the Reported Failure: {entry.failure}")
                    if await anext(watching, None) is None:
                        raise RuntimeError("Start Echo Plugin: Reconnect After the Host Watch Closed")
                    entry = (await client.plugins.get(entry.name)).entry
            finally:
                await watching.aclose()
            description = await client.operations.get("example.echo")
            response = await client.invocations.create(
                "example.echo", description.contract.fingerprint, StringValue(value=options.text).SerializeToString()
            )
            print(StringValue.FromString(response).value)
        finally:
            current = (await client.plugins.get(created.entry.name)).entry
            await client.plugins.delete(current.name, current.revision)
    finally:
        await endpoint.close()
        await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
