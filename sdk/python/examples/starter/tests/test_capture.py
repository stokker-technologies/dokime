# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""An Operator Serves, Materializes And Runs The Starter With A Host Clock."""

# === Imports === #

# Standard Library Imports
import unittest

import grpc
from google.protobuf.timestamp_pb2 import Timestamp
from google.protobuf.wrappers_pb2 import BoolValue, DoubleValue
from harmos.v1 import plugin_pb2 as wire
from harmos.v1.plugin_pb2_grpc import PluginStub
from host import invoke
from suite import plugin

# Third Party Imports
import dokime

# Local Imports
from dokime.v1 import message, message_type

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class CaptureStory(unittest.IsolatedAsyncioTestCase):
    async def test_serve_capture_and_release(self):
        # Serve The Remote Plugin And Read Its Typed Catalog.
        endpoint = await dokime.serve(plugin(), address="127.0.0.1:0")
        self.addAsyncCleanup(endpoint.close)
        channel = grpc.aio.insecure_channel(endpoint.address)
        self.addAsyncCleanup(channel.close)
        client = PluginStub(channel)
        description = await client.Describe(wire.DescribeRequest(revision=1))
        suites = message_type("Suites").FromString(
            next(item.payload for item in description.catalog.sections if item.type == "dokime.v1.Suites")
        )
        template = suites.suites[0].templates[0]
        self.assertEqual(len(template.documents[0].sections[0].cards), 2)

        # Materialize A Resolved Parameter Set Without Creating An Execution.
        case = await invoke(
            client,
            template.materialize,
            message("MaterializeRequest", template=template.metadata.identifier, parameters={"limit": {"number": 12}}),
            message_type("TestCase"),
        )
        self.assertEqual([item.metadata.identifier for item in case.steps], ["measure", "confirm"])
        origin = message(
            "Origin",
            plugin_run={
                "plugin": description.id,
                "template": template.metadata.identifier,
                "test_case": case,
                "digest": description.catalog.digest,
                "started_at": description.started_at,
            },
        )
        envelope = {"execution": "execution-one", "test_case": case, "origin": origin}

        # Initialize One Execution-Owned Suite Instance.
        await invoke(
            client, template.initialize, message("InitializeRequest", **envelope), message_type("InitializeResponse"), scope="execution-one"
        )
        records = []
        calls = []
        requirements = {item.id: item.fingerprint for item in description.catalog.requires}

        async def host(child):
            self.assertEqual(child.fingerprint, requirements[child.operation])
            calls.append(child.operation)
            if child.operation == "dokime.execution.record":
                request = message_type("PublishRecord").FromString(child.payload)
                self.assertEqual(request.execution, "execution-one")
                records.append(request.payload)
                return message("RecordReceipt", ordinal=len(records))
            if child.operation == "dokime.execution.prompt":
                question = message_type("OperatorQuestion").FromString(child.payload)
                return message("OperatorAnswer", prompt=question.prompt, action=question.definition.actions[0].identifier, accepted=True)
            if child.operation == "harmos.clock.sleep":
                self.assertEqual(wire.Sleep.FromString(child.payload).duration.seconds, 1)
                return wire.SleepResponse()
            if child.operation == "dokime.execution.statistics":
                request = message_type("StatisticsRequest").FromString(child.payload)
                self.assertEqual((request.stream, request.step), ("example.reading", "measure"))
                return message("Statistics", count=1, min=11, max=11, mean=11)
            if child.operation == "dokime.execution.records":
                return message("Records", count=len(records), last={"kind": "example.confirmation", "payload": records[-1]})
            if child.operation == "dokime.execution.progress":
                return message("ReportProgressResponse")
            if child.operation == "dokime.execution.log":
                return message("WriteLogResponse")
            raise ValueError(f"Unexpected host operation: {child.operation}")

        # Run Measure And Read Its Observer At A Fixed Host Tick.
        await invoke(
            client,
            case.steps[0].operation,
            message("StepRequest", step="measure", **envelope),
            message_type("StepResponse"),
            scope="execution-one",
            host=host,
        )
        publication = client.Publish()
        await publication.write(wire.PublishFrame(open=wire.PublishRequest(stream=case.streams[0], scope="execution-one")))
        await publication.write(wire.PublishFrame(tick=wire.Tick(at=Timestamp(seconds=123))))
        sample = (await publication.read()).samples[0]
        self.assertEqual((sample.at.seconds, DoubleValue.FromString(sample.row).value), (123, 11))
        publication.cancel()

        # Answer The Prompt And Evaluate Against Acknowledged Evidence.
        await invoke(
            client,
            case.steps[1].operation,
            message("StepRequest", step="confirm", **envelope),
            message_type("StepResponse"),
            scope="execution-one",
            host=host,
        )
        result = await invoke(
            client,
            case.criteria[0].operation,
            message("CriterionRequest", criterion="within.limit", **envelope),
            message_type("CriterionResponse"),
            scope="execution-one",
            host=host,
        )
        self.assertTrue(result.passed)
        self.assertTrue(all(BoolValue.FromString(payload).value for payload in records))

        # Release Resources Through Cleanup And Stop The Remote Plugin.
        await invoke(
            client,
            template.cleanup,
            message("CleanupRequest", termination=1, **envelope),
            message_type("CleanupResponse"),
            scope="execution-one",
            host=host,
        )
        self.assertFalse(endpoint.plugin._sessions)
        self.assertEqual(calls[-1], "dokime.execution.log")
        await client.Stop(wire.StopRequest(reason="Finished"))
        self.assertEqual((await client.Describe(wire.DescribeRequest(revision=1))).started_at, description.started_at)
        await endpoint.close()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
