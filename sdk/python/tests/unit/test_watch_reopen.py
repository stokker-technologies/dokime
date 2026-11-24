# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Watches Outlive the Host's Per-Watch Deadline."""

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator
from types import SimpleNamespace
from unittest import IsolatedAsyncioTestCase
from unittest.mock import Mock

# Third Party Imports
from grpc import StatusCode
from grpc.aio import AioRpcError, Metadata

# Local Imports
from dokime_sdk import Client, Execution, Progress
from dokime_sdk._generated import client_types
from dokime_sdk.operations import OperationError

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

type Pass = tuple[list[client_types.ProgressWatchResponse], StatusCode | None]

# === Classes === #


class WatchReopenTests(IsolatedAsyncioTestCase):
    def _execution(self, *passes: Pass) -> Execution:
        self.opened = 0

        async def watch(name: str) -> AsyncGenerator[client_types.ProgressWatchResponse]:
            frames, failure = passes[self.opened]
            self.opened += 1
            for frame in frames:
                yield frame
            if failure is not None:
                raise AioRpcError(failure, Metadata(), Metadata(), f"Watch Progress: {failure.name}")

        client = Mock(spec=Client)
        client._transport = SimpleNamespace(executions=SimpleNamespace(progress=SimpleNamespace(watch=watch)))
        detail = client_types.ExecutionDetail(summary={"execution_id": "execution", "case_id": "case"}, cycle_id="cycle")
        return Execution._bind(client, detail)

    async def test_progress_continues_across_the_host_deadline_without_a_repeat(self) -> None:
        execution = self._execution(([_frame(1), _frame(2)], StatusCode.DEADLINE_EXCEEDED), ([_frame(2), _frame(3, sealed=True)], None))
        progress = [value async for value in execution.watch()]
        self.assertEqual([(value.completed, value.sealed) for value in progress], [(1, False), (2, False), (3, True)])
        self.assertEqual(progress[1], Progress("running", 2, 3, "Step", False))
        self.assertEqual(self.opened, 2)

    async def test_other_failures_and_an_unsealed_end_still_raise(self) -> None:
        for passes, code in (((([_frame(1)], StatusCode.UNAVAILABLE),), "UNAVAILABLE"), ((([_frame(1)], None),), "invalid_response")):
            with self.assertRaises(OperationError) as raised:
                [value async for value in self._execution(*passes).watch()]
            self.assertEqual(raised.exception.value.code, code)
            self.assertEqual(self.opened, 1)


# === Public Functions === #

# === Private Functions === #


def _frame(completed: int, *, sealed: bool = False) -> client_types.ProgressWatchResponse:
    return client_types.ProgressWatchResponse(
        value={"completed": completed, "total": 3, "label": "Step"}, phase="sealed" if sealed else "running", sealed=sealed
    )


# === Entry Point === #
