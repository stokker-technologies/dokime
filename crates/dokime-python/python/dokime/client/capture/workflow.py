# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Durable local evidence capture composed from execution and history handles."""

# === Imports === #

# Standard Library Imports
import asyncio
from collections import deque
from collections.abc import AsyncIterator, Mapping
from dataclasses import asdict, dataclass
import json
from pathlib import Path
from typing import TYPE_CHECKING
import uuid

# Third Party Imports

# Local Imports
from .. import _wire
from ..execution import Cycle, RunRequest
from ..models import ResultSummary, RunStatus, Value
from ..history import ExecutionEvidence
from ..errors import ProtocolError
from .files import CaptureFiles, json_default, save_execution, write_json

# Type Checking Imports
if TYPE_CHECKING:
    from ..client import DokimeClient


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class CaptureUpdate:
    """Coarse local progress; never carries individual records or samples."""

    stage: str
    message: str


@dataclass(frozen=True)
class CaptureResult:
    output: Path
    request: RunStatus
    result: ResultSummary


class Capture:
    """One local capture. Owns persistence, never signals, printing or the client.

    Use prepare for a new cycle, open to recover an existing capture. Collection
    requires live cycle replay on the original backend. Device actions are not
    resumed after restart. A failed test may still have complete evidence.
    """

    def __init__(self, client: "DokimeClient", files: CaptureFiles) -> None:
        self._client = client
        self._files = files
        self._request: RunRequest | None = None
        self._result: CaptureResult | None = None
        self._busy = False
        self._finished = False
        self._sequence = 0
        self._updates: deque[tuple[int, CaptureUpdate]] = deque(maxlen=32)
        self._changed = asyncio.Event()

    @property
    def output(self) -> Path:
        return self._files.root

    @property
    def result(self) -> CaptureResult:
        if self._result is None:
            raise RuntimeError("capture has not completed successfully")
        return self._result

    @classmethod
    async def prepare(
        cls,
        client: "DokimeClient",
        *,
        output: str | Path,
        suite: str,
        template: str,
        parameters: Mapping[str, Value] | None = None,
        label: str | None = None,
    ) -> "Capture":
        """Reserve output and persist cycle/key before admitting device work."""
        # Validate all supplied values before creating a directory or remote cycle.
        if parameters is not None and not isinstance(parameters, Mapping):
            raise ValueError("parameters must be a mapping")
        supplied = dict(parameters) if parameters is not None else {}
        json.dumps(supplied, default=json_default, allow_nan=False)
        for key, item in supplied.items():
            if not isinstance(key, str):
                raise ValueError("parameter names must be strings")
            _wire.encode(item)
        root = Path(output)
        root.mkdir(parents=True, exist_ok=False)
        capture = cls(
            client, CaptureFiles(root, {"format_version": 1, "complete": False})
        )
        try:
            capture._stage("connecting", "Connecting...")
            backend = await client.connect()
            capture._files.state["backend_id"] = backend.id
            capture._stage("creating_cycle", "Creating Cycle...")
            cycle_id = await client.create_cycle(
                suite=suite, template=template, parameters=supplied, label=label
            )
            capture._files.state.update(
                cycle_id=cycle_id, request_key=str(uuid.uuid4())
            )
            capture._stage("prepared", "Ready To Start...")
            return capture
        except BaseException:
            capture._finish("failed", "Capture Preparation Failed")
            raise

    @classmethod
    async def open(cls, client: "DokimeClient", *, output: str | Path) -> "Capture":
        """Read saved identity without submitting or creating a new cycle."""
        root = Path(output)
        state = json.loads((root / "capture.json").read_text(encoding="utf-8"))
        if not isinstance(state, dict) or state.get("format_version") != 1:
            raise ValueError("unsupported capture format")
        for key in ("backend_id", "cycle_id", "request_key"):
            if not isinstance(state.get(key), str) or not state[key]:
                raise ValueError(
                    "cycle creation was not confirmed; inspect the backend before starting again"
                )
        if "request_id" in state and (
            not isinstance(state["request_id"], str) or not state["request_id"]
        ):
            raise ValueError("invalid saved request identity")
        backend = await client.connect()
        if state["backend_id"] != backend.id:
            raise ValueError("capture belongs to another backend")
        capture = cls(client, CaptureFiles(root, state))
        capture._emit("opened", "Opened Existing Capture...")
        return capture

    async def start(self) -> RunRequest:
        """Submit or resolve the persisted admission key; never create a cycle."""
        if self._busy:
            raise RuntimeError("capture is already collecting or submitting")
        self._busy = True
        self._finished = False
        try:
            with self._files.lease():
                return await self._start()
        except BaseException:
            if not self._finished:
                self._finish("failed", "Execution Admission Failed")
            raise
        finally:
            self._busy = False

    async def _start(self) -> RunRequest:
        self._finished = False
        self._result = None
        self._request = None
        state = self._files.state
        state["complete"] = False
        try:
            if (await self._client.connect()).id != state["backend_id"]:
                raise ValueError("capture belongs to another backend")
            self._stage("submitting", "Starting Execution...")
            if state.get("request_id"):
                request = await self._client.get_run_request(state["request_id"])
            else:
                cycle = await self._client.cycle(state["cycle_id"])
                (control,) = cycle.executions
                request = await control.start(request_key=state["request_key"])
                await request.status()
            if request.cycle_id != state["cycle_id"]:
                raise ValueError("saved request belongs to another cycle")
            state["request_id"] = request.id
            self._request = request
            self._stage("admitted", "Execution Admitted...")
            return request
        except BaseException:
            self._finish("failed", "Execution Admission Failed")
            raise

    async def updates(self) -> AsyncIterator[CaptureUpdate]:
        """Replay up to 32 recent updates and follow until success or failure.

        Slow listeners may miss intermediate updates. No listener can backpressure
        acquisition, and late listeners always see the final stage.
        """
        seen = 0
        while True:
            changed = self._changed
            for sequence, update in tuple(self._updates):
                if sequence > seen:
                    seen = sequence
                    yield update
            if self._finished and seen == self._sequence:
                return
            await changed.wait()

    async def collect(
        self, request: RunRequest, *, report: str = "cycle.summary"
    ) -> CaptureResult:
        """Collect through request completion and final drain, then export history.

        Local cancellation closes files and leaves an incomplete capture; it does
        not stop remote work. Use the execution control handle explicitly to cancel work.
        """
        if self._busy:
            raise RuntimeError("capture is already collecting or submitting")
        self._busy = True
        self._finished = False
        try:
            if request is not self._request:
                raise ValueError("use the request returned by this capture's start()")
            with self._files.lease():
                return await self._collect(request, report=report)
        except BaseException:
            if not self._finished:
                self._finish("failed", "Capture Incomplete")
            raise
        finally:
            self._busy = False

    async def _collect(self, request: RunRequest, *, report: str) -> CaptureResult:
        self._finished = False
        self._result = None
        files = self._files
        cycle = await Cycle._bind(
            request._transport, request.cycle_id, request._backend
        )
        files.state["complete"] = False
        try:
            self._stage("capturing", "Capturing Data...")
            files.open_live()
            async with asyncio.TaskGroup() as tasks:
                tasks.create_task(self._records(request, cycle))
                tasks.create_task(self._streams(request, cycle))
                prompts = tasks.create_task(self._prompts(request, cycle))
                receipt = await request.wait()
                prompts.cancel()
            files.finish_live()
            files.state["request"] = asdict(receipt)
            self._stage("finalizing", "Finalizing Capture...")
            await self._export(request, cycle)
            self._stage("report", "Downloading Report...")
            report_path = self.output / "report.html"
            report_path.with_name("report.html.part").unlink(missing_ok=True)
            download = await cycle.download_report(report_path, identifier=report)
            result = await cycle.result()
            files.state.update(
                report=asdict(download), result=asdict(result), complete=True
            )
            self._stage("complete", "Capture Complete")
            self._result = CaptureResult(self.output, receipt, result)
            self._finish("complete", "Capture Complete")
            return self._result
        except BaseException:
            files.state["complete"] = False
            # Keep the last persisted acquisition stage for recovery. Reporting a
            # failure must not hide the original I/O/transport exception.
            self._finish("failed", "Capture Incomplete")
            raise
        finally:
            files.close()

    async def _records(self, request: RunRequest, cycle: Cycle) -> None:
        async for record in cycle.records(until=request):
            self._files.append("records", record)

    async def _streams(self, request: RunRequest, cycle: Cycle) -> None:
        async for sample in cycle.samples(until=request):
            self._files.append("streams", sample)

    async def _prompts(self, request: RunRequest, cycle: Cycle) -> None:
        waiting = False
        while True:
            pending = bool([prompt async for prompt in cycle.pending_prompts()])
            if pending != waiting:
                self._emit(
                    "waiting" if pending else "capturing",
                    "Waiting For Operator Input — Answer In Dokime..."
                    if pending
                    else "Capturing Data...",
                )
                waiting = pending
            await asyncio.sleep(0.5)

    async def _export(self, request: RunRequest, cycle: Cycle) -> None:
        executions = []
        for handle in cycle.executions:
            point = await handle.checkpoint()
            if point is None:
                raise ProtocolError("execution has no saved seal")
            executions.append(
                ExecutionEvidence(cycle._transport, point, cycle._backend)
            )
        self._files.state["checkpoints"] = [
            asdict(item.checkpoint) for item in executions
        ]
        self._files.save()
        write_json(self.output / "cycle.json", await cycle.document())
        for execution in executions:
            await save_execution(execution, self._files)

    def _stage(self, stage: str, message: str) -> None:
        self._files.state["stage"] = stage
        self._files.save()
        self._emit(stage, message)

    def _emit(self, stage: str, message: str) -> None:
        self._sequence += 1
        self._updates.append((self._sequence, CaptureUpdate(stage, message)))
        previous, self._changed = self._changed, asyncio.Event()
        previous.set()

    def _finish(self, stage: str, message: str) -> None:
        self._finished = True
        if not self._updates or self._updates[-1][1].stage != stage:
            self._emit(stage, message)
        else:
            self._changed.set()


# === Public Functions === #


# === Private Functions === #


# === Entry Point === #
