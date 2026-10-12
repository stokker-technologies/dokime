# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Own Python callback tasks across cancellation of their Rust waiters."""

# === Imports === #

# Standard Library Imports
import asyncio
from collections.abc import Coroutine
from contextvars import Context
from typing import Any

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Callback:
    """One task owned until its finalizers finish, independently of Rust waiters."""

    def __init__(
        self,
        loop: asyncio.AbstractEventLoop,
        context: Context,
        coroutine: Coroutine[Any, Any, object],
    ) -> None:
        self._loop = loop
        self._context = context.copy()
        self._coroutine = coroutine
        self._task: asyncio.Task[object] | None = None
        self._done = asyncio.Event()
        self._cancel_requested = False
        self._start_error: BaseException | None = None

    def schedule(self) -> None:
        try:
            self._loop.call_soon_threadsafe(self._start, context=self._context)
        except BaseException:
            self._coroutine.close()
            raise

    def cancel(self) -> None:
        self._loop.call_soon_threadsafe(self._cancel)

    async def wait(self) -> object:
        await self._done.wait()
        if self._start_error is not None:
            raise self._start_error
        assert self._task is not None

        return self._task.result()

    def _start(self) -> None:
        try:
            self._task = self._loop.create_task(
                self._coroutine, context=self._context.copy()
            )
        except BaseException as error:
            self._coroutine.close()
            self._start_error = error
            self._done.set()
            return

        self._task.add_done_callback(self._finished)
        if self._cancel_requested:
            self._task.cancel()

    def _cancel(self) -> None:
        # Cancelling twice can interrupt an awaited finally block. Replacement
        # and shutdown may both request cancellation, but only the first acts.
        if self._cancel_requested:
            return
        self._cancel_requested = True
        if self._task is not None:
            self._task.cancel()

    def _finished(self, task: asyncio.Task[object]) -> None:
        # Retrieve exceptions even if Rust's first waiter was cancelled. The
        # result remains available to the owner that subsequently joins us.
        if not task.cancelled():
            task.exception()
        self._done.set()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
