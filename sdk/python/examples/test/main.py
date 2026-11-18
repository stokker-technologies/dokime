# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Start the Plugin and Drive Its Suite Through a Running Host."""

# === Imports === #

# Standard Library Imports
import argparse
import asyncio
from uuid import uuid4

# Local Imports
from suite import plugin

# Third Party Imports
from dokime_sdk import MapValue, Metadata
from dokime_sdk.client import (
    PLUGIN_STATE_READY,
    ExecutionBinding,
    PluginPlacement,
    PluginRemote,
    PluginValue,
    RegisteredSelection,
    SelectionPlan,
)
from dokime_sdk.connection import connect

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", help="Host Endpoint or Pairing File; Defaults to DOKIME_REMOTE")
    options = parser.parse_args()
    client = await connect(options.host)
    endpoint = await plugin.start("127.0.0.1:0")
    try:
        created = await client.plugins.create("example", PluginPlacement(remote=PluginRemote(url=endpoint.url)), PluginValue(enabled=True))
        try:
            entry = created.entry
            watching = client.plugins.watch(name=entry.name)
            try:
                while entry.state != PLUGIN_STATE_READY:
                    if entry.HasField("failure"):
                        raise RuntimeError(f"Start Plugin: Resolve the Reported Failure: {entry.failure}")
                    if await anext(watching, None) is None:
                        raise RuntimeError("Start Plugin: Reconnect to the Host After Its Watch Closed")
                    entry = (await client.plugins.get(entry.name)).entry
            finally:
                await watching.aclose()
            suites = (await client.suites.list(entry.name)).values
            suite = (await client.suites.get(suites[0].metadata.identifier, entry.name, 1)).value
            plan = SelectionPlan(
                selections=[RegisteredSelection(suite_id=suite.metadata.identifier, template_id=suite.templates[0].metadata.identifier)]
            )
            cases = (await client.cases.create(plan, False)).cases
            cycle = (await client.cycles.create([], Metadata(label="Capture"), True, "")).cycle
            execution = str(uuid4())
            await client.executions.create(cycle.cycle_id, [ExecutionBinding(execution_id=execution, case_id=cases[0].case_id)])
            async for progress in client.executions.progress.watch(execution):
                if progress.phase == "prompt":
                    pending = await client.prompts.list("", True, execution_id=execution)
                    for item in pending.prompts:
                        question = (await client.prompts.get(item.prompt.prompt_id)).prompt
                        print(question.prompt.definition.message)
                        for action in question.prompt.definition.actions:
                            print(f"{action.identifier}: {action.label}")
                        answer = await asyncio.to_thread(input, "Choose an Action: ")
                        await client.prompts.answers.create(item.prompt.prompt_id, answer, MapValue(), True)
            print((await client.executions.get(execution)).execution)
        finally:
            current = (await client.plugins.get(created.entry.name)).entry
            await client.plugins.delete(current.name, current.revision)
    finally:
        await endpoint.close()
        await client.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
