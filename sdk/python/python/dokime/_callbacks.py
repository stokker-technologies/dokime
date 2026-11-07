# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Event Loop And Worker Callback Ownership"""

# === Imports === #

# Standard Library Imports
import asyncio
import concurrent.futures
import inspect
import json
import threading
import traceback
import warnings
from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass
class _Job:
    loop: asyncio.AbstractEventLoop
    name: str
    callback: Callable[..., Any]
    context: Any
    payload: str
    result: concurrent.futures.Future[str]
    task: asyncio.Task[None] | None = None
    worker: concurrent.futures.Future[Any] | None = None

    async def run(self) -> None:
        try:
            if inspect.iscoroutinefunction(self.callback):
                value = await self.callback(self.context, json.loads(self.payload))
            else:
                self.worker = concurrent.futures.Future()

                def work() -> None:
                    try:
                        assert self.worker is not None
                        self.worker.set_result(self.callback(self.context, json.loads(self.payload)))
                    except BaseException as error:  # noqa: BLE001 - Every Python Exception Must Settle The Native Callback.
                        assert self.worker is not None
                        self.worker.set_exception(error)

                threading.Thread(target=work, name=self.name, daemon=True).start()
                value = await asyncio.shield(asyncio.wrap_future(self.worker))
            if inspect.isawaitable(value):
                value = await value
            if not self.result.done():
                self.result.set_result(json.dumps(value, allow_nan=False))
        except asyncio.CancelledError:
            self.result.cancel()
            raise
        except BaseException:  # noqa: BLE001 - Carry SystemExit And Other Base Exceptions Back To The Rust Caller.
            if not self.result.done():
                self.result.set_exception(RuntimeError(traceback.format_exc()))

    async def stop(self, grace: float) -> None:
        if self.task is None:
            return
        self.task.cancel()
        await asyncio.sleep(0)
        pending: list[asyncio.Future[Any]] = [self.task] if not self.task.done() else []
        if self.worker is not None and not self.worker.done():
            pending.append(asyncio.wrap_future(self.worker))
        if pending:
            for future in pending:
                future.add_done_callback(_settled)
            _, unsettled = await asyncio.wait(pending, timeout=grace)
            if unsettled:
                warnings.warn(f"Callback {self.name} ignored cancellation", RuntimeWarning, stacklevel=2)


# === Public Functions === #


def start(loop: asyncio.AbstractEventLoop, callback: Callable[..., Any], context: Any, payload: str, name: str) -> _Job:
    job = _Job(loop, name, callback, context, payload, concurrent.futures.Future())

    def begin() -> None:
        job.task = loop.create_task(job.run(), name=name)

    loop.call_soon_threadsafe(begin)
    return job


async def wait(job: _Job) -> str:
    return await asyncio.wrap_future(job.result)


def cancel(job: _Job, grace: float) -> None:
    def stop() -> None:
        job.loop.create_task(job.stop(grace), name=f"Cancel {job.name}")

    job.loop.call_soon_threadsafe(stop)


# === Private Functions === #


def _settled(future: asyncio.Future[Any]) -> None:
    if not future.cancelled():
        future.exception()


# === Entry Point === #
