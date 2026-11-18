# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""An Author Materializes, Retries And Releases Isolated Executions Through A Host."""

# === Imports === #

# Standard Library Imports
import asyncio
import unittest
from collections.abc import Mapping
from typing import Any, ClassVar, cast

# Third Party Imports
from harmos import plugin
from harmos._wire.plugin.v1.invocation_pb2 import Request

# Local Imports
from dokime_sdk import Plugin, Scalar, Suite
from dokime_sdk.templates import Context, Materialization, Termination, TestTemplate, criterion, materialize, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class MaterializationStory(unittest.IsolatedAsyncioTestCase):
    async def test_scoped_class_handlers_share_execution_state_and_plugin(self) -> None:
        class ChannelsTestTemplate(TestTemplate):
            @materialize
            def channels(self, case: Materialization, parameters: Mapping[str, Scalar]) -> None:
                channels = parameters["channels"]
                if not isinstance(channels, list):
                    raise TypeError("Materialize Channels: Supply a List")
                for channel in channels:
                    case.step("Read", self.measure, identifier=f"channel_{channel}", channel=channel)
                case.criterion("All Channels", self.all_channels)

            async def setup(self, ctx: Context) -> None:
                self.measured: list[int] = []
                cast(ChannelsPlugin, ctx.plugin).events.append((ctx.execution, "setup"))

            @step("Measure")
            async def measure(self, ctx: Context, *, channel: int) -> None:
                self.measured.append(channel * channel)
                cast(ChannelsPlugin, ctx.plugin).events.append((ctx.execution, channel))

            @criterion("All Channels")
            async def all_channels(self, ctx: Context) -> bool:
                return self.measured == [4, 16]

            async def cleanup(self, ctx: Context) -> None:
                cast(ChannelsPlugin, ctx.plugin).events.append((ctx.execution, ctx.termination))

        class ChannelsSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [ChannelsTestTemplate]

        class ChannelsPlugin(Plugin):
            version = "1.0.0"
            suites: ClassVar[list[type[Suite]]] = [ChannelsSuite]

            def __init__(self) -> None:
                self.events: list[tuple[str, object]] = []
                super().__init__()

        author = ChannelsPlugin()
        template = author._suites[0].templates[0]
        case = template.build({"channels": [2, 4]}).message
        self.assertEqual([item.operation for item in case.steps], ["channels.channels.measure"] * 2)
        handlers = {definition.id: (definition, handler) for definition, handler in author._plugin._registrations()[0]}
        for execution in ("first", "second"):
            origin = {"plugin_run": {"plugin": author._identifier, "template": template.identifier, "test_case": case}}
            transport = plugin.Context(Request(scope=execution, attempt=1), author._identifier, {})
            for name, arguments in (
                ("setup", {}),
                ("measure", {"step": "channels.channels.channel_2"}),
                ("measure", {"step": "channels.channels.channel_4"}),
                ("all_channels", {"criterion": "channels.channels.all_channels"}),
                ("cleanup", {"termination": Termination.COMPLETED.value}),
            ):
                definition, handler = handlers[f"channels.channels.{name}"]
                request = cast(Any, definition.request)(execution=execution, test_case=case, origin=origin, **arguments)
                result = await handler(transport, request)
                if name == "all_channels":
                    self.assertTrue(cast(Any, result).passed)
        self.assertEqual(
            author.events, [(execution, item) for execution in ("first", "second") for item in ("setup", 2, 4, Termination.COMPLETED)]
        )
        self.assertFalse(author._sessions)

    async def test_cleanup_retains_partial_state_after_failed_or_cancelled_setup(self) -> None:
        for failure in (ValueError, asyncio.CancelledError):
            events: list[object] = []

            class PartialTestTemplate(TestTemplate):
                async def setup(self, ctx: Context) -> None:
                    self.device = "partial"
                    cast(PartialPlugin, ctx.plugin).events.append(ctx.plugin)
                    raise cast(PartialPlugin, ctx.plugin).failure_type()

                async def cleanup(self, ctx: Context) -> None:
                    cast(PartialPlugin, ctx.plugin).events.append((self.device, ctx.termination))

            class PartialSuite(Suite):
                templates: ClassVar[list[type[TestTemplate]]] = [PartialTestTemplate]

            class PartialPlugin(Plugin):
                version = "1.0.0"
                suites: ClassVar[list[type[Suite]]] = [PartialSuite]

                def __init__(self) -> None:
                    self.events: list[object] = []
                    self.failure_type: type[BaseException] = ValueError
                    super().__init__()

            author = PartialPlugin()
            author.events = events
            author.failure_type = failure
            template = author._suites[0].templates[0]
            case = template.build({}).message
            origin = {"plugin_run": {"plugin": author._identifier, "template": template.identifier, "test_case": case}}
            transport = plugin.Context(Request(scope="execution", attempt=1), author._identifier, {})
            handlers = {definition.id: (definition, handler) for definition, handler in author._plugin._registrations()[0]}
            definition, handler = handlers["partial.partial.setup"]
            request = cast(Any, definition.request)(execution="execution", test_case=case, origin=origin)
            try:
                await handler(transport, request)
            except failure:
                pass
            else:
                self.fail("Setup Must Preserve Its Failure")
            definition, handler = handlers["partial.partial.cleanup"]
            request = cast(Any, definition.request)(
                execution="execution", test_case=case, origin=origin, termination=Termination.FAILED.value
            )
            await handler(transport, request)
            self.assertEqual(events, [author, ("partial", Termination.FAILED)])
            self.assertFalse(author._sessions)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
