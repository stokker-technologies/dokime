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

from google.protobuf.wrappers_pb2 import Int32Value
from harmos.v1 import types_pb2 as wire

# Third Party Imports
import dokime

# Local Imports
from dokime.v1 import message

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class MaterializationStory(unittest.IsolatedAsyncioTestCase):
    async def test_materialize_retry_and_cleanup(self):
        # Compose A Parameter-Dependent Template With Execution-Owned State.
        instances = []
        reading = dokime.Operation("example.reading.square", Int32Value, Int32Value, wire.VISIBILITY_PRIVATE)

        class Operations:
            @dokime.operation(reading.id, request=Int32Value, response=Int32Value, visibility=wire.VISIBILITY_PRIVATE)
            async def read(self, ctx, request):
                from operator import mul

                return Int32Value(value=mul(request.value, request.value))

        @dokime.suite(
            "example.channels",
            label="Channels",
            parameters=[message("Parameter", metadata={"identifier": "channels", "label": "Channels"}, type=7, element_type=2)],
        )
        class Channels:
            def __init__(self):
                self.channels = []
                self.attempts = []
                instances.append(self)

            @dokime.materialize
            def cases(cls, case, parameters):
                for channel in parameters["channels"]:
                    case.step(f"Channel {channel}", cls.measure, channel=channel)
                case.criterion("All Channels", cls.all_channels)

            @dokime.step("Measure", retries=1)
            async def measure(self, ctx, *, channel):
                self.attempts.append((channel, ctx.attempt))
                if channel == 2 and ctx.attempt == 1:
                    raise dokime.Failure(
                        wire.Failure(
                            classification=wire.CLASSIFICATION_TRANSIENT,
                            code="retry_measurement",
                            message="Retry measurement",
                        )
                    )
                measured = await ctx.operation(reading)(Int32Value(value=channel))
                self.channels.append(measured.value)

            @dokime.criterion("All Channels")
            def all_channels(self, ctx):
                return self.channels == [channel * channel for channel in ctx.parameters["channels"]]

        # Admit The Plugin Through A Real Product Host And Its Client Root.
        mechanical = Path(__file__).resolve().parents[2] / "examples/starter/tests/host.py"
        host = await runpy.run_path(str(mechanical))["Host"].start(self)
        plugin = dokime.Plugin("channels", "1.0.0", suites=[Channels], requires=[], operations=Operations())
        self.assertNotIn(reading.id, {operation.id for operation in plugin.catalog().operations})
        endpoint = await dokime.serve(plugin, address="127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)
        self.addAsyncCleanup(host.close)
        await host.client.plugins.add(endpoint.url)
        suites = await host.client.suites.list()
        parameters = {"channels": [[2, 4]]}
        first = await host.client.templates.materialize(suites[0].metadata.identifier, "example.channels", parameters)
        second = await host.client.templates.materialize(suites[0].metadata.identifier, "example.channels", parameters)
        self.assertEqual([case.origin for case in first], [case.origin for case in second])
        self.assertEqual(instances, [])
        case = first[0].origin.plugin_run.test_case
        self.assertEqual([item.metadata.identifier for item in case.steps], ["channel.2", "channel.4"])

        # Execute Twice; The Host Owns Retry, Scope And Cleanup.
        identifiers = []
        for _ in range(2):
            cycle = await host.client.cycles.start(first)
            identifiers.extend(cycle.executions)
            async for _progress in cycle.watch():
                pass
            execution = await host.client.executions.get(cycle.executions[0])
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
            dokime.Plugin("duplicate", "1.0.0", suites=[Channels, Channels]).catalog()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
