# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Pure SDK Thermal Client Journey"""

# === Imports === #

import asyncio
import sys
from pathlib import Path

from dokime import (
    EvidenceCapturedRecordSelection,
    EvidenceRecordScope,
    EvidenceScope,
    MapValue,
    Metadata,
    Value,
    Window,
)
from dokime.client import (
    PLUGIN_STATE_READY,
    RESULT_STATUS_PASSED,
    Client,
    EvidenceBucketRequest,
    ExecutionDetail,
    ExecutionFilter,
    ExecutionOrder,
    ExecutionSelection,
    PluginPlacement,
    PluginRemote,
    PluginValue,
    RegisteredSelection,
    RegisteredSelectionParametersEntry,
    ReportSubject,
    SelectionPlan,
    SelectionValues,
)
from dokime.schemas import FrozenSchemaCache

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def journey(phase: str, url: str, plugin_url: str, expected: Path) -> None:
    # Connect Through the Composed Client Surface
    client = Client.connect(url)
    try:
        if phase == "read":
            # Resolve the Original Frozen Evidence After Process Restart
            saved = ExecutionDetail.FromString(expected.read_bytes())
            identifier = saved.summary.execution_id
            detail = (await client.executions.get(identifier)).execution
            assert detail == saved
            records = await client.evidence.records.list(
                EvidenceRecordScope(execution=identifier),
                "",
                [],
                Window(),
                "",
                EvidenceCapturedRecordSelection(),
            )
            samples = await client.evidence.samples.list(
                EvidenceScope(execution_id=identifier),
                ["thermal.temperature"],
                [],
                [],
                [],
                [],
                Window(),
                EvidenceBucketRequest(),
                False,
            )
            assert (
                records.SerializeToString(deterministic=True)
                == expected.with_suffix(".records").read_bytes()
            )
            assert (
                samples.SerializeToString(deterministic=True)
                == expected.with_suffix(".samples").read_bytes()
            )
            return

        # Create the Library Plugin Entry and Read Its Product Catalog
        placement = PluginPlacement(remote=PluginRemote(url=plugin_url))
        entry = (
            await client.plugins.create("thermal", placement, PluginValue(enabled=True))
        ).entry
        changes = client.plugins.watch(name=entry.name)
        try:
            while entry.state != PLUGIN_STATE_READY:
                if entry.HasField("failure"):
                    raise RuntimeError(f"Await Thermal Plugin Ready: {entry.failure}")
                if await anext(changes, None) is None:
                    raise RuntimeError("Await Thermal Plugin Ready: Watch Closed")
                entry = (await client.plugins.get(entry.name)).entry
        finally:
            await changes.aclose()
        assert (await client.plugins.list()).entries == [entry]
        suites = await client.suites.list("")
        suite_id = suites.values[0].metadata.identifier
        suite = (await client.suites.get(suite_id, "", 0)).value
        assert suite.templates[0].metadata.identifier == "thermal.cycle"

        # Preview the Selection and Admit Its Frozen Origin
        selection = RegisteredSelection(
            suite_id=suite_id,
            template_id="thermal.cycle",
            parameters=[
                RegisteredSelectionParametersEntry(
                    key="limit", value=SelectionValues(values=[Value(number=85.0)])
                )
            ],
        )
        cases = await client.cases.create(SelectionPlan(selections=[selection]), True)
        cycle = await client.cycles.create(
            [case.origin for case in cases.cases], Metadata(label="Thermal Cycle"), True, ""
        )
        admitted = await client.executions.list(
            cycle.cycle.cycle_id,
            ExecutionSelection(),
            ExecutionFilter(),
            ExecutionOrder(),
        )
        identifier = admitted.executions[0].summary.execution_id

        # Await the Prompt and Its Durable Answer Through Composed Resources
        phases: list[str] = []
        async for progress in client.executions.progress.watch(identifier):
            phases.append(progress.phase)
            if progress.phase != "prompt":
                continue
            pending = await client.prompts.list(identifier, True)
            prompt_id = pending.prompts[0].prompt.prompt_id
            prompt = await client.prompts.get(prompt_id)
            assert prompt.prompt.prompt.prompt_id == prompt_id
            await client.prompts.answers.create(prompt_id, "yes", MapValue(), True)
        assert "prompt" in phases and "finished" in phases

        # Decode the Captured Record Using Its Frozen Descriptors
        detail = (await client.executions.get(identifier)).execution
        assert detail.summary.state == 3 and detail.seal
        assert detail.outcome.evaluated.status == RESULT_STATUS_PASSED
        listed = await client.executions.list(
            "", ExecutionSelection(), ExecutionFilter(), ExecutionOrder()
        )
        assert [execution.summary for execution in listed.executions] == [
            detail.summary
        ]
        records = await client.evidence.records.list(
            EvidenceRecordScope(execution=identifier),
            "",
            [],
            Window(),
            "",
            EvidenceCapturedRecordSelection(),
        )
        assert len(records.records) == 1 and records.records[0].payload
        schemas = FrozenSchemaCache()
        door = await schemas.decode(
            client, detail, records.records[0].record_type, records.records[0].payload
        )
        fields = {field.name: value for field, value in door.ListFields()}
        assert fields["closed"] is True
        samples = await client.evidence.samples.list(
            EvidenceScope(execution_id=identifier),
            ["thermal.temperature"],
            [],
            [],
            [],
            [],
            Window(),
            EvidenceBucketRequest(),
            False,
        )
        assert all(sample.WhichOneof("kind") == "raw" for sample in samples.samples)
        times = [
            (sample.raw.timestamp.seconds, sample.raw.timestamp.nanos)
            for sample in samples.samples
        ]
        assert times == [(1, 0), (2, 0)], f"Sample the Hold at 1 s and 2 s: {times}"
        assert all(sample.raw.payload for sample in samples.samples)
        assert all(
            sample.raw.values.values["celsius"].number == 84.0
            for sample in samples.samples
        )

        # Read the Prepared Report Through the Shared File Resource
        prepared = await client.reports.create(ReportSubject(execution=identifier), "")
        report = await client.files.read(prepared.file, 0, prepared.file.length)
        assert identifier in report.data.decode()
        expected.write_bytes(detail.SerializeToString(deterministic=True))
        expected.with_suffix(".records").write_bytes(
            records.SerializeToString(deterministic=True)
        )
        expected.with_suffix(".samples").write_bytes(
            samples.SerializeToString(deterministic=True)
        )
    finally:
        await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(journey(sys.argv[1], sys.argv[2], sys.argv[3], Path(sys.argv[4])))
