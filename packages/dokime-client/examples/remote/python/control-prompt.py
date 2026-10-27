# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Answer One Confirmation And Save Execution Evidence"""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import sys
from contextlib import aclosing
from pathlib import Path

# Third Party Imports
from dokime_client.client import Cycle, DokimeClient, DokimeError, Execution

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# Connection Parameters
ENDPOINT = "https://127.0.0.1:8766"
CA_CERTIFICATE: Path | None = Path("server.pem")
KEY_FILE: Path | None = Path("remote.key")
ENV_FILE: Path | None = None

# Case And Execution Parameters
TEMPLATE = "obc.checks.pulse"
CYCLE_LABEL = "Remote Confirmation Demonstration"
ANSWER = "yes"
WAIT_SECONDS = 60.0

# Evidence Files
OUTPUT_DIRECTORY = Path("prompt-evidence")
OVERWRITE = False

# === Types === #

# === Classes === #

# === Public Functions === #


async def run() -> int:
    if ANSWER not in ("yes", "no") or WAIT_SECONDS <= 0:
        raise ValueError("ANSWER must be yes or no and WAIT_SECONDS must be positive")
    OUTPUT_DIRECTORY.mkdir(parents=True, exist_ok=True)
    if (OUTPUT_DIRECTORY / "execution.zip").exists() and not OVERWRITE:
        raise FileExistsError("execution.zip exists; choose another output directory or enable OVERWRITE")
    if (OUTPUT_DIRECTORY / "cycle.zip").exists() and not OVERWRITE:
        raise FileExistsError("cycle.zip exists; choose another output directory or enable OVERWRITE")

    async with DokimeClient(ENDPOINT, ca_certificate=CA_CERTIFICATE, key_file=KEY_FILE, env_file=ENV_FILE) as client:
        cycle: Cycle | None = None
        execution: Execution | None = None
        exported = False
        try:
            # Materialize The Confirmation Case And Append Its Attempt
            template = await client.template(TEMPLATE)
            (test_case,) = await template.materialize(parameters={"require_confirmation": True})
            attempt = Execution(test_case)
            cycle = await client.create_cycle(label=CYCLE_LABEL, continue_automatically=False)
            print(f"Cycle: {cycle.id}", flush=True)
            Path("cycle.json").write_text(json.dumps({"cycle_id": cycle.id}, indent=2) + "\n", encoding="utf-8")
            print(f"Execution: {attempt.id}", flush=True)
            await cycle.append([attempt])
            execution = attempt

            # Start Without Blocking So The Script Can Answer Its Prompt
            await execution.start()
            # Python Lists Pending Prompts But Offers No Blocking Next-Prompt Call.
            # This One Bounded Wait Ends At A Prompt, A Saved Seal Or The Deadline.
            async with asyncio.timeout(WAIT_SECONDS):
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

            # Inspect And Answer This Exact Execution's Prompt Once
            print(f"Prompt: {prompt.id} Execution: {execution.id}", flush=True)
            question = await execution.prompt(prompt.id)
            print("Question:", json.dumps(question), flush=True)
            Path("prompt.json").write_text(json.dumps(question, indent=2) + "\n", encoding="utf-8")
            print(f"Answer: {ANSWER}", flush=True)
            await execution.respond(prompt.id, action=ANSWER)
            outcome = await execution.wait(timeout=WAIT_SECONDS)
            print(f"Saved Outcome: {outcome.status} Execution: {execution.id}", flush=True)
            saved = await execution.export(OUTPUT_DIRECTORY / "execution.zip", overwrite=OVERWRITE)
            print(f"Execution Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)
            exported = True
            await cycle.seal()
            saved = await cycle.export(OUTPUT_DIRECTORY / "cycle.zip", overwrite=OVERWRITE)
            print(f"Cycle Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)
            return 0

        except asyncio.CancelledError:
            # Recover Only This Script's Execution And Cycle
            print("Cancellation Requested; Waiting For Saved Evidence", file=sys.stderr, flush=True)
            if cycle is None:
                print("Cycle Creation Was Not Confirmed; Inspect The Server Before Creating Another", file=sys.stderr, flush=True)
                raise
            if execution is not None and await execution.checkpoint() is None:
                await execution.stop()
            await cycle.seal()
            if execution is not None:
                outcome = await execution.wait()
                print(f"Saved Outcome: {outcome.status} Execution: {execution.id}", flush=True)
                if not exported:
                    await execution.export(OUTPUT_DIRECTORY / "execution.zip", overwrite=OVERWRITE)
            saved = await cycle.export(OUTPUT_DIRECTORY / "cycle.zip", overwrite=OVERWRITE)
            print(f"Cycle Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)
            return 130


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        raise SystemExit(asyncio.run(run()))
    except KeyboardInterrupt:
        print("Recovery Was Not Confirmed; Retain The Printed Cycle And Execution Identities", file=sys.stderr)
        raise SystemExit(130)
    except (DokimeError, OSError, ValueError, TimeoutError) as error:
        print(f"Workflow Incomplete: {error}. Inspect The Printed Execution And Prompt Before Resubmitting", file=sys.stderr)
        raise SystemExit(1)
