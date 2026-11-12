# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""An Author Materializes Parameterized Steps And Isolates Execution State."""

# === Imports === #

# Standard Library Imports
import runpy
import unittest
from pathlib import Path

import grpc
from harmos.v1 import plugin_pb2 as wire
from harmos.v1.plugin_pb2_grpc import PluginStub

# Third Party Imports
import dokime

# Local Imports
from dokime.v1 import message, message_type

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class MaterializationStory(unittest.IsolatedAsyncioTestCase):
    async def test_materialize_retry_and_cleanup(self):
        # Compose A Parameter-Dependent Template With Suite-Owned State.
        instances = []

        @dokime.suite("example.channels", label="Channels")
        class Channels:
            def __init__(self):
                self.channels = []
                instances.append(self)

            @dokime.materialize
            def cases(cls, case, parameters):
                for channel in parameters["channels"]:
                    case.step(f"Channel {channel}", cls.measure, channel=channel)
                case.criterion("All Channels", cls.all_channels)

            @dokime.step("Measure", retries=1)
            def measure(self, ctx, *, channel):
                if channel == 2 and ctx.attempt == 1:
                    raise dokime.Failure(
                        wire.Failure(classification=wire.CLASSIFICATION_TRANSIENT, code="retry_measurement", message="Retry measurement")
                    )
                self.channels.append(channel)

            @dokime.criterion("All Channels")
            def all_channels(self, ctx):
                return self.channels == ctx.parameters["channels"]

        mechanical = Path(__file__).resolve().parents[2] / "examples/starter/tests/host.py"
        invoke = runpy.run_path(str(mechanical))["invoke"]
        endpoint = await dokime.serve(Channels, address="127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)
        channel = grpc.aio.insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        client = PluginStub(channel)

        # Materialization Is Repeatable And Does Not Construct Resources.
        request = message(
            "MaterializeRequest",
            template="example.channels",
            parameters={"channels": {"list": {"values": [{"integer": 2}, {"integer": 4}]}}},
        )
        first = await invoke(client, "example.channels.materialize", request, message_type("TestCase"))
        second = await invoke(client, "example.channels.materialize", request, message_type("TestCase"))
        self.assertEqual(first, second)
        self.assertEqual(instances, [])
        self.assertEqual([item.metadata.identifier for item in first.steps], ["channel.2", "channel.4"])
        description = await client.Describe(wire.DescribeRequest(revision=1))
        contract = next(item for item in description.catalog.operations if item.id == first.steps[0].operation)
        self.assertEqual(contract.policy.attempts, 2)

        # Run Two Executions Without An Initialize Hook And Keep Them Separate.
        for execution in ("one", "two"):
            origin = message("Origin", plugin_run={"plugin": description.id, "template": "example.channels", "test_case": first})
            envelope = {"execution": execution, "test_case": first, "origin": origin}
            for item in first.steps:
                if item.metadata.identifier == "channel.2":
                    with self.assertRaises(dokime.Failure) as failed:
                        await invoke(
                            client,
                            item.operation,
                            message("StepRequest", step=item.metadata.identifier, **envelope),
                            message_type("StepResponse"),
                            scope=execution,
                        )
                    self.assertEqual(failed.exception.value.classification, wire.CLASSIFICATION_TRANSIENT)
                await invoke(
                    client,
                    item.operation,
                    message("StepRequest", step=item.metadata.identifier, **envelope),
                    message_type("StepResponse"),
                    scope=execution,
                    attempt=2,
                )
            result = await invoke(
                client,
                first.criteria[0].operation,
                message("CriterionRequest", criterion="all.channels", **envelope),
                message_type("CriterionResponse"),
                scope=execution,
            )
            self.assertTrue(result.passed)
            await invoke(
                client, "example.channels.cleanup", message("CleanupRequest", **envelope), message_type("CleanupResponse"), scope=execution
            )
        self.assertEqual([instance.channels for instance in instances], [[2, 4], [2, 4]])
        self.assertFalse(endpoint.plugin._sessions)

        # Refuse An Execution Envelope With A Different Harmos Scope.
        with self.assertRaises(grpc.aio.AioRpcError) as refused:
            await invoke(
                client,
                first.steps[0].operation,
                message("StepRequest", step="channel.2", **envelope),
                message_type("StepResponse"),
                scope="another",
            )
        self.assertEqual(refused.exception.code(), grpc.StatusCode.INTERNAL)
        with self.assertRaises(ValueError):
            dokime.Plugin("duplicate", "1.0.0", suites=[Channels, Channels]).catalog()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
