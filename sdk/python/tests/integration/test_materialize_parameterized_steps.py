# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Class Materialization and Scoped Execution State."""

# === Imports === #

# Standard Library Imports
import asyncio
import runpy
import unittest
from collections.abc import Mapping
from datetime import timedelta
from pathlib import Path
from typing import Any, ClassVar, cast

# Third Party Imports
from harmos import plugin
from harmos._wire.plugin.v1.invocation_pb2 import Request

# Local Imports
import dokime_sdk.operations
import dokime_sdk.templates
from dokime_sdk import Client, Failure, Plugin, ResultStatus, Scalar, Suite
from dokime_sdk.operations import OperationPolicy, RetryPolicy
from dokime_sdk.templates import (
    Context,
    Materialization,
    ParameterDefinition,
    Termination,
    TestTemplate,
    criterion,
    materialize,
    observer,
    step,
    stream,
)

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

            async def initialize(self, ctx: Context) -> None:
                self.measured: list[int] = []
                cast(ChannelsPlugin, ctx.plugin).events.append((ctx.execution, "initialize"))

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
            origin = {"plugin_run": {"plugin": author.identifier, "template": template.identifier, "test_case": case}}
            transport = plugin.Context(Request(scope=execution, attempt=1), author.identifier, {})
            for name, arguments in (
                ("initialize", {}),
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
            author.events, [(execution, item) for execution in ("first", "second") for item in ("initialize", 2, 4, Termination.COMPLETED)]
        )
        self.assertFalse(author._sessions)

    async def test_plugin_observers_use_the_actual_plugin_without_execution_fields(self) -> None:
        contexts: list[dokime_sdk.operations.Context] = []

        class ObserverPlugin(Plugin):
            version = "1.0.0"

            def __init__(self) -> None:
                self.reading_count = 0
                super().__init__()

            @stream("Reading", row=int)
            @observer()
            async def reading(self, ctx: dokime_sdk.operations.Context) -> int:
                contexts.append(ctx)
                self.reading_count += 1
                return self.reading_count

        author = ObserverPlugin()
        definition, handler, observing = author._plugin._registrations()[1][0]
        self.assertTrue(observing)
        self.assertEqual(definition.id, "observer.reading")
        transport = plugin.Context(Request(attempt=1), author.identifier, {})
        for expected in (1, 2):
            result = await cast(Any, handler)(transport)
            self.assertEqual(result.value, expected)
        for context in contexts:
            self.assertIs(context.plugin, author)
            self.assertFalse(hasattr(context, "execution"))
            self.assertFalse(hasattr(context, "parameters"))

    async def test_cleanup_retains_partial_state_after_failed_or_cancelled_initialization(self) -> None:
        for failure in (ValueError, asyncio.CancelledError):
            events: list[object] = []

            class PartialTestTemplate(TestTemplate):
                async def initialize(self, ctx: Context) -> None:
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
            origin = {"plugin_run": {"plugin": author.identifier, "template": template.identifier, "test_case": case}}
            transport = plugin.Context(Request(scope="execution", attempt=1), author.identifier, {})
            handlers = {definition.id: (definition, handler) for definition, handler in author._plugin._registrations()[0]}
            definition, handler = handlers["partial.partial.initialize"]
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


class NativeAuthoringStory(unittest.IsolatedAsyncioTestCase):
    async def test_counts_keep_the_host_resolved_values_through_materialization(self) -> None:
        class DurationTestTemplate(TestTemplate):
            parameters: ClassVar[list[ParameterDefinition]] = [
                dokime_sdk.templates.Parameter.count("duration_min", "Duration").unit("min").default(240).range(1).build()
            ]

            @step("Wait")
            async def wait(self, ctx: Context) -> None:
                pass

            @criterion("Waited")
            async def waited(self, ctx: Context) -> bool:
                return True

        class DurationSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [DurationTestTemplate]

        class DurationPlugin(Plugin):
            version = "1.0.0"
            suites: ClassVar[list[type[Suite]]] = [DurationSuite]

        mechanical = Path(__file__).resolve().parents[2] / "tests/support/host.py"
        host = await runpy.run_path(str(mechanical))["Host"].start(self)
        client = cast(Client, host.client)
        registered = await client.plugins.register(DurationPlugin())
        defaulted = await client.cases.materialize(DurationTestTemplate)
        chosen = await client.cases.materialize(DurationTestTemplate, parameters={"duration_min": [5]})
        self.assertEqual([len(defaulted), len(chosen)], [1, 1])
        # Steps Rebuild the Case From the Frozen Values, So a Count Must Survive Execution Too.
        cycle = await client.cycle(label="Durations")
        execution = await cycle.execution(defaulted[0])
        async for _ in execution.run():
            pass
        self.assertEqual((await execution.seal()).status, ResultStatus.PASSED)
        await registered.unregister()

    async def test_class_template_materialization_retry_and_artifacts_through_host(self) -> None:
        instances: list[list[int]] = []
        attempts: list[tuple[int, int]] = []
        lifecycle: list[tuple[str, object]] = []

        class ChannelsTestTemplate(TestTemplate):
            parameters: ClassVar[list[ParameterDefinition]] = [
                ParameterDefinition(metadata={"identifier": "channels"}, type="VALUE_TYPE_LIST", element_type="VALUE_TYPE_INTEGER")
            ]

            def __init__(self) -> None:
                self.channels: list[int] = []

            @materialize
            def channels_case(self, case: Materialization, parameters: Mapping[str, Scalar]) -> None:
                channels = parameters["channels"]
                if not isinstance(channels, list):
                    raise TypeError("Materialize Channels: Supply a List")
                for channel in channels:
                    case.step("Measure", self.measure, identifier=f"channel_{channel}", channel=channel)
                case.criterion("All Channels", self.all_channels)

            async def initialize(self, ctx: Context) -> None:
                instances.append(self.channels)

            @step("Measure", policy=OperationPolicy(retries=RetryPolicy(2, timedelta(milliseconds=100))))
            async def measure(self, ctx: Context, *, channel: int) -> None:
                attempts.append((channel, ctx.attempt))
                if channel == 2 and ctx.attempt == 1:
                    raise dokime_sdk.operations.FailureError(
                        Failure("Transient", "retry_measurement", "Measurement Requires Another Attempt", "Retry Measurement")
                    )
                measured = channel * channel
                await ctx.artifact(f"channel_{channel}.bin", measured.to_bytes(4, "big"))
                self.channels.append(measured)

            @criterion("All Channels")
            async def all_channels(self, ctx: Context) -> bool:
                return self.channels == [4, 16]

        class ChannelsSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [ChannelsTestTemplate]

        class ChannelsPlugin(Plugin):
            version = "1.0.0"
            suites: ClassVar[list[type[Suite]]] = [ChannelsSuite]

            async def initialize(self, ctx: dokime_sdk.operations.Context) -> None:
                lifecycle.append(("initialize", ctx.plugin))

            async def finalize(self, ctx: dokime_sdk.operations.Context) -> None:
                lifecycle.append(("finalize", ctx.plugin))

        mechanical = Path(__file__).resolve().parents[2] / "tests/support/host.py"
        host = await runpy.run_path(str(mechanical))["Host"].start(self)
        client = cast(Client, host.client)
        author = ChannelsPlugin()
        registered = await client.plugins.register(author)
        self.assertEqual(lifecycle, [("initialize", author)])
        cases = await client.cases.materialize(ChannelsTestTemplate, parameters={"channels": [[2, 4]]})
        self.assertEqual(len(cases), 1)
        self.assertEqual(instances, [])
        for _ in range(2):
            cycle = await client.cycle(label="Channel Measurements")
            execution = await cycle.execution(cases[0])
            async for _ in execution.run():
                pass
            outcome = await execution.seal()
            self.assertEqual(outcome.status, ResultStatus.PASSED)
            await cycle.seal()
            self.assertTrue(await cycle.evidence())
        self.assertEqual(instances, [[4, 16], [4, 16]])
        self.assertEqual(attempts, [(2, 1), (2, 2), (4, 1)] * 2)
        self.assertFalse(author._sessions)
        await registered.unregister()
        self.assertEqual(lifecycle, [("initialize", author), ("finalize", author)])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
