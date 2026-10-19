# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Answer One Execution Confirmation And Export Evidence"""

# === Imports === #

# Standard Library Imports
import argparse
import asyncio
import json
import sys
from contextlib import aclosing
from pathlib import Path
from uuid import uuid4

# Third Party Imports
from dokime.client import DokimeClient, DokimeError, TestExecutionHandle

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def run(arguments: argparse.Namespace) -> int:
    directory: Path = arguments.directory
    directory.mkdir(parents=True, exist_ok=True)
    for name in ("execution.zip", "cycle.zip"):
        destination = directory / name
        if destination.exists() and not arguments.overwrite:
            raise FileExistsError(f"{destination} exists; choose another directory or pass --overwrite")

    async with DokimeClient(
        arguments.endpoint,
        key_file=arguments.key_file,
        env_file=arguments.env_file,
        ca_certificate=arguments.ca_certificate,
    ) as client:
        cycle_id: str | None = None
        execution: TestExecutionHandle | None = None
        exported = False
        try:
            cycle_id = await client.create_cycle(label="Remote Confirmation Demonstration")
            print(f"Cycle: {cycle_id}", flush=True)
            (execution,) = await client.append_cases(
                cycle_id,
                suite="obc.checks",
                template="obc.checks.pulse",
                parameters={"require_confirmation": True},
            )
            request_key = str(uuid4())
            print(f"Execution: {execution.id} Request Key: {request_key} Mode: automatic", flush=True)
            await execution.start(request_key=request_key)

            async with asyncio.timeout(60):
                while True:
                    async with aclosing(execution.pending_prompts()) as reader:
                        pending = [prompt async for prompt in reader]
                    if pending:
                        if len(pending) != 1:
                            raise ValueError("the demonstration expects exactly one confirmation")
                        prompt = pending[0]
                        break
                    if await execution.checkpoint() is not None:
                        raise ValueError("execution sealed without the expected confirmation")
                    await asyncio.sleep(0.2)

            print(f"Prompt: {prompt.id} Execution: {execution.id}", flush=True)
            question = await execution.prompt(prompt.id)
            print("Question:", json.dumps(question), flush=True)
            print(f"Answer: {arguments.answer}", flush=True)
            await execution.respond(prompt.id, action=arguments.answer)
            outcome = await execution.wait()
            print(f"Saved Outcome: {outcome.status} Execution: {execution.id}", flush=True)
            await execution.export(directory / "execution.zip", overwrite=arguments.overwrite)
            exported = True
            cycle = await client.cycle(cycle_id)
            await cycle.seal()
            saved = await cycle.export(directory / "cycle.zip", overwrite=arguments.overwrite)
            print(f"Cycle Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)
            return 0 if outcome.status == "passed" else 1

        except asyncio.CancelledError:
            print("Cancellation Requested; Waiting For Saved Evidence", file=sys.stderr, flush=True)
            if cycle_id is None:
                print("Cycle Creation Was Not Confirmed; Inspect The Host Before Creating Another", file=sys.stderr, flush=True)
                raise
            if execution is not None and await execution.checkpoint() is None:
                await execution.stop()
            # Close Admission Before Waiting On An Interrupted Submission.
            cycle = await client.cycle(cycle_id)
            await cycle.seal()
            if execution is not None:
                outcome = await execution.wait()

        if execution is not None:
            print(f"Saved Outcome: {outcome.status} Execution: {execution.id}", flush=True)
            if not exported:
                await execution.export(directory / "execution.zip", overwrite=arguments.overwrite)
        saved = await cycle.export(directory / "cycle.zip", overwrite=arguments.overwrite)
        print(f"Cycle Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)
        return 130


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)

    parser.add_argument("--endpoint", required=True)
    parser.add_argument("--ca-certificate", type=Path)
    parser.add_argument("--key-file", type=Path)
    parser.add_argument("--env-file", type=Path, help="Read this explicit dotenv file; process variables take precedence")

    parser.add_argument("--directory", type=Path, default=Path("prompt-evidence"))
    parser.add_argument("--overwrite", action="store_true")
    parser.add_argument("--answer", choices=("yes", "no"), default="yes", help="The confirmation action this script submits once")
    arguments = parser.parse_args()

    try:
        return asyncio.run(run(arguments))
    except KeyboardInterrupt:
        print("Client Closed; Cancellation May Be Unconfirmed. Retain The Cycle, Execution And Request Key Above", file=sys.stderr)
        return 130
    except (DokimeError, OSError, ValueError, TimeoutError) as error:
        print(f"Workflow Incomplete: {error}", file=sys.stderr)
        print("Remote Work May Continue. Inspect The Printed Execution And Prompt Before Resubmitting", file=sys.stderr)
        return 1


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
