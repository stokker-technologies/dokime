# SPDX-License-Identifier: Apache-2.0

"""Run One Execution on a Remote Host Through Its .pair File and Save the Report and Evidence."""

# === Imports === #

# Standard Library Imports
import argparse
import asyncio
from pathlib import Path

# Third Party Imports
from dokime_sdk import PromptAnswer, PromptQuestion, connect

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def rehearse(pairing: Path, template: str, answer: str | None, output: Path) -> None:
    client = await connect(pairing)
    try:
        # List the Host's Catalog and Select the Requested Template.
        catalog = await client.cases.templates()
        for entry in catalog:
            print(f"Template {entry.identifier} ({entry.label}) in Suite {entry.suite}")
        selected = next((entry for entry in catalog if entry.identifier == template and entry.available), None)
        if selected is None:
            raise SystemExit(f"Rehearse: The Host Offers No Available Template {template}; Pick One Listed Above")

        # Answer Prompts From Here, or Leave Them to the Operator at the Host.
        if answer is not None:
            choice = answer

            async def respond(question: PromptQuestion) -> PromptAnswer:
                print(f"Prompt {question.text} {list(question.choices)}: Answering {choice}")
                return PromptAnswer(choice)

            client.on_prompt(respond)

        # Run One Execution and Stream Its Progress Until It Seals.
        cycle = await client.cycle(label="Remote Rehearsal")
        cases = await client.cases.materialize(selected.identifier, suite=selected.suite)
        execution = await cycle.execution(cases[0])
        async for progress in execution.run():
            print(f"Progress {progress.phase} {progress.completed}/{progress.total} {progress.label}")
        outcome = await execution.seal()
        print(f"Outcome {outcome.status.name} {outcome.reason}")

        # Seal the Cycle and Save Its Report and Evidence Archive.
        await cycle.seal()
        output.mkdir(parents=True, exist_ok=True)
        (output / "report.html").write_bytes(await cycle.report())
        (output / "evidence.zip").write_bytes(await cycle.evidence())
        print(f"Saved {output / 'report.html'} and {output / 'evidence.zip'}")
    finally:
        await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pairing", type=Path, help="Remote Access .pair File Exported by the Host")
    parser.add_argument("template", help="Template Identifier From the Host's Catalog")
    parser.add_argument("--answer", help="Choice Label Answering Every Prompt; Omit to Answer at the Host")
    parser.add_argument("--output", type=Path, default=Path("_output"), help="Directory for report.html and evidence.zip")
    arguments = parser.parse_args()
    asyncio.run(rehearse(arguments.pairing, arguments.template, arguments.answer, arguments.output))
