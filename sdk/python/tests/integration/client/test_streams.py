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
from dokime.client import ClientClosed, DokimeClient, ProtocolError, RpcError, _contracts, _wire
from dokime.client.execution.listeners import read_records, read_samples

from .backend import EXECUTION_ID, SUITE_ID, BackendFixture, serve
from .canonical import register

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_close_cancels_pending_stream_opening(integration_loop: asyncio.Runner) -> None:
    async def scenario():
        operation = _contracts.operation("Evidence", "WatchRecords")
        server = grpc.aio.server()
        register(server, BackendFixture())
        started = asyncio.Event()
        reading = asyncio.Event()
        finished = asyncio.Event()
        gate = asyncio.Event()

        async def stream(request, context):
            started.set()
            try:
                await gate.wait()
                yield operation.rpc_response(response=operation.response())
            finally:
                finished.set()

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

        async def read():
            # Finish The Python Handoff Before Testing The Pending Native Opening.
            await reader._open()
            reading.set()
            return await anext(aiter(reader))

        pending = asyncio.create_task(read())
        try:
            await started.wait()
            await reading.wait()
            await reader.close()
            with pytest.raises(asyncio.CancelledError):
                await pending
            await finished.wait()
            with pytest.raises(ClientClosed):
                await anext(aiter(reader))
        finally:
            await reader.close()
            await client.close()
            await server.stop(0)

    integration_loop.run(scenario())


@pytest.mark.parametrize(
    ("failure", "ordering"),
    [
        ("missing_response", "immediate"),
        ("trailing_item", "immediate"),
        ("status_after_response", "immediate"),
        ("status_after_response", "buffered"),
        ("status_after_response", "consumed"),
    ],
)
def test_incomplete_stream_refused(failure: str, ordering: str, monkeypatch: pytest.MonkeyPatch, integration_loop: asyncio.Runner) -> None:
    async def scenario():
        operation = _contracts.operation("Evidence", "WatchRecords")
        assert operation.output is not None
        server = grpc.aio.server()
        register(server, BackendFixture())
        consumed = asyncio.Event()
        terminated = asyncio.Event()

        async def stream(request, context):
            assert operation.output is not None
            context.add_done_callback(lambda _: terminated.set())
            await context.send_initial_metadata((("fixture-phase", "opened"),))
            yield operation.rpc_response(item=operation.output())
            if failure == "missing_response":
                return
            yield operation.rpc_response(response=operation.response())
            if failure == "trailing_item":
                yield operation.rpc_response(item=operation.output())
            else:
                if ordering == "consumed":
                    await consumed.wait()
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
        decode = client._transport._response

        def response(payload, expected):
            envelope = decode(payload, expected)
            if expected is operation.rpc_response and envelope.HasField("response"):
                consumed.set()
            return envelope

        monkeypatch.setattr(client._transport, "_response", response)
        try:
            if ordering == "buffered":
                await reader._open()
                await terminated.wait()
            expected = RpcError if failure == "status_after_response" else ProtocolError
            with pytest.raises(expected) as caught:
                await reader.response()
            if failure == "status_after_response":
                assert caught.value.code == "DATA_LOSS"
                assert ("fixture-phase", "opened") in caught.value.initial_metadata
                assert ("fixture-phase", "failed") in caught.value.trailing_metadata
            with pytest.raises(ClientClosed):
                await anext(aiter(reader))
        finally:
            await reader.close()
            await client.close()
            await server.stop(0)

    integration_loop.run(scenario())


def test_unavailable_unary_reads_retry_without_mutation_replay(integration_loop: asyncio.Runner):
    class TransientBackend(BackendFixture):
        def __init__(self):
            super().__init__()
            self.reads = 0
            self.starts_seen = 0

        async def GetSuite(self, request, context):
            self.reads += 1
            if self.reads == 1:
                await context.abort(grpc.StatusCode.UNAVAILABLE, "listener stopping")
            return await super().GetSuite(request, context)

        async def StartExecution(self, request, context):
            self.starts_seen += 1
            await context.abort(grpc.StatusCode.UNAVAILABLE, "listener stopping")

    async def scenario():
        fixture = TransientBackend()
        async with serve(fixture) as (_, client):
            await client._transport.unary("Catalog", "GetSuite", _wire.request("Catalog", "GetSuite", suite_id=SUITE_ID))
            assert fixture.reads == 2
            with pytest.raises(RpcError) as caught:
                await client._transport.unary(
                    "Execution", "StartExecution", _wire.request("Execution", "StartExecution", execution_id=EXECUTION_ID)
                )
            assert caught.value.code == "UNAVAILABLE"
            assert fixture.starts_seen == 1

    integration_loop.run(scenario())


@pytest.mark.parametrize("kind", ["records", "samples"])
def test_unavailable_listeners_resume_delivered_cursors(kind, integration_loop: asyncio.Runner):
    class TransientBackend(BackendFixture):
        def __init__(self):
            super().__init__()
            self.opens = []
            self.finished = asyncio.Event()
            self.publish("first")
            self.publish("second")

        async def WatchRecords(self, request, context):
            after = request.after.position if request.HasField("after") else 0
            self.opens.append(after)
            yield self.page(request.selection, after, after + 1)
            if len(self.opens) == 1:
                await context.abort(grpc.StatusCode.UNAVAILABLE, "listener stopping")
            await self.finished.wait()

        async def WatchSamples(self, request, context):
            positions = {entry.key: entry.value for entry in request.selection.after_stream_cursors}
            self.opens.append(positions.get("voltage", 0))
            if len(self.opens) == 1:
                yield await self.ListSamples(request.selection, context)
                await context.abort(grpc.StatusCode.UNAVAILABLE, "listener stopping")
            async for page in super().WatchSamples(request, context):
                yield page

    async def scenario():
        fixture = TransientBackend()
        async with serve(fixture) as (_, client):
            backend = await client.connect()
            scope = _wire.nested("Evidence", "ListRecords", "selection.scope", execution_id=EXECUTION_ID)
            seen = []
            if kind == "records":
                reader = read_records(client._transport, backend, scope, [], fixture.finished.wait())
            else:
                reader = read_samples(client._transport, backend, scope, [], 1, False)
            try:
                async for page in reader:
                    if kind == "records":
                        seen.extend(item.record_id for item in page.records)
                        if len(seen) == 2:
                            fixture.finished.set()
                    else:
                        seen.extend(item.sample_id for item in page.samples)
            finally:
                await reader.aclose()
            assert fixture.opens == [0, 1]
            assert seen == (["event-1", "event-2"] if kind == "records" else ["sample-1", "sample-2"])

    integration_loop.run(scenario())


# === Private Functions === #

# === Entry Point === #
