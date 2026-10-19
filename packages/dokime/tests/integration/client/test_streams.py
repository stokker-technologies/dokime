# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Refuse Incomplete Canonical Streams And Preserve Remote Failure Details"""

# === Imports === #

# Standard Library Imports
import asyncio

# Third Party Imports
import grpc
import pytest

# Local Imports
import dokime
from dokime.client import DokimeClient, ProtocolError, RpcError

from .backend import BackendFixture
from .canonical import register

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("failure", ["missing_response", "trailing_item", "status_after_response"])
def test_incomplete_stream_refused(failure: str) -> None:
    async def scenario():
        operation = dokime.DOKIME_EVIDENCE_WATCH_RECORDS
        assert operation.output is not None
        server = grpc.aio.server()
        register(server, BackendFixture())

        async def stream(request, context):
            assert operation.output is not None
            await context.send_initial_metadata((("fixture-phase", "opened"),))
            yield operation.rpc_response(item=operation.output())
            if failure == "missing_response":
                return
            yield operation.rpc_response(response=operation.response())
            if failure == "trailing_item":
                yield operation.rpc_response(item=operation.output())
            else:
                context.set_trailing_metadata((("fixture-phase", "failed"),))
                await context.abort(grpc.StatusCode.DATA_LOSS, "failure after final response")

        server.add_registered_method_handlers(
            "dokime.v1.Evidence",
            {
                "WatchRecords": grpc.unary_stream_rpc_method_handler(
                    stream,
                    request_deserializer=operation.request.FromString,
                    response_serializer=lambda result: result.SerializeToString(),
                )
            },
        )
        port = server.add_insecure_port("127.0.0.1:0")
        await server.start()
        client = DokimeClient(f"http://127.0.0.1:{port}")
        reader = client.download(operation, operation.request())
        try:
            expected = RpcError if failure == "status_after_response" else ProtocolError
            with pytest.raises(expected) as caught:
                await reader.response()
            if failure == "status_after_response":
                assert caught.value.code == "DATA_LOSS"
                assert ("fixture-phase", "opened") in caught.value.initial_metadata
                assert ("fixture-phase", "failed") in caught.value.trailing_metadata
            assert not client._transport._calls
        finally:
            await reader.close()
            await client.close()
            await server.stop(0)

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
