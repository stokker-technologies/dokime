# SPDX-License-Identifier: Apache-2.0

"""Log Records Share the Host's Evidence Order and Execution Scope."""

# === Imports === #

# Standard Library Imports
import asyncio
import io
import logging
from unittest import IsolatedAsyncioTestCase

# Third Party Imports
from google.protobuf.message import Message
from harmos import OperationPolicy, plugin
from harmos._wire.host.v1.time_pb2 import TimeGetResponse
from harmos._wire.plugin.v1.invocation_pb2 import Request

# Local Imports
from dokime_sdk import Plugin, Suite, _logs
from dokime_sdk._generated import shared
from dokime_sdk._wire.host.v1.evidence.logs_pb2 import LogsAppendRequest, LogsAppendResponse
from dokime_sdk._wire.host.v1.evidence.records_pb2 import RecordsAppendRequest, RecordsAppendResponse
from dokime_sdk.templates import TestTemplate, record
from dokime_sdk.templates._context import Context

# Type Checking Imports

# === Public Exports === #

# === Constants === #

log = logging.getLogger(__name__)

# === Types === #

# === Classes === #


class LogTemplate(TestTemplate):
    reading = record(int, identifier="reading")


class LogSuite(Suite):
    templates = (LogTemplate,)


class LogPlugin(Plugin):
    version = "1.0.0"
    suites = (LogSuite,)
    records = (LogTemplate.reading,)


class RecordingTransport(plugin.Context):
    def __init__(self, scope: str = "execution") -> None:
        super().__init__(Request(scope=scope, attempt=1), "log", {})
        self.evidence: list[LogsAppendRequest | RecordsAppendRequest] = []

    async def _call[Q: Message, R: Message](
        self, operation: plugin.Operation[Q, R], request: Q, *, policy: OperationPolicy | None = None
    ) -> R:
        response = operation.response()
        if isinstance(response, TimeGetResponse):
            response.now.SetInParent()
        if isinstance(request, (LogsAppendRequest, RecordsAppendRequest)):
            self.evidence.append(request)
        if isinstance(response, LogsAppendResponse):
            response.receipt.SetInParent()
        if isinstance(response, RecordsAppendResponse):
            response.ordinal = len(self.evidence)
        return response


class LoggingTests(IsolatedAsyncioTestCase):
    def context(self, transport: RecordingTransport) -> Context:
        owner = LogPlugin()
        return Context(transport, owner, {}, owner._suites[0].templates[0])

    async def test_logs_flush_before_evidence_and_on_handler_return(self) -> None:
        transport = RecordingTransport()
        context = self.context(transport)

        async def execute() -> None:
            log.info("Before %s", "Record")
            await context.record(LogTemplate.reading, 42)
            log.warning("After Record")

        await _logs.run(context, execute)
        first, reading, last = transport.evidence
        self.assertIsInstance(first, LogsAppendRequest)
        self.assertIsInstance(reading, RecordsAppendRequest)
        self.assertIsInstance(last, LogsAppendRequest)
        assert isinstance(first, LogsAppendRequest) and isinstance(last, LogsAppendRequest)
        self.assertEqual((first.message, last.message), ("Before Record", "After Record"))
        self.assertEqual((first.severity, last.severity), (shared.SEVERITY_INFO, shared.SEVERITY_WARNING))
        self.assertEqual((first.execution, last.execution), ("execution", "execution"))

    async def test_exception_flushes_all_levels_and_restores_the_scope(self) -> None:
        transport = RecordingTransport()
        context = self.context(transport)

        async def execute() -> None:
            log.info("Info")
            log.error("Error")
            log.critical("Critical")
            raise ValueError("Invalid Reading")

        with self.assertRaisesRegex(ValueError, "Invalid Reading"):
            await _logs.run(context, execute)
        logs = [value for value in transport.evidence if isinstance(value, LogsAppendRequest)]
        self.assertEqual([value.severity for value in logs], [shared.SEVERITY_INFO, shared.SEVERITY_FAILURE, shared.SEVERITY_FAILURE])
        self.assertIsNone(_logs._CURRENT.get())

    async def test_library_debug_stays_out_of_evidence_while_plugin_info_reaches_it(self) -> None:
        # Start From Python's Default Levels Whatever Earlier Tests Configured.
        root, package = logging.getLogger(), logging.getLogger(__name__.partition(".")[0])
        for logger, level in ((root, root.level), (package, package.level)):
            self.addCleanup(logger.setLevel, level)
        root.setLevel(logging.WARNING)
        package.setLevel(logging.NOTSET)
        transport = RecordingTransport()
        context = self.context(transport)

        async def execute() -> None:
            logging.getLogger("library").debug("Library Detail")
            log.debug("Plugin Detail")
            log.info("Plugin Progress")

        await _logs.run(context, execute)
        self.assertEqual([value.message for value in transport.evidence if isinstance(value, LogsAppendRequest)], ["Plugin Progress"])

    async def test_concurrent_handlers_keep_their_own_scopes(self) -> None:
        first, second = RecordingTransport("first"), RecordingTransport("second")
        contexts = (self.context(first), self.context(second))
        entered, release = asyncio.Event(), asyncio.Event()

        async def left() -> None:
            log.info("First")
            entered.set()
            await release.wait()

        async def right() -> None:
            await entered.wait()
            log.info("Second")
            release.set()

        await asyncio.gather(_logs.run(contexts[0], left), _logs.run(contexts[1], right))
        for transport, expected in ((first, "first"), (second, "second")):
            self.assertEqual([value.execution for value in transport.evidence], [expected])

    async def test_unscoped_logging_goes_to_plugin_diagnostics(self) -> None:
        output = io.StringIO()
        handler = _logs._Handler(output)
        record = logging.LogRecord("example", logging.WARNING, "", 0, "Store the Token", (), None)
        handler.emit(record)
        self.assertEqual(output.getvalue(), "Store the Token\n")


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
