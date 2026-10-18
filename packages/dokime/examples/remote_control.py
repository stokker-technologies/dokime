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
import argparse
import asyncio
from pathlib import Path
from urllib.parse import urlsplit

# Third Party Imports
from dokime.client import DokimeClient, Record, Sample

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def run(arguments: argparse.Namespace) -> int:
    async with DokimeClient(
        arguments.endpoint,
        key_file=arguments.key_file,
        env_file=arguments.env_file,
        ca_certificate=arguments.ca_certificate,
        rpc_timeout=arguments.rpc_timeout,
    ) as client:
        backend = await client.connect()
        print(f"Backend: {backend.id} Epoch: {backend.epoch}", flush=True)
        execution = await client.execution(arguments.execution)
        print(
            f"Execution: {execution.id} Request Key: {arguments.request_key}",
            flush=True,
        )

        async def record(item: Record) -> None:
            print(f"Record: {item.type} #{item.sequence} {item.message or ''}", flush=True)

        async def sample(item: Sample) -> None:
            print(f"Sample: {item.stream} #{item.sequence} {item.values!r}", flush=True)

        outcome = await execution.run(
            request_key=arguments.request_key,
            on_record=record,
            on_sample=sample,
            timeout=arguments.timeout,
        )
        print(
            f"Saved Outcome: {outcome.status} Execution: {outcome.checkpoint.execution_id}",
            flush=True,
        )
        return 0 if outcome.status == "passed" else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--endpoint", required=True, help="Authenticated HTTPS listener")
    parser.add_argument("--key-file", type=Path)
    parser.add_argument("--env-file", type=Path)
    parser.add_argument("--ca-certificate", type=Path)
    parser.add_argument("--execution", required=True, help="Existing planned execution ID")
    parser.add_argument("--request-key", required=True, help="Retain this key for recovery")
    parser.add_argument("--rpc-timeout", type=float, default=30)
    parser.add_argument("--timeout", type=float, help="Local wait timeout; never stops remote work")
    arguments = parser.parse_args()
    if urlsplit(arguments.endpoint).scheme != "https":
        parser.error("remote control requires an HTTPS endpoint")
    try:
        return asyncio.run(run(arguments))
    except KeyboardInterrupt:
        print(
            f"Local Client Closed; Remote Work May Still Be Running. Execution: {arguments.execution} Request Key: {arguments.request_key}",
            flush=True,
        )
        return 130


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
