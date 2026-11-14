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
import asyncio
import sys

# Third Party Imports
from google.protobuf.wrappers_pb2 import StringValue
from harmos import VISIBILITY_PUBLIC
from harmos.plugin import Context, operation

# Local Imports
from dokime.plugin import Plugin

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Operations:
    @operation("example.echo", request=StringValue, response=StringValue, visibility=VISIBILITY_PUBLIC)
    async def echo(self, context: Context, request: StringValue) -> StringValue:
        return StringValue(value=request.value)


# === Public Functions === #


async def main() -> None:
    plugin = Plugin("example.echo", "1.0.0", operations=Operations(), requires=[])
    endpoint = await plugin.start("127.0.0.1:0")
    print(endpoint.url, flush=True)
    try:
        await asyncio.to_thread(sys.stdin.buffer.read)
    finally:
        await endpoint.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
