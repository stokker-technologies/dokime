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
        if phase == "read":
            # Read the Same Verified Execution, Records and Samples After Restart
            saved = message("ExecutionDetail")
            saved.ParseFromString(expected.read_bytes())
            identifier = saved.summary.execution_id
            detail = await client.executions.get(identifier)
            assert detail == saved
            records = await client.records.list(identifier)
            assert len(records.records) == 1 and records.records[0].payload
            samples = await client.samples.list(identifier)
            assert [sample.timestamp.seconds for sample in samples.samples] == [0, 1, 2]
            assert records.SerializeToString(deterministic=True) == expected.with_suffix(".records").read_bytes()
            assert samples.SerializeToString(deterministic=True) == expected.with_suffix(".samples").read_bytes()
            return

        # Add the Thermal URL and Read Its Catalog
        entry = await client.plugins.add(plugin_url)
        assert (await client.plugins.list()).plugins == [entry]
        suites = await client.suites.list()
        suite_id = suites[0].suite_id
        suite = await client.suites.get(suite_id)
        assert suite.templates[0].metadata.identifier == "thermal.cycle"

        # Materialize Parameters and Start Their Frozen Origins
        cases = await client.templates.materialize(suite_id, "thermal.cycle", {"limit": [85.0]})
        cycle = await client.cycles.start(cases)
        identifier = cycle.executions[0]

        # Await the Door Event and Answer Through the Prompts Service
        phases = []
        async for progress in cycle.watch():
            phases.append(progress.phase)
            if progress.phase == "prompt":
                pending = await client.prompts.pending(identifier)
                prompt_id = pending[0].prompt_id
                prompt = await client.prompts.get(prompt_id)
                assert prompt.prompt_id == prompt_id
                await client.prompts.respond(prompt_id, "yes", accepted=True)
        assert "prompt" in phases and "finished" in phases

        # Read the Sealed Execution, Record Payload and Controlled Stream Ticks
        detail = await client.executions.get(identifier)
        assert detail.summary.state == 3 and detail.seal
        assert (
            detail.outcome.evaluated.status
            == detail.outcome.evaluated.DESCRIPTOR.fields_by_name["status"].enum_type.values_by_name["RESULT_STATUS_PASSED"].number
        )
        assert await client.executions.list() == [detail.summary]
        records = await client.records.list(identifier)
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
        samples = await client.samples.list(identifier)
        assert [sample.timestamp.seconds for sample in samples.samples] == [0, 1, 2]
        assert all(sample.payload for sample in samples.samples)
        assert all(sample.values.values["celsius"].number == 84.0 for sample in samples.samples)

        # Verify the Ordered Report Stream and Its Digest Completion
        data = await client.reports.download(identifier)
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
