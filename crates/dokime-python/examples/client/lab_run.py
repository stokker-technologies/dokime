# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Observe a remote execution through its saved outcome. Edit constants, then run."""

# === Imports === #

# Standard Library Imports
import asyncio
from dataclasses import dataclass
import math
from pathlib import Path
import signal

# Third Party Imports
from rich.console import Console
from rich.live import Live
from rich.table import Table
from rich.text import Text

# Local Imports
from dokime.client import (
    DokimeClient,
    RunRequest,
    TestExecutionHandle,
    Value,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ENDPOINT = "http://127.0.0.1:8765"
SUITE = "mini.charger.steady"
TEMPLATE = "mini.charger.steady.measurement"
PARAMETERS: dict[str, Value] = {"duration_s": 1.0, "sample_interval_s": 0.1}

TOKEN_FILE: Path | None = None
CA_CERTIFICATE: Path | None = None


# Empty filters listen to everything; use IDs from your installed plugin.
RECORD_TYPES: tuple[str, ...] = ()  # Authored event kinds, not occurrence IDs.
STREAMS: tuple[str, ...] = ()

CANCEL_AFTER: float | None = None  # Seconds; None waits for completion or Ctrl-C.

# === Types === #

# === Classes === #


@dataclass
class ExecutionDisplay:
    """Latest display values; durable evidence belongs to the native backend."""

    record: str = "Waiting For Records"
    stream: str = "Waiting For Samples"
    status: str = "Waiting For Status"
    stopping: bool = False

    def render(self) -> Table:
        table = Table(title="Live Execution", expand=True)
        table.add_column("Listener", no_wrap=True)
        table.add_column("Latest Update", overflow="ellipsis")

        table.add_row(Text("Record", style="cyan"), Text(self.record))
        table.add_row(Text("Stream", style="magenta"), Text(self.stream))
        table.add_row(Text("Execution", style="green"), Text(self.status))

        if self.stopping:
            table.add_row(Text("Control", style="yellow"), Text("Stopping Execution…"))

        return table


# === Public Functions === #


async def main() -> None:
    if CANCEL_AFTER is not None and (
        not math.isfinite(CANCEL_AFTER) or CANCEL_AFTER <= 0
    ):
        raise ValueError("CANCEL_AFTER must be finite and greater than zero")

    token = TOKEN_FILE.read_text(encoding="utf-8").strip() if TOKEN_FILE else None
    client = DokimeClient(ENDPOINT, token=token, ca_certificate=CA_CERTIFICATE)

    try:
        cycle = await client.new_cycle(
            suite=SUITE, template=TEMPLATE, parameters=PARAMETERS
        )
        (handle,) = cycle.executions
        request = await handle.start(request_key="lab-example")
        await observe_execution(handle, request)

    finally:
        await client.close()


async def listen_records(
    handle: TestExecutionHandle, display: ExecutionDisplay
) -> None:
    """Handle captured events, replaying early records and following new ones."""
    async for record in handle.records(follow=True, record_types=RECORD_TYPES):
        display.record = (
            f"{record.type} · {record.id} · #{record.sequence} · {record.severity}"
        )


async def listen_streams(
    handle: TestExecutionHandle, display: ExecutionDisplay
) -> None:
    """Handle each stream sample, including the final samples after completion."""
    async for sample in handle.samples(follow=True, streams=STREAMS):
        display.stream = f"{sample.stream} · {sample.id} · #{sample.sequence}"


async def listen_status(request: RunRequest, display: ExecutionDisplay) -> None:
    """Handle changed execution receipts separately from captured event records."""
    async for status in request.updates():
        display.status = (
            f"{status.phase.value} · Finished: {status.finished}/{status.planned}"
        )


# === Private Functions === #


async def observe_execution(handle: TestExecutionHandle, request: RunRequest) -> None:
    console = Console()
    display = ExecutionDisplay()

    stop_requested = asyncio.Event()
    previous = signal.signal(signal.SIGINT, lambda *_: stop_requested.set())

    try:
        with Live(get_renderable=display.render, console=console, refresh_per_second=4):
            async with asyncio.TaskGroup() as tasks:
                tasks.create_task(listen_records(handle, display))
                tasks.create_task(listen_streams(handle, display))
                tasks.create_task(listen_status(request, display))

                stopper = tasks.create_task(
                    stop_when_requested(
                        handle, request, stop_requested, CANCEL_AFTER, display
                    )
                )

                outcome = await handle.wait()
                stopper.cancel()

        # Leaving the task group waits for every listener's final drain.
        style = "green" if outcome.status == "passed" else "yellow"
        console.print(Text(f"Outcome: {outcome.status.title()}", style=style))
        console.print(Text(f"Saved Execution: {outcome.checkpoint.execution_id}"))

    finally:
        signal.signal(signal.SIGINT, previous)


# Stop remote work, then let collection finish.
async def stop_when_requested(
    handle: TestExecutionHandle,
    request: RunRequest,
    requested: asyncio.Event,
    after: float | None,
    display: ExecutionDisplay,
) -> None:
    try:
        await asyncio.wait_for(requested.wait(), timeout=after)
    except TimeoutError:
        pass

    display.stopping = True
    async for status in request.updates():
        if status.terminal:
            return
        if status.last_execution_id is not None:
            if status.last_execution_id != handle.id:
                raise RuntimeError("request belongs to another execution")
            await handle.stop()
            return


# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
