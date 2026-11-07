# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Settle Python Callback Cancellation and Carry Failures"""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import threading
from types import SimpleNamespace

# Third Party Imports
import pytest

# Local Imports
from dokime import _callbacks

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_callback_shutdown() -> None:
    async def story() -> None:
        loop = asyncio.get_running_loop()
        entered = asyncio.Event()
        exited = asyncio.Event()
        context = SimpleNamespace(value=3)

        async def callback(ctx: SimpleNamespace, payload: int) -> int:
            entered.set()
            try:
                await asyncio.Event().wait()
            finally:
                exited.set()
            return ctx.value + payload

        job = _callbacks.start(loop, callback, context, "4", "cancelled")
        await entered.wait()
        _callbacks.cancel(job, 0.0)
        await exited.wait()
        with pytest.raises(asyncio.CancelledError):
            await _callbacks.wait(job)

        worker_thread: list[int] = []

        def worker(ctx: SimpleNamespace, payload: int) -> int:
            worker_thread.append(threading.get_ident())
            return ctx.value + payload

        job = _callbacks.start(loop, worker, context, "4", "worker")
        assert json.loads(await _callbacks.wait(job)) == 7
        assert len(worker_thread) == 1
        assert worker_thread[0] != threading.get_ident()

        async def fail(ctx: SimpleNamespace, payload: int) -> None:
            raise ArithmeticError(payload)

        job = _callbacks.start(loop, fail, context, "9", "failure")
        with pytest.raises(RuntimeError) as caught:
            await _callbacks.wait(job)
        assert "ArithmeticError" in str(caught.value)
        assert "fail" in str(caught.value)

    asyncio.run(story())


# === Private Functions === #

# === Entry Point === #
