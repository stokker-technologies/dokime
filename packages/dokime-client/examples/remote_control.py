# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Run One Remote Execution And Drain Its Records And Samples"""

# === Imports === #

# Standard Library Imports
import asyncio
import json
from pathlib import Path

# Third Party Imports
from dokime_client.client import DokimeClient

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

PAIRING_FILE = Path("lab.dokime-pair")
EXECUTION_ID = "00000000-0000-0000-0000-000000000001"
WAIT_SECONDS: float | None = None

# === Types === #

# === Classes === #

# === Public Functions === #


async def run() -> int:
    async with DokimeClient.from_pairing(PAIRING_FILE) as client:
        backend = await client.connect()
        print(f"Backend: {backend.id} Epoch: {backend.epoch}", flush=True)
        execution = await client.execution(EXECUTION_ID)
        print(
            f"Execution: {execution.id}",
            flush=True,
        )

        async def records() -> None:
            async for item in execution.records():
                print(item.to_json(), flush=True)

        async def samples() -> None:
            async for item in execution.samples():
                print(item.to_json(), flush=True)

        await execution.start()
        async with asyncio.timeout(WAIT_SECONDS), asyncio.TaskGroup() as tasks:
            tasks.create_task(records())
            tasks.create_task(samples())
            outcome = await execution
        print(
            f"Saved Outcome: {outcome.status} Execution: {outcome.checkpoint.execution_id}",
            flush=True,
        )
        Path("execution.json").write_text(
            json.dumps({"execution_id": execution.id, "status": outcome.status}, indent=2) + "\n", encoding="utf-8"
        )
        return outcome.exit_code


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        raise SystemExit(asyncio.run(run()))
    except KeyboardInterrupt:
        print(f"Local Client Closed; Remote Work May Still Be Running. Execution: {EXECUTION_ID}", flush=True)
        raise SystemExit(130)
