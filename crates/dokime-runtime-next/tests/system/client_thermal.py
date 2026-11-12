# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Pure SDK Thermal Client Journey"""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
import sys
from pathlib import Path

# Local Imports
import dokime
from dokime.v1 import message

# Third Party Imports
from google.protobuf import descriptor_pool, message_factory

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def journey(phase, url, plugin_url, expected):
    # Connect with the Pure SDK's Generated Client Half
    client = await dokime.connect(url)
    try:
        execution = client.stubs["dokime.v1.Execution"]
        evidence = client.stubs["dokime.v1.Evidence"]
        telemetry = client.stubs["dokime.v1.Telemetry"]
        if phase == "read":
            # Read the Same Verified Execution, Records and Samples After Restart
            saved = message("ExecutionDetail")
            saved.ParseFromString(expected.read_bytes())
            identifier = saved.summary.execution_id
            detail = await execution.GetExecution(message("ExecutionGetExecutionRequest", execution_id=identifier))
            assert detail == saved
            records = await evidence.ListRecords(
                message(
                    "EvidenceListRecordsRequest",
                    selection={"scope": {"execution_id": identifier}},
                )
            )
            assert len(records.records) == 1 and records.records[0].payload
            samples = await telemetry.ListSamples(message("TelemetryListSamplesRequest", scope={"execution_id": identifier}))
            assert [sample.timestamp.seconds for sample in samples.samples] == [0, 1, 2]
            assert records.SerializeToString(deterministic=True) == expected.with_suffix(".records").read_bytes()
            assert samples.SerializeToString(deterministic=True) == expected.with_suffix(".samples").read_bytes()
            return

        # Add the Thermal URL and Read Its Catalog
        entry = await client.plugins.add(plugin_url)
        assert (await client.plugins.list()).plugins == [entry]
        catalog = client.stubs["dokime.v1.Catalog"]
        suites = await catalog.ListSuites(message("CatalogListSuitesRequest"))
        suite_id = suites.suites[0].suite_id
        suite = await catalog.GetSuite(message("CatalogGetSuiteRequest", suite_id=suite_id))
        assert suite.templates[0].metadata.identifier == "thermal.cycle"

        # Materialize Parameters and Start Their Frozen Origins
        planning = client.stubs["dokime.v1.Planning"]
        planned = await planning.MaterializeTemplate(
            message(
                "PlanningMaterializeTemplateRequest",
                selection={
                    "suite_id": suite_id,
                    "template_id": "thermal.cycle",
                    "parameters": [{"key": "limit", "value": {"values": [{"number": 85.0}]}}],
                },
            )
        )
        cycle = await execution.CreateCycle(
            message(
                "ExecutionCreateCycleRequest",
                continue_automatically=True,
                origins=[case.origin for case in planned.cases],
            )
        )
        identifier = cycle.execution_ids[0]
        observation = client.stubs["dokime.v1.Observation"]
        prompts = client.stubs["dokime.v1.Prompts"]
        progress = observation.WatchExecutionProgress(message("ObservationWatchExecutionProgressRequest", test_execution_id=identifier))

        # Await the Door Event and Answer Through the Prompts Service
        phases = []
        terminal = False
        async for frame in progress:
            if frame.WhichOneof("value") == "response":
                terminal = True
                continue
            phases.append(frame.item.progress.phase)
            if frame.item.progress.phase == "prompt":
                pending = await prompts.ListPending(message("PromptsListPendingRequest", execution_id=identifier))
                prompt_id = pending.prompts[0].prompt_id
                prompt = await prompts.GetPrompt(message("PromptsGetPromptRequest", prompt_id=prompt_id))
                assert prompt.prompt.prompt_id == prompt_id
                await prompts.Respond(
                    message(
                        "PromptsRespondRequest",
                        prompt_id=prompt_id,
                        action="yes",
                        accepted=True,
                    )
                )
        assert terminal and "prompt" in phases and "finished" in phases

        # Read the Sealed Execution, Record Payload and Controlled Stream Ticks
        detail = await execution.GetExecution(message("ExecutionGetExecutionRequest", execution_id=identifier))
        assert detail.summary.state == 3 and detail.seal
        assert (
            detail.outcome.evaluated.status
            == detail.outcome.evaluated.DESCRIPTOR.fields_by_name["status"].enum_type.values_by_name["RESULT_STATUS_PASSED"].number
        )
        query = client.stubs["dokime.v1.Query"]
        assert (await query.ListExecutions(message("QueryListExecutionsRequest"))).rows == [detail.summary]
        records = await evidence.ListRecords(
            message(
                "EvidenceListRecordsRequest",
                selection={"scope": {"execution_id": identifier}},
            )
        )
        assert len(records.records) == 1 and records.records[0].payload
        pool = descriptor_pool.DescriptorPool()
        pending = list(detail.schemas.catalog.types.file)
        loaded = set()
        while pending:
            ready = [definition for definition in pending if all(name in loaded for name in definition.dependency)]
            if not ready:
                raise RuntimeError("Decode Frozen Evidence: Descriptor Dependencies Are Incomplete")
            for definition in ready:
                pool.Add(definition)
                loaded.add(definition.name)
                pending.remove(definition)
        door_type = message_factory.GetMessageClass(pool.FindMessageTypeByName("thermal.v1.Door"))
        assert door_type.FromString(records.records[0].payload).closed
        samples = await telemetry.ListSamples(message("TelemetryListSamplesRequest", scope={"execution_id": identifier}))
        assert [sample.timestamp.seconds for sample in samples.samples] == [0, 1, 2]
        assert all(sample.payload for sample in samples.samples)
        assert all(sample.values.values["celsius"].number == 84.0 for sample in samples.samples)

        # Verify the Ordered Report Stream and Its Digest Completion
        reports = client.stubs["dokime.v1.Reports"]
        data = bytearray()
        completion = None
        terminal = False
        async for frame in reports.DownloadExecutionReport(message("ReportsDownloadExecutionReportRequest", execution_id=identifier)):
            if frame.WhichOneof("value") == "response":
                terminal = True
                continue
            kind = frame.item.payload.WhichOneof("kind")
            if kind == "chunk":
                assert frame.item.payload.chunk.offset == len(data)
                data.extend(frame.item.payload.chunk.data)
            if kind == "complete":
                completion = frame.item.payload.complete
        assert terminal and completion is not None
        assert completion.size_bytes == len(data)
        assert completion.sha256 == hashlib.sha256(data).hexdigest()
        assert identifier in data.decode()
        expected.write_bytes(detail.SerializeToString(deterministic=True))
        expected.with_suffix(".records").write_bytes(records.SerializeToString(deterministic=True))
        expected.with_suffix(".samples").write_bytes(samples.SerializeToString(deterministic=True))
    finally:
        await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(journey(sys.argv[1], sys.argv[2], sys.argv[3], Path(sys.argv[4])))
