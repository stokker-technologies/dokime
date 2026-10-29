# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Run One Remote Execution And Drain Its Records And Samples"""

import asyncio
import json
from pathlib import Path

from dokime_client.client import DokimeClient, Record, Sample

PAIRING_FILE = Path("lab.dokime-pair")
EXECUTION_ID = "00000000-0000-0000-0000-000000000001"
WAIT_SECONDS: float | None = None


async def run() -> int:
    async with DokimeClient.from_pairing(PAIRING_FILE) as client:
        backend = await client.connect()
        print(f"Backend: {backend.id} Epoch: {backend.epoch}", flush=True)
        execution = await client.execution(EXECUTION_ID)
        print(
            f"Execution: {execution.id}",
            flush=True,
        )

        async def record(item: Record) -> None:
            print(f"Record: {item.type} #{item.sequence} {item.message or ''}", flush=True)

        async def sample(item: Sample) -> None:
            print(f"Sample: {item.stream} #{item.sequence} {item.values!r}", flush=True)

        outcome = await execution.run(
            on_record=record,
            on_sample=sample,
            timeout=WAIT_SECONDS,
        )
        print(
            f"Saved Outcome: {outcome.status} Execution: {outcome.checkpoint.execution_id}",
            flush=True,
        )
        Path("execution.json").write_text(
            json.dumps({"execution_id": execution.id, "status": outcome.status}, indent=2) + "\n", encoding="utf-8"
        )
        return 0


if __name__ == "__main__":
    try:
        raise SystemExit(asyncio.run(run()))
    except KeyboardInterrupt:
        print(f"Local Client Closed; Remote Work May Still Be Running. Execution: {EXECUTION_ID}", flush=True)
        raise SystemExit(130)
