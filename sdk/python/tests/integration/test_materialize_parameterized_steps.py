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
import runpy
import unittest
from pathlib import Path
from uuid import uuid4

from google.protobuf.wrappers_pb2 import Int32Value
from harmos import CLASSIFICATION_TRANSIENT, VISIBILITY_PRIVATE, FailureValue
from harmos.client import PluginPlacement, PluginRemote, PluginValue
from harmos.plugin import Context as InvocationContext
from harmos.plugin import Operation, operation

# Third Party Imports
import dokime
from dokime import Metadata, Parameter
from dokime.authoring import Case, criterion, materialize, step, suite
from dokime.client import ExecutionBinding, RegisteredSelection, SelectionPlan
from dokime.plugin import Context
from dokime.values import Scalar, encode

# Local Imports
# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class MaterializationStory(unittest.IsolatedAsyncioTestCase):
    async def test_materialize_retry_and_cleanup(self) -> None:
        # Compose A Parameter-Dependent Template With Execution-Owned State.
        instances = []
        reading = Operation("example.reading.square", Int32Value, Int32Value, VISIBILITY_PRIVATE)

        class Operations:
            @operation(reading.id, request=Int32Value, response=Int32Value, visibility=VISIBILITY_PRIVATE)
            async def read(self, ctx: InvocationContext, request: Int32Value) -> Int32Value:
                from operator import mul

                return Int32Value(value=mul(request.value, request.value))

        @suite(
            "example.channels",
            label="Channels",
            parameters=[
                Parameter(
                    metadata={"identifier": "channels", "label": "Channels"}, type="VALUE_TYPE_LIST", element_type="VALUE_TYPE_INTEGER"
                )
            ],
        )
        class Channels:
            def __init__(self) -> None:
                self.channels = []
                self.attempts = []
                instances.append(self)

            @materialize
            def cases(cls, case: Case, parameters: dict[str, Scalar]) -> None:
                channels = parameters["channels"]
                if not isinstance(channels, list):
                    raise TypeError("Materialize Channels: Supply a List")
                for channel in channels:
                    case.step(f"Channel {channel}", cls.measure, channel=channel)
                case.criterion("All Channels", cls.all_channels)

            @step("Measure", retries=1)
            async def measure(self, ctx: Context, *, channel: int) -> None:
                self.attempts.append((channel, ctx.attempt))
                if channel == 2 and ctx.attempt == 1:
                    raise dokime.Failure(
                        FailureValue(
                            classification=CLASSIFICATION_TRANSIENT,
                            code="retry_measurement",
                            message="Retry measurement",
                        )
                    )
                measured = await ctx.operation(reading)(Int32Value(value=channel))
                self.channels.append(measured.value)

            @criterion("All Channels")
            def all_channels(self, ctx: Context) -> bool:
                channels = ctx.parameters["channels"]
                if not isinstance(channels, list) or not all(isinstance(channel, int) for channel in channels):
                    raise TypeError("Evaluate Channels: Supply Integer Channels")
                return self.channels == [int(channel) * int(channel) for channel in channels if isinstance(channel, int)]

        # Admit The Plugin Through A Real Product Host And Its Client Root.
        mechanical = Path(__file__).resolve().parents[3] / "tests/fixtures/python/starter/host.py"
        host = await runpy.run_path(str(mechanical))["Host"].start(self)
        plugin = dokime.Plugin("channels", "1.0.0", suites=[Channels], requires=[], operations=Operations())
        self.assertNotIn(reading.id, {operation.id for operation in plugin.declarations().operations})
        endpoint = await plugin.start("127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)
        self.addAsyncCleanup(host.close)
        await host.client.plugins.create("channels", PluginPlacement(remote=PluginRemote(url=endpoint.url)), PluginValue())
        suites = (await host.client.suites.list()).suites
        plan = SelectionPlan(
            selections=[
                RegisteredSelection(
                    suite_id=suites[0].metadata.identifier,
                    template_id="example.channels",
                    parameters=[{"key": "channels", "value": {"values": [encode([2, 4])]}}],
                )
            ]
        )
        first = (await host.client.cases.create(plan, True)).cases
        second = (await host.client.cases.create(plan, True)).cases
        self.assertEqual([case.origin for case in first], [case.origin for case in second])
        self.assertEqual(instances, [])
        case = first[0].origin.plugin_run.test_case
        self.assertEqual([item.metadata.identifier for item in case.steps], ["channel.2", "channel.4"])

        # Execute Twice; The Host Owns Retry, Scope And Cleanup.
        identifiers = []
        for _ in range(2):
            registered = await host.client.cases.create(plan, False)
            cycle = (await host.client.cycles.create([], Metadata(), True, "")).cycle
            identifier = str(uuid4())
            identifiers.append(identifier)
            await host.client.executions.create(
                cycle.cycle_id, [ExecutionBinding(execution_id=identifier, case_id=registered.cases[0].case_id)]
            )
            finished = False
            async for progress in host.client.executions.progress.watch(identifier):
                finished = progress.finished
            self.assertTrue(finished)
            execution = (await host.client.executions.get(identifier)).execution
            self.assertEqual(
                execution.outcome.evaluated.status,
                execution.outcome.evaluated.DESCRIPTOR.fields_by_name["status"].enum_type.values_by_name["RESULT_STATUS_PASSED"].number,
                execution,
            )
        self.assertEqual(len(set(identifiers)), 2)
        self.assertEqual([item.channels for item in instances], [[4, 16], [4, 16]])
        self.assertEqual([item.attempts for item in instances], [[(2, 1), (2, 2), (4, 1)]] * 2)
        self.assertFalse(endpoint.plugin._sessions)
        with self.assertRaises(ValueError):
            dokime.Plugin("duplicate", "1.0.0", suites=[Channels, Channels]).declarations()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
