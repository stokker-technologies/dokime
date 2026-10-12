# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Python Upload Producers Through Native Completion"""

# === Imports === #

# Standard Library Imports
import asyncio
from collections.abc import AsyncIterable
from contextvars import copy_context

# Third Party Imports

# Local Imports
from ._callbacks import Callback
from ._native.client import Client, ClientError, RequestStream

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def upload(client: Client, method: str, frames: AsyncIterable[bytes]) -> bytes:
    """Feed native bounded input and join the producer before releasing its owner."""
    handle = await client.upload(method)
    producer = Callback(
        asyncio.get_running_loop(), copy_context(), _feed(handle, frames)
    )
    try:
        producer.schedule()
    except BaseException:
        await handle.close()
        raise

    produced = asyncio.create_task(producer.wait())
    response = asyncio.ensure_future(handle.response())
    failure: BaseException | None = None
    try:
        done, _ = await asyncio.wait(
            (produced, response), return_when=asyncio.FIRST_COMPLETED
        )
        if produced in done:
            try:
                produced.result()
            except ClientError as error:
                # A peer refusal can close the input before the producer observes
                # its reply. Preserve the server status rather than queue closure.
                if error.kind != "closed":
                    raise
        return await response
    except BaseException as error:
        failure = error
        raise
    finally:
        cleanup = asyncio.create_task(
            _settle(handle, producer, produced, response, failure)
        )
        await _join(cleanup)


# === Private Functions === #


async def _feed(handle: RequestStream, frames: AsyncIterable[bytes]) -> None:
    iterator = aiter(frames)
    try:
        async for frame in iterator:
            await handle.send(frame)
        await handle.end_input()
    finally:
        close = getattr(iterator, "aclose", None)
        if close is not None:
            await _join(asyncio.ensure_future(close()))


async def _settle(
    handle: RequestStream,
    producer: Callback,
    produced: asyncio.Task[object],
    response: asyncio.Future[bytes],
    failure: BaseException | None,
) -> None:
    producer.cancel()
    response.cancel()
    try:
        outcomes = await asyncio.gather(produced, response, return_exceptions=True)
    finally:
        await handle.close()

    error = outcomes[0]
    if isinstance(error, ClientError) and error.kind == "closed":
        return
    if isinstance(error, BaseException) and not isinstance(
        error, asyncio.CancelledError
    ):
        if failure is None:
            raise error
        if error is not failure:
            failure.add_note(f"Upload producer cleanup also failed: {error!r}")


async def _join[T](task: asyncio.Future[T]) -> T:
    # Repeated cancellation must not interrupt an iterator's awaited finalizer.
    # The caller still receives cancellation after cleanup acknowledges stopping.
    cancelled: asyncio.CancelledError | None = None
    while not task.done():
        try:
            await asyncio.shield(task)
        except asyncio.CancelledError as error:
            cancelled = error

    result = task.result()
    if cancelled is not None:
        raise cancelled
    return result


# === Entry Point === #
