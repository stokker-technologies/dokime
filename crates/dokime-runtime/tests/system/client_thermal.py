# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Run and Reopen Thermal Evidence Through Bound SDK Handles."""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import sys
from dataclasses import asdict
from pathlib import Path

# Local Imports
from dokime_sdk import PromptAnswer, PromptQuestion, ResultStatus, connect

# Third Party Imports
from thermal_example.plugin import (
    Door,
    Temperature,
    ThermalCycleTestTemplate,
    ThermalPlugin,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def journey(phase: str, url: str, expected: Path) -> None:
    # Connect to the Same Host Before and After Its Process Restart.
    client = await connect(url)
    try:
        if phase == "read":
            saved = json.loads(expected.read_text(encoding="utf-8"))
            execution = await client.executions.get(saved["identifier"])
            cycle = await client.cycles.get(saved["cycle"])
            outcome = await execution.seal()
            assert asdict(outcome) == saved["outcome"]
        else:
            # Admit the Python Class and Let the Script's Operator Answer Once.
            await client.plugins.register(ThermalPlugin())

            async def answer(question: PromptQuestion) -> PromptAnswer:
                return PromptAnswer(question.choices[0])

            client.on_prompt(answer)
            cycle = await client.cycle(label="Thermal Cycle")
            cases = await client.cases.materialize(
                ThermalCycleTestTemplate, parameters={"limit": [85.0]}
            )
            assert len(cases) == 1
            execution = await cycle.execution(cases[0])
            phases = []
            async for progress in execution.run():
                phases.append(progress.phase)
            assert "prompt" in phases
            outcome = await execution.seal()
            assert outcome.status == ResultStatus.PASSED
            await cycle.seal()

        # Read Frozen Rows Using the Product Handles and the Fixture's Row Types.
        records = await execution.records(
            "thermal.door", row_type=Door
        ).read()
        assert len(records) == 1
        assert (
            records[0].value.SerializeToString()
            == Door(closed=True).SerializeToString()
        )
        samples = await execution.samples(
            "thermal.temperature", row_type=Temperature
        ).collect()
        assert [sample.timestamp_ns for sample in samples] == [
            1_000_000_000,
            2_000_000_000,
        ]
        # The Example Observer Approaches 84 °C From 23 °C on Each Granted Sample.
        assert [sample.value.SerializeToString() for sample in samples] == [
            Temperature(celsius=celsius).SerializeToString()
            for celsius in (38.25, 49.6875)
        ]
        retained = {
            "identifier": execution.identifier,
            "cycle": cycle.identifier,
            "outcome": asdict(outcome),
            "records": [
                (
                    row.identifier,
                    row.sequence,
                    row.timestamp_ns,
                    row.value.SerializeToString().hex(),
                )
                for row in records
            ],
            "samples": [
                (
                    row.identifier,
                    row.sequence,
                    row.timestamp_ns,
                    row.value.SerializeToString().hex(),
                )
                for row in samples
            ],
        }
        normalized = json.dumps(retained, sort_keys=True)
        if phase == "read":
            assert normalized == expected.read_text(encoding="utf-8")
            assert await cycle.evidence() == expected.with_suffix(".zip").read_bytes()
        else:
            expected.write_text(normalized, encoding="utf-8")
            expected.with_suffix(".zip").write_bytes(await cycle.evidence())
    finally:
        await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(journey(sys.argv[1], sys.argv[2], Path(sys.argv[3])))
