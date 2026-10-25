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
from urllib.parse import urlsplit

# Third Party Imports
from dokime_client.client import DokimeClient, Record, Sample

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ENDPOINT = "https://127.0.0.1:8766"
KEY_FILE: Path | None = Path("remote.key")
CA_CERTIFICATE: Path | None = Path("server.pem")
ENV_FILE: Path | None = None
EXECUTION_ID = "00000000-0000-0000-0000-000000000001"
RPC_TIMEOUT = 30.0
WAIT_SECONDS: float | None = None

# === Types === #

# === Classes === #

# === Public Functions === #


async def run() -> int:
    if urlsplit(ENDPOINT).scheme != "https":
        raise ValueError("remote control requires an HTTPS endpoint")
    async with DokimeClient(
        ENDPOINT,
        key_file=KEY_FILE,
        env_file=ENV_FILE,
        ca_certificate=CA_CERTIFICATE,
        rpc_timeout=RPC_TIMEOUT,
    ) as client:
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


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        raise SystemExit(asyncio.run(run()))
    except KeyboardInterrupt:
        print(f"Local Client Closed; Remote Work May Still Be Running. Execution: {EXECUTION_ID}", flush=True)
        raise SystemExit(130)
