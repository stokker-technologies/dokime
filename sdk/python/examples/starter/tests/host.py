# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Mechanical Harmos Calls For The Starter's In-Process Host."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
import grpc
from harmos import Failure
from harmos.v1 import plugin_pb2 as wire

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def invoke(client, operation, request, response_type, *, scope="", host=None, attempt=1):
    call = client.Invoke()
    await call.write(
        wire.HostFrame(
            request=wire.Request(
                operation=operation, payload=request.SerializeToString(), invocation="starter", attempt=attempt, scope=scope
            )
        )
    )
    try:
        while True:
            frame = await call.read()
            if frame is grpc.aio.EOF:
                raise RuntimeError("Plugin sent no terminal response")
            if frame.WhichOneof("frame") == "child":
                if host is None:
                    raise RuntimeError(f"Unexpected host call: {frame.child.operation}")
                reply = await host(frame.child)
                await call.write(wire.HostFrame(child=wire.ChildResponse(call=frame.child.call, payload=reply.SerializeToString())))
                continue
            if frame.WhichOneof("frame") != "response":
                raise RuntimeError("Unexpected plugin frame")
            if frame.response.WhichOneof("result") == "failure":
                raise Failure(frame.response.failure)
            response = response_type.FromString(frame.response.payload)
            if await call.read() is not grpc.aio.EOF:
                raise RuntimeError("Plugin sent frames after its terminal response")
            return response
    finally:
        call.cancel()


# === Private Functions === #

# === Entry Point === #
