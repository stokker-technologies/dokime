# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Ordered Pairing Connections"""

# === Imports === #

# Standard Library Imports
import json
import socket
import tempfile
import unittest
from asyncio import wait_for
from collections.abc import Sequence
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
from grpc import ChannelConnectivity, StatusCode
from grpc.aio import Channel, ServicerContext, UnaryStreamClientInterceptor, UnaryUnaryClientInterceptor, insecure_channel, server
from harmos._wire.host.v1.status_pb2 import StatusGetRequest
from harmos._wire.model.v1.failure_pb2 import Failure

# Local Imports
from dokime_sdk._connection import connect
from dokime_sdk._errors import SdkError
from dokime_sdk._wire.host.v1.status_pb2 import StatusGetResponse
from dokime_sdk._wire.host.v1.status_pb2_grpc import StatusServicer, add_StatusServicer_to_server

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ConnectionStory(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.channels: list[Channel] = []

        def open_channel(target: str, *, interceptors: Sequence[UnaryUnaryClientInterceptor | UnaryStreamClientInterceptor]) -> Channel:
            if self.channels:
                self.assertEqual(self.channels[-1].get_state(), ChannelConnectivity.SHUTDOWN)
            channel = insecure_channel(target, interceptors=interceptors)
            self.channels.append(channel)
            return channel

        self.channel_patch = patch("dokime_sdk._connection.insecure_channel", side_effect=open_channel)
        self.channel_patch.start()
        self.addCleanup(self.channel_patch.stop)

    def refused(self) -> str:
        listener = socket.socket()
        listener.bind(("127.0.0.1", 0))
        self.addCleanup(listener.close)
        return f"http://127.0.0.1:{listener.getsockname()[1]}"

    async def listening(self, identity: str, failure: Failure | None = None) -> str:
        class Status(StatusServicer):
            async def Get(self, request: StatusGetRequest, context: ServicerContext) -> StatusGetResponse:
                if failure is not None:
                    context.set_trailing_metadata((("grpc-status-details-bin", failure.SerializeToString()),))
                    await context.abort(StatusCode.PERMISSION_DENIED, "Host Refused Connection")
                return StatusGetResponse(status={"identity": {"host_id": identity}})

        listener = server()
        add_StatusServicer_to_server(Status(), listener)
        port = listener.add_insecure_port("127.0.0.1:0")
        await listener.start()
        self.addAsyncCleanup(listener.stop, None)
        return f"http://127.0.0.1:{port}"

    def pairing(self, endpoints: list[str]) -> Path:
        path = Path(self.directory.name) / "host.pair"
        path.write_text(
            json.dumps(
                {
                    "version": 1,
                    "endpoints": endpoints,
                    "backend_identifier": "intended-host",
                    "label": "Operator",
                    "certificate_pem": None,
                    "access_key": None,
                }
            )
        )
        return path

    async def test_connection_refused_then_accepted(self) -> None:
        refused = self.refused()
        accepted = await self.listening("intended-host")
        path = self.pairing([refused, accepted])
        with patch("dokime_sdk._connection.wait_for", wraps=wait_for) as bounded:
            client = await connect(path, connect_timeout=7.5)
        self.addAsyncCleanup(client.close)
        self.assertEqual([call.args[1] for call in bounded.call_args_list], [7.5, 7.5])
        self.assertEqual(self.channels[0].get_state(), ChannelConnectivity.SHUTDOWN)
        self.assertEqual((await client._transport.status.get()).status.identity.host_id, "intended-host")

    async def test_accepted_pairing_endpoint_does_not_inspect_later_endpoints(self) -> None:
        accepted = await self.listening("intended-host")
        client = await connect(self.pairing([accepted, "not-an-origin"]))
        self.addAsyncCleanup(client.close)
        self.assertEqual(len(self.channels), 1)
        self.assertEqual((await client._transport.status.get()).status.identity.host_id, "intended-host")

    async def test_all_refused_errors_follow_file_order(self) -> None:
        endpoints = [self.refused(), self.refused()]
        path = self.pairing(endpoints)
        with self.assertRaises(SdkError) as failure:
            await connect(path)
        message = str(failure.exception)
        self.assertLess(message.index(endpoints[0]), message.index(endpoints[1]))
        for endpoint in endpoints:
            self.assertIn(f"{endpoint}:", message)
        self.assertTrue(all(channel.get_state() == ChannelConnectivity.SHUTDOWN for channel in self.channels))

    async def test_wrong_identity_then_right_identity(self) -> None:
        wrong = await self.listening("other-host")
        right = await self.listening("intended-host")
        client = await connect(self.pairing([wrong, right]))
        self.addAsyncCleanup(client.close)
        self.assertEqual(self.channels[0].get_state(), ChannelConnectivity.SHUTDOWN)
        self.assertEqual((await client._transport.status.get()).status.identity.host_id, "intended-host")

    async def test_plain_url_uses_connection_budget(self) -> None:
        endpoint = await self.listening("intended-host")
        with patch("dokime_sdk._connection.wait_for", wraps=wait_for) as bounded:
            client = await connect(endpoint, connect_timeout=4.25)
        self.addAsyncCleanup(client.close)
        self.assertEqual(bounded.call_args.args[1], 4.25)
        self.assertEqual((await client._transport.status.get()).status.identity.host_id, "intended-host")

    async def test_connection_failure_preserves_the_remote_classification_and_cause(self) -> None:
        remote = Failure(
            classification="CLASSIFICATION_REFUSED",
            code="access",
            action="Export a New Pairing",
            cause={"classification": "CLASSIFICATION_PERMANENT", "code": "revoked", "details": {"key": "identity"}},
        )
        endpoint = await self.listening("intended-host", remote)
        with self.assertRaises(SdkError) as failed:
            await connect(endpoint, expected_backend_id="intended-host")
        self.assertEqual(failed.exception.failure.classification, "Refused")
        failure = failed.exception.failure.cause
        self.assertIsNotNone(failure)
        assert failure is not None and failure.cause is not None
        self.assertEqual((failure.code, failure.cause.code), ("access", "revoked"))
        self.assertEqual(failure.cause.details, {"key": "identity"})

    async def test_invalid_connection_budget(self) -> None:
        for value in (0.0, -1.0, float("inf"), float("nan")):
            with self.assertRaises(ValueError):
                await connect(self.refused(), connect_timeout=value)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
