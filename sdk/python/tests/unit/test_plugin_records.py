# SPDX-License-Identifier: Apache-2.0

"""Plugin Records Keep One Identity Across Operation and Template Contexts."""

# === Imports === #

from unittest import IsolatedAsyncioTestCase
from unittest.mock import AsyncMock

from google.protobuf.timestamp_pb2 import Timestamp
from harmos import plugin
from harmos._wire.plugin.v1.invocation_pb2 import Request

from dokime_sdk import Plugin, Suite
from dokime_sdk._wire.host.v1.evidence.records_pb2 import RecordsAppendResponse
from dokime_sdk._wire.model.v1.record_contracts_pb2 import RecordContracts
from dokime_sdk.operations import Context
from dokime_sdk.templates import TestTemplate, record

# === Classes === #

EVENT = record(int, identifier="event", label="Event")


class CaptureTestTemplate(TestTemplate):
    pass


class CaptureSuite(Suite):
    templates = (CaptureTestTemplate,)


class EventPlugin(Plugin):
    version = "1.0.0"
    records = (EVENT,)
    suites = (CaptureSuite,)


class PluginRecordTests(IsolatedAsyncioTestCase):
    def test_records_are_declared_once_at_plugin_scope(self) -> None:
        owner = EventPlugin()
        section = next(item for item in owner.declarations().sections if item.type == RecordContracts.DESCRIPTOR.full_name)
        records = RecordContracts.FromString(section.payload).records
        self.assertEqual([item.metadata.identifier for item in records], ["event.event"])
        self.assertEqual(owner._suites[0].templates[0].resolve(EVENT), "event.event")

    async def test_observer_record_calls_leave_recipient_selection_to_the_host(self) -> None:
        owner = EventPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        transport.now = AsyncMock(return_value=Timestamp(seconds=1))
        transport._call = AsyncMock(return_value=RecordsAppendResponse(ordinal=7))
        result = await Context(transport, owner).record(EVENT, 42, request_id="event")
        self.assertEqual(result, 7)
        call = transport._call.await_args
        assert call is not None
        request = call.args[1]
        self.assertEqual(request.execution, "")
        self.assertEqual(request.kind, "event.event")
        self.assertEqual(request.request_id, "event")
        self.assertEqual(request.source_at, Timestamp(seconds=1))
